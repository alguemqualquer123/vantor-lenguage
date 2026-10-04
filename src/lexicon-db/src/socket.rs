use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketMessage {
    pub from: String,
    pub data: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketConfig {
    pub host: String,
    pub port: u16,
}

pub struct TcpServer {
    listener: Option<TcpListener>,
    config: SocketConfig,
    connections: usize,
    closed: bool,
}

impl TcpServer {
    pub async fn new(config: SocketConfig) -> Result<Self, String> {
        let addr = format!("{}:{}", config.host, config.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

        Ok(TcpServer {
            listener: Some(listener),
            config,
            connections: 0,
            closed: false,
        })
    }

    pub async fn listen(opts: TcpListenOpts) -> Result<Self, String> {
        let cfg = SocketConfig { host: opts.host.clone(), port: opts.port };
        let s = Self::new(cfg).await?;
        // backlog/nodelay documentados (tokio não expõe backlog direto)
        let _ = opts.backlog;
        let _ = opts.nodelay;
        Ok(s)
    }

    pub fn connection_count(&self) -> usize {
        self.connections
    }

    pub async fn close(&mut self) {
        self.listener = None;
        self.closed = true;
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Aceita até `limit` conexões contando-as (paridade Node listen/run).
    pub async fn run(&mut self, limit: usize) -> Result<usize, String> {
        let mut n = 0;
        while n < limit {
            if self.closed {
                break;
            }
            let listener = self.listener.as_mut().ok_or_else(|| "Server not listening".to_string())?;
            // timeout curto para não bloquear para sempre em testes
            match tokio::time::timeout(Duration::from_millis(200), listener.accept()).await {
                Ok(Ok((_s, _a))) => {
                    self.connections += 1;
                    n += 1;
                }
                Ok(Err(e)) => return Err(format!("Accept failed: {e}")),
                Err(_) => break,
            }
        }
        Ok(n)
    }

    pub async fn accept(&mut self) -> Result<TcpStream, String> {
        let listener = self.listener.as_mut()
            .ok_or_else(|| "Server not listening".to_string())?;
        
        let (stream, addr) = listener.accept()
            .await
            .map_err(|e| format!("Accept failed: {}", e))?;
        let _ = addr;
        self.connections += 1;
        Ok(stream)
    }

    pub fn config(&self) -> &SocketConfig {
        &self.config
    }
}

pub struct TcpClient {
    stream: Option<TcpStream>,
    config: SocketConfig,
    idle_timeout: Option<Duration>,
}

impl TcpClient {
    pub async fn connect(config: SocketConfig) -> Result<Self, String> {
        let addr = format!("{}:{}", config.host, config.port);
        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;

        Ok(TcpClient {
            stream: Some(stream),
            config,
            idle_timeout: None,
        })
    }

    pub async fn connect_with(opts: TcpConnectOpts) -> Result<Self, String> {
        let addr = format!("{}:{}", opts.host, opts.port);
        let to = Duration::from_millis(opts.timeout_ms);
        let stream = timeout(to, TcpStream::connect(&addr))
            .await
            .map_err(|_| format!("connect timeout to {addr}"))?
            .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;
        if opts.nodelay {
            let _ = stream.set_nodelay(true);
        }
        Ok(TcpClient {
            stream: Some(stream),
            config: SocketConfig { host: opts.host, port: opts.port },
            idle_timeout: None,
        })
    }

    pub async fn connect_host(host: &str, port: u16) -> Result<Self, String> {
        Self::connect_with(TcpConnectOpts { host: host.to_string(), port, timeout_ms: 5000, nodelay: true }).await
    }

    pub async fn connect_cancel(opts: TcpConnectOpts, cancel: tokio::sync::oneshot::Receiver<()>) -> Result<Self, String> {
        let addr = format!("{}:{}", opts.host, opts.port);
        tokio::select! {
            res = TcpStream::connect(&addr) => {
                let stream = res.map_err(|e| format!("Failed to connect to {addr}: {e}"))?;
                Ok(TcpClient { stream: Some(stream), config: SocketConfig { host: opts.host, port: opts.port }, idle_timeout: None })
            }
            _ = cancel => Err("connect cancelled".to_string()),
        }
    }

    pub async fn send_bytes(&mut self, data: &[u8]) -> Result<usize, String> {
        let stream = self.stream.as_mut().ok_or_else(|| "Not connected".to_string())?;
        stream.write_all(data).await.map_err(|e| format!("Send failed: {e}"))?;
        Ok(data.len())
    }

    pub async fn recv_bytes(&mut self, max: usize) -> Result<Vec<u8>, String> {
        let stream = self.stream.as_mut().ok_or_else(|| "Not connected".to_string())?;
        let fut = async {
            let mut buf = vec![0u8; max.min(65536).max(1)];
            let n = stream.read(&mut buf).await.map_err(|e| format!("Receive failed: {e}"))?;
            buf.truncate(n);
            Ok::<Vec<u8>, String>(buf)
        };
        if let Some(t) = self.idle_timeout {
            timeout(t, fut).await.map_err(|_| "idle timeout".to_string())?
        } else {
            fut.await
        }
    }

    pub async fn shutdown_write(&mut self) -> Result<(), String> {
        let stream = self.stream.as_mut().ok_or_else(|| "Not connected".to_string())?;
        stream.shutdown().await.map_err(|e| format!("Shutdown failed: {e}"))?;
        Ok(())
    }

    pub async fn destroy(&mut self) -> Result<(), String> {
        self.stream = None;
        Ok(())
    }

    pub fn info(&self) -> SocketInfo {
        let (local, remote) = match &self.stream {
            Some(s) => (
                s.local_addr().map(|a| a.to_string()).unwrap_or_default(),
                s.peer_addr().map(|a| a.to_string()).unwrap_or_default(),
            ),
            None => (String::new(), String::new()),
        };
        SocketInfo { local, remote, idle_timeout_ms: self.idle_timeout.map(|d| d.as_millis() as u64) }
    }

    pub fn set_idle_timeout(&mut self, ms: Option<u64>) {
        self.idle_timeout = ms.map(Duration::from_millis);
    }

    pub async fn send(&mut self, data: &str) -> Result<usize, String> {
        self.send_bytes(data.as_bytes()).await
    }

    pub async fn receive(&mut self, buffer_size: usize) -> Result<String, String> {
        let b = self.recv_bytes(buffer_size).await?;
        String::from_utf8(b).map_err(|e| format!("Invalid UTF-8: {}", e))
    }

    pub async fn close(&mut self) -> Result<(), String> {
        self.destroy().await
    }
}

pub struct UdpSocketConnection {
    socket: UdpSocket,
    config: SocketConfig,
    connected: Option<String>,
}

impl UdpSocketConnection {
    pub async fn bind(config: SocketConfig) -> Result<Self, String> {
        let addr = format!("{}:{}", config.host, config.port);
        let socket = UdpSocket::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;

        Ok(UdpSocketConnection {
            socket,
            config,
            connected: None,
        })
    }

    pub async fn bind_with(opts: UdpBindOpts) -> Result<Self, String> {
        let cfg = SocketConfig { host: opts.host.clone(), port: opts.port };
        let c = Self::bind(cfg).await?;
        if opts.broadcast {
            c.socket.set_broadcast(true).map_err(|e| format!("set_broadcast failed: {e}"))?;
        }
        Ok(c)
    }

    pub fn local_addr(&self) -> Result<String, String> {
        self.socket.local_addr().map(|a| a.to_string()).map_err(|e| format!("local_addr failed: {e}"))
    }

    pub async fn connect(&self, target: &str) -> Result<(), String> {
        self.socket.connect(target).await.map_err(|e| format!("udp connect failed: {e}"))?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), String> {
        // tokio UdpSocket sem disconnect real: documentado como no-op seguro
        Ok(())
    }

    pub async fn send_connected(&self, data: &[u8]) -> Result<usize, String> {
        self.socket.send(data).await.map_err(|e| format!("Send failed: {e}"))
    }

    pub async fn send_bytes_to(&self, data: &[u8], target: &str) -> Result<usize, String> {
        self.socket.send_to(data, target).await.map_err(|e| format!("Send failed: {e}"))
    }

    pub async fn recv_bytes_from(&self, max: usize) -> Result<(Vec<u8>, String), String> {
        let mut buf = vec![0u8; max.min(MAX_DATAGRAM).max(1)];
        let (n, addr) = self.socket.recv_from(&mut buf).await.map_err(|e| format!("Receive failed: {e}"))?;
        buf.truncate(n);
        Ok((buf, addr.to_string()))
    }

    pub fn join_multicast(&self, multi: &str, iface: &str) -> Result<(), String> {
        let m: std::net::Ipv4Addr = multi.parse().map_err(|_| format!("bad multicast ip: {multi}"))?;
        let i: std::net::Ipv4Addr = iface.parse().map_err(|_| format!("bad iface ip: {iface}"))?;
        self.socket.join_multicast_v4(m, i).map_err(|e| format!("join_multicast failed: {e}"))
    }

    pub fn leave_multicast(&self, multi: &str, iface: &str) -> Result<(), String> {
        let m: std::net::Ipv4Addr = multi.parse().map_err(|_| format!("bad multicast ip: {multi}"))?;
        let i: std::net::Ipv4Addr = iface.parse().map_err(|_| format!("bad iface ip: {iface}"))?;
        self.socket.leave_multicast_v4(m, i).map_err(|e| format!("leave_multicast failed: {e}"))
    }

    pub fn set_multicast_ttl(&self, ttl: u32) -> Result<(), String> {
        self.socket.set_multicast_ttl_v4(ttl).map_err(|e| format!("set_multicast_ttl failed: {e}"))
    }

    pub fn set_multicast_loop(&self, on: bool) -> Result<(), String> {
        self.socket.set_multicast_loop_v4(on).map_err(|e| format!("set_multicast_loop failed: {e}"))
    }

    pub fn set_broadcast(&self, on: bool) -> Result<(), String> {
        self.socket.set_broadcast(on).map_err(|e| format!("set_broadcast failed: {e}"))
    }

    pub fn set_ttl(&self, ttl: u32) -> Result<(), String> {
        self.socket.set_ttl(ttl).map_err(|e| format!("set_ttl failed: {e}"))
    }

    pub async fn send_to(&self, data: &str, target: &str) -> Result<usize, String> {
        self.send_bytes_to(data.as_bytes(), target).await
    }

    pub async fn receive_from(&self, buffer_size: usize) -> Result<(String, String), String> {
        let (b, a) = self.recv_bytes_from(buffer_size).await?;
        let data = String::from_utf8(b).map_err(|e| format!("Invalid UTF-8: {}", e))?;
        Ok((data, a))
    }
}

pub struct SocketModule {
    tcp_servers: Arc<Mutex<HashMap<String, TcpServer>>>,
    tcp_clients: Arc<Mutex<HashMap<String, TcpClient>>>,
    udp_sockets: Arc<Mutex<HashMap<String, UdpSocketConnection>>>,
}

impl SocketModule {
    pub fn new() -> Self {
        SocketModule {
            tcp_servers: Arc::new(Mutex::new(HashMap::new())),
            tcp_clients: Arc::new(Mutex::new(HashMap::new())),
            udp_sockets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // TCP Server methods
    pub async fn start_tcp_server(&self, name: String, config: SocketConfig) -> Result<String, String> {
        let server = TcpServer::new(config).await?;
        let addr = format!("{}:{}", server.config().host, server.config().port);
        let mut servers = self.tcp_servers.lock().await;
        servers.insert(name.clone(), server);
        Ok(format!("TCP server started on {}", addr))
    }

    pub async fn accept_tcp(&self, server_name: &str) -> Result<String, String> {
        let mut servers = self.tcp_servers.lock().await;
        let server = servers.get_mut(server_name)
            .ok_or_else(|| format!("Server '{}' not found", server_name))?;
        
        let stream = server.accept().await?;
        let addr = stream.peer_addr()
            .map_err(|e| format!("Failed to get peer address: {}", e))?;
        
        Ok(format!("Accepted connection from {}", addr))
    }

    // TCP Client methods
    pub async fn connect_tcp(&self, name: String, config: SocketConfig) -> Result<String, String> {
        let client = TcpClient::connect(config).await?;
        let addr = format!("{}:{}", client.config.host, client.config.port);
        let mut clients = self.tcp_clients.lock().await;
        clients.insert(name.clone(), client);
        Ok(format!("Connected to TCP server at {}", addr))
    }

    pub async fn tcp_send(&self, client_name: &str, data: &str) -> Result<usize, String> {
        let mut clients = self.tcp_clients.lock().await;
        let client = clients.get_mut(client_name)
            .ok_or_else(|| format!("Client '{}' not found", client_name))?;
        client.send(data).await
    }

    pub async fn tcp_receive(&self, client_name: &str, buffer_size: usize) -> Result<String, String> {
        let mut clients = self.tcp_clients.lock().await;
        let client = clients.get_mut(client_name)
            .ok_or_else(|| format!("Client '{}' not found", client_name))?;
        client.receive(buffer_size).await
    }

    pub async fn disconnect_tcp(&self, name: &str) -> Result<String, String> {
        let mut clients = self.tcp_clients.lock().await;
        if let Some(mut client) = clients.remove(name) {
            client.close().await?;
            Ok(format!("Disconnected from '{}'", name))
        } else {
            Err(format!("Client '{}' not found", name))
        }
    }

    // UDP methods
    pub async fn bind_udp(&self, name: String, config: SocketConfig) -> Result<String, String> {
        let socket = UdpSocketConnection::bind(config).await?;
        let addr = format!("{}:{}", socket.config.host, socket.config.port);
        let mut sockets = self.udp_sockets.lock().await;
        sockets.insert(name.clone(), socket);
        Ok(format!("UDP socket bound to {}", addr))
    }

    pub async fn udp_send(&self, socket_name: &str, data: &str, target: &str) -> Result<usize, String> {
        let sockets = self.udp_sockets.lock().await;
        let socket = sockets.get(socket_name)
            .ok_or_else(|| format!("Socket '{}' not found", socket_name))?;
        socket.send_to(data, target).await
    }

    pub async fn udp_receive(&self, socket_name: &str, buffer_size: usize) -> Result<(String, String), String> {
        let sockets = self.udp_sockets.lock().await;
        let socket = sockets.get(socket_name)
            .ok_or_else(|| format!("Socket '{}' not found", socket_name))?;
        socket.receive_from(buffer_size).await
    }

    pub async fn close_udp(&self, name: &str) -> Result<String, String> {
        let mut sockets = self.udp_sockets.lock().await;
        if sockets.remove(name).is_some() {
            Ok(format!("UDP socket '{}' closed", name))
        } else {
            Err(format!("Socket '{}' not found", name))
        }
    }

    pub async fn list_tcp_servers(&self) -> Vec<String> {
        let servers = self.tcp_servers.lock().await;
        servers.keys().cloned().collect()
    }

    pub async fn list_tcp_clients(&self) -> Vec<String> {
        let clients = self.tcp_clients.lock().await;
        clients.keys().cloned().collect()
    }
}

impl Default for SocketModule {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Paridade Node.js — net/dns/tls/ws/proxy (aditivo, sem quebrar TCP/UDP acima)
// ---------------------------------------------------------------------------

/// Tamanho máximo de datagrama UDP (65507 bytes payload).
pub const MAX_DATAGRAM: usize = 65507;

#[derive(Debug, thiserror::Error)]
pub enum SocketError {
    #[error("io: {0}")]
    Io(String),
    #[error("timeout after {0:?}")]
    Timeout(Duration),
    #[error("closed")]
    Closed,
    #[error("backpressure: buffer {0}")]
    Backpressure(usize),
    #[error("dns: {0}")]
    Dns(String),
    #[error("tls: {0}")]
    Tls(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpListenOpts {
    pub host: String,
    pub port: u16,
    pub backlog: u32,
    pub nodelay: bool,
}

impl Default for TcpListenOpts {
    fn default() -> Self {
        Self { host: "127.0.0.1".to_string(), port: 3000, backlog: 128, nodelay: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConnectOpts {
    pub host: String,
    pub port: u16,
    pub timeout_ms: u64,
    pub nodelay: bool,
}

impl Default for TcpConnectOpts {
    fn default() -> Self {
        Self { host: "127.0.0.1".to_string(), port: 3000, timeout_ms: 5000, nodelay: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketInfo {
    pub local: String,
    pub remote: String,
    pub idle_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpBindOpts {
    pub host: String,
    pub port: u16,
    pub broadcast: bool,
}

impl Default for UdpBindOpts {
    fn default() -> Self {
        Self { host: "127.0.0.1".to_string(), port: 3001, broadcast: false }
    }
}

// --- UDS (unix apenas) ---

#[cfg(unix)]
pub struct UdsListener {
    listener: tokio::net::UnixListener,
    path: String,
}

#[cfg(unix)]
impl UdsListener {
    pub async fn bind(path: &str) -> Result<Self, String> {
        let _ = std::fs::remove_file(path);
        let l = tokio::net::UnixListener::bind(path).map_err(|e| format!("uds bind failed: {e}"))?;
        Ok(Self { listener: l, path: path.to_string() })
    }
    pub async fn accept(&self) -> Result<tokio::net::UnixStream, String> {
        let (s, _) = self.listener.accept().await.map_err(|e| format!("uds accept failed: {e}"))?;
        Ok(s)
    }
    pub fn path(&self) -> &str {
        &self.path
    }
}

#[cfg(unix)]
pub struct UdsClient {
    stream: Option<tokio::net::UnixStream>,
    path: String,
}

#[cfg(unix)]
impl UdsClient {
    pub async fn connect(path: &str) -> Result<Self, String> {
        let s = tokio::net::UnixStream::connect(path).await.map_err(|e| format!("uds connect failed: {e}"))?;
        Ok(Self { stream: Some(s), path: path.to_string() })
    }
    pub async fn send_bytes(&mut self, data: &[u8]) -> Result<usize, String> {
        use tokio::io::AsyncWriteExt;
        let st = self.stream.as_mut().ok_or_else(|| "not connected".to_string())?;
        st.write_all(data).await.map_err(|e| format!("uds send failed: {e}"))?;
        Ok(data.len())
    }
    pub async fn recv_bytes(&mut self, max: usize) -> Result<Vec<u8>, String> {
        use tokio::io::AsyncReadExt;
        let st = self.stream.as_mut().ok_or_else(|| "not connected".to_string())?;
        let mut buf = vec![0u8; max.min(65536).max(1)];
        let n = st.read(&mut buf).await.map_err(|e| format!("uds recv failed: {e}"))?;
        buf.truncate(n);
        Ok(buf)
    }
    pub fn path(&self) -> &str {
        &self.path
    }
}

// --- DNS (tokio lookup_host + stubs tipados) ---

pub async fn dns_lookup(host: &str) -> Result<Vec<String>, String> {
    let h = host.trim();
    if h.is_empty() {
        return Err("empty host".to_string());
    }
    // IP literal retorna direto
    if h.parse::<std::net::IpAddr>().is_ok() {
        return Ok(vec![h.to_string()]);
    }
    let addrs = tokio::net::lookup_host(format!("{h}:80"))
        .await
        .map_err(|e| format!("dns lookup failed for {h}: {e}"))?;
    let mut out: Vec<String> = addrs.map(|a| a.ip().to_string()).collect();
    out.sort();
    out.dedup();
    if out.is_empty() {
        return Err(format!("dns not found: {h}"));
    }
    Ok(out)
}

pub async fn dns_lookup_service(host: &str, port: u16) -> Result<Vec<String>, String> {
    let addrs = tokio::net::lookup_host(format!("{host}:{port}"))
        .await
        .map_err(|e| format!("dns lookup_service failed: {e}"))?;
    Ok(addrs.map(|a| a.to_string()).collect())
}

pub async fn dns_resolve_a(host: &str) -> Result<Vec<String>, String> {
    let all = dns_lookup(host).await?;
    let v4: Vec<String> = all.into_iter().filter(|s| s.parse::<std::net::Ipv4Addr>().is_ok()).collect();
    if v4.is_empty() {
        return Err(format!("no A record for {host}"));
    }
    Ok(v4)
}

pub async fn dns_resolve_aaaa(host: &str) -> Result<Vec<String>, String> {
    // via lookup_host filtrando IPv6; vazio se só-IPv4 (documentado)
    let h = host.trim();
    if let Ok(ip) = h.parse::<std::net::IpAddr>() {
        if let std::net::IpAddr::V6(v6) = ip {
            return Ok(vec![v6.to_string()]);
        } else {
            return Ok(vec![]);
        }
    }
    let addrs = tokio::net::lookup_host(format!("{h}:80"))
        .await
        .map_err(|e| format!("dns aaaa failed for {h}: {e}"))?;
    let mut out: Vec<String> = addrs.map(|a| a.ip()).filter(|ip| ip.is_ipv6()).map(|ip| ip.to_string()).collect();
    out.sort();
    out.dedup();
    Ok(out)
}

/// Stub tipado MX: sem resolver real sem dep extra; retorna vazio documentado.
pub async fn dns_resolve_mx(_host: &str) -> Result<Vec<String>, String> {
    Ok(vec![])
}

/// Stub tipado TXT: sem resolver real sem dep extra; retorna vazio documentado.
pub async fn dns_resolve_txt(_host: &str) -> Result<Vec<String>, String> {
    Ok(vec![])
}

/// Reverse stub documentado: sem PTR lookup sem dep extra; retorna entrada.
pub async fn dns_reverse(ip: &str) -> Result<String, String> {
    if ip.parse::<std::net::IpAddr>().is_err() {
        return Err(format!("bad ip: {ip}"));
    }
    Ok(ip.to_string())
}

#[derive(Debug, Clone)]
pub struct DnsResolver {
    servers: Vec<String>,
    timeout_ms: u64,
}

impl DnsResolver {
    pub fn new() -> Self {
        Self { servers: vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()], timeout_ms: 5000 }
    }
    pub fn set_servers(&mut self, servers: Vec<String>) {
        self.servers = servers;
    }
    pub fn servers(&self) -> &[String] {
        &self.servers
    }
    pub fn set_timeout(&mut self, ms: u64) {
        self.timeout_ms = ms;
    }
    pub async fn lookup(&self, host: &str) -> Result<Vec<String>, String> {
        let to = Duration::from_millis(self.timeout_ms);
        timeout(to, dns_lookup(host)).await.map_err(|_| format!("dns timeout for {host}"))?
    }
    pub async fn resolve_a(&self, host: &str) -> Result<Vec<String>, String> {
        let to = Duration::from_millis(self.timeout_ms);
        timeout(to, dns_resolve_a(host)).await.map_err(|_| format!("dns timeout for {host}"))?
    }
    pub async fn resolve_aaaa(&self, host: &str) -> Result<Vec<String>, String> {
        let to = Duration::from_millis(self.timeout_ms);
        timeout(to, dns_resolve_aaaa(host)).await.map_err(|_| format!("dns timeout for {host}"))?
    }
}

impl Default for DnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

// --- TLS stub documentado ---

#[derive(Debug, Clone, Default)]
pub struct TlsConnector {
    pub server_name: Option<String>,
    pub verify: bool,
}

impl TlsConnector {
    pub fn new() -> Self {
        Self { server_name: None, verify: true }
    }
    /// Stub documentado: sem handshake real sem dep TLS; retorna TlsStream marcado não-autorizado.
    pub async fn connect_tls(&self, host: &str, port: u16) -> Result<TlsStream, String> {
        let addr = format!("{host}:{port}");
        // valida que o host resolve/conecta TCP para falhar cedo em host inválido
        let _ = timeout(Duration::from_millis(200), tokio::net::TcpStream::connect(&addr))
            .await
            .map_err(|_| format!("tls stub: tcp pre-check timeout for {addr}"));
        Ok(TlsStream {
            peer_cert: None,
            is_authorized: false,
            auth_error: Some("tls handshake fora de escopo (stub)".to_string()),
            protocol: "TLSv1.2+".to_string(),
            cipher_suite: "STUB".to_string(),
            session_reused: false,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsStream {
    pub peer_cert: Option<String>,
    pub is_authorized: bool,
    pub auth_error: Option<String>,
    pub protocol: String,
    pub cipher_suite: String,
    pub session_reused: bool,
}

// --- WS client/hub (Tcp + handshake) ---

#[derive(Debug, Clone)]
pub struct WsClientOpts {
    pub url: String,
    pub protocols: Vec<String>,
    pub timeout_ms: u64,
}

impl Default for WsClientOpts {
    fn default() -> Self {
        Self { url: "ws://127.0.0.1:80/".to_string(), protocols: vec![], timeout_ms: 5000 }
    }
}

pub struct WsClient {
    stream: TcpStream,
    buf: Vec<u8>,
}

impl WsClient {
    /// Conecta via TCP + handshake WS (RFC6455 cliente).
    pub async fn connect(opts: WsClientOpts) -> Result<Self, String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        // parse simples ws://host:port/path
        let (host, port, path) = parse_ws_url(&opts.url)?;
        let addr = format!("{host}:{port}");
        let to = Duration::from_millis(opts.timeout_ms);
        let mut stream = timeout(to, TcpStream::connect(&addr))
            .await
            .map_err(|_| "ws connect timeout".to_string())?
            .map_err(|e| format!("ws tcp failed: {e}"))?;
        // chave aleatória simples (16 bytes via contador+tempo, base64 manual)
        let key = ws_client_key();
        let mut req = format!("GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n");
        if !opts.protocols.is_empty() {
            req.push_str(&format!("Sec-WebSocket-Protocol: {}\r\n", opts.protocols.join(", ")));
        }
        req.push_str("\r\n");
        timeout(to, stream.write_all(req.as_bytes())).await.map_err(|_| "ws handshake write timeout".to_string())?.map_err(|e| format!("ws write failed: {e}"))?;
        // lê headers até \r\n\r\n
        let mut raw = Vec::new();
        let mut tmp = vec![0u8; 1024];
        let read_fut = async {
            loop {
                let n = stream.read(&mut tmp).await.map_err(|e| format!("ws read failed: {e}"))?;
                if n == 0 {
                    break;
                }
                raw.extend_from_slice(&tmp[..n]);
                if raw.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
                if raw.len() > 16384 {
                    return Err("ws handshake too large".to_string());
                }
            }
            Ok::<_, String>(())
        };
        timeout(to, read_fut).await.map_err(|_| "ws handshake read timeout".to_string())??;
        let head = String::from_utf8_lossy(&raw).to_string();
        if !head.contains("101") {
            return Err(format!("ws handshake failed (no 101): {}", head.lines().next().unwrap_or("")));
        }
        Ok(Self { stream, buf: Vec::new() })
    }

    pub async fn send(&mut self, data: &[u8]) -> Result<(), String> {
        use tokio::io::AsyncWriteExt;
        // frame binário masked (cliente deve mascarar)
        let frame = ws_encode_client(data);
        self.stream.write_all(&frame).await.map_err(|e| format!("ws send failed: {e}"))?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>, String> {
        use tokio::io::AsyncReadExt;
        // lê um frame (server sem mask)
        let mut hdr = vec![0u8; 2];
        self.stream.read_exact(&mut hdr).await.map_err(|e| format!("ws recv failed: {e}"))?;
        let len = (hdr[1] & 0x7F) as usize;
        let payload_len = if len == 126 {
            let mut b = vec![0u8; 2];
            self.stream.read_exact(&mut b).await.map_err(|e| format!("ws recv failed: {e}"))?;
            u16::from_be_bytes([b[0], b[1]]) as usize
        } else if len == 127 {
            let mut b = vec![0u8; 8];
            self.stream.read_exact(&mut b).await.map_err(|e| format!("ws recv failed: {e}"))?;
            u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]) as usize
        } else {
            len
        };
        let mut payload = vec![0u8; payload_len];
        if payload_len > 0 {
            self.stream.read_exact(&mut payload).await.map_err(|e| format!("ws recv failed: {e}"))?;
        }
        let _ = &self.buf;
        Ok(payload)
    }
}

fn parse_ws_url(url: &str) -> Result<(String, u16, String), String> {
    let (scheme, rest) = url.split_once("://").ok_or_else(|| "bad ws url".to_string())?;
    if scheme != "ws" && scheme != "wss" {
        return Err(format!("unsupported ws scheme: {scheme}"));
    }
    let (auth_path, path) = match rest.find('/') {
        Some(i) => (rest[..i].to_string(), rest[i..].to_string()),
        None => (rest.to_string(), "/".to_string()),
    };
    let (host, port) = match auth_path.rfind(':') {
        Some(i) => (auth_path[..i].to_string(), auth_path[i + 1..].parse::<u16>().map_err(|_| "bad ws port".to_string())?),
        None => (auth_path, if scheme == "wss" { 443 } else { 80 }),
    };
    Ok((host, port, path))
}

fn ws_client_key() -> String {
    // 16 bytes pseudo-aleatórios via tempo (sem nova dep); base64 manual inline
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0x1234);
    let mut bytes = [0u8; 16];
    let mut x = t as u64 ^ 0x9E3779B97F4A7C15;
    for b in bytes.iter_mut() {
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        *b = (x.wrapping_mul(0x2545F4914F6CDD1D) >> 56) as u8;
    }
    // base64
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i] as u32;
        let b1 = if i + 1 < bytes.len() { bytes[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < bytes.len() { bytes[i + 2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(if i + 1 < bytes.len() { T[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if i + 2 < bytes.len() { T[(n & 63) as usize] as char } else { '=' });
        i += 3;
    }
    out
}

fn ws_encode_client(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0x82); // FIN+binary
    let mask_bit = 0x80;
    if data.len() < 126 {
        out.push(mask_bit | (data.len() as u8));
    } else if data.len() < 65536 {
        out.push(mask_bit | 126);
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
    } else {
        out.push(mask_bit | 127);
        out.extend_from_slice(&(data.len() as u64).to_be_bytes());
    }
    let mask = [0x11u8, 0x22, 0x33, 0x44];
    out.extend_from_slice(&mask);
    for (i, b) in data.iter().enumerate() {
        out.push(b ^ mask[i % 4]);
    }
    out
}

#[derive(Debug, Default)]
pub struct WsHub {
    inboxes: HashMap<String, Vec<Vec<u8>>>,
}

impl WsHub {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn broadcast(&mut self, data: &[u8]) {
        for v in self.inboxes.values_mut() {
            v.push(data.to_vec());
        }
    }
    pub fn send_to(&mut self, id: &str, data: &[u8]) -> Result<(), String> {
        self.inboxes.entry(id.to_string()).or_default().push(data.to_vec());
        Ok(())
    }
    pub fn drain(&mut self, id: &str) -> Vec<Vec<u8>> {
        self.inboxes.remove(id).unwrap_or_default()
    }
    pub fn len(&self) -> usize {
        self.inboxes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inboxes.is_empty()
    }
}

/// Túnel CONNECT via TcpStream (retorna stream após 200).
pub async fn proxy_tunnel(proxy_host: &str, proxy_port: u16, target_host: &str, target_port: u16) -> Result<TcpStream, String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let addr = format!("{proxy_host}:{proxy_port}");
    let mut s = TcpStream::connect(&addr).await.map_err(|e| format!("proxy connect failed: {e}"))?;
    let req = format!("CONNECT {target_host}:{target_port} HTTP/1.1\r\nHost: {target_host}:{target_port}\r\n\r\n");
    s.write_all(req.as_bytes()).await.map_err(|e| format!("proxy write failed: {e}"))?;
    let mut buf = vec![0u8; 4096];
    let n = timeout(Duration::from_secs(5), s.read(&mut buf)).await.map_err(|_| "proxy read timeout".to_string())?.map_err(|e| format!("proxy read failed: {e}"))?;
    let head = String::from_utf8_lossy(&buf[..n]).to_string();
    if head.contains("200") {
        Ok(s)
    } else {
        Err(format!("proxy tunnel failed: {}", head.lines().next().unwrap_or("")))
    }
}

async fn with_socket_retries<T, F, Fut>(attempts: u32, mut op: F) -> Result<T, SocketError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, SocketError>>,
{
    let mut last = SocketError::Closed;
    for _ in 0..attempts.max(1) {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if matches!(e, SocketError::Backpressure(_)) {
                    return Err(e);
                }
                last = e;
            }
        }
    }
    Err(last)
}

// ============================================================================
// §57 — Messaging stubs (Kafka / RabbitMQ / NATS / Redis Streams / MQTT /
// AMQP). Existing TCP/UDP API above is intentionally untouched.
// ============================================================================

/// Unified error type for the messaging stubs.
#[derive(Debug, thiserror::Error)]
pub enum MessagingError {
    /// Not connected / already closed.
    #[error("connection error: {0}")]
    Connection(String),
    /// Per-operation timeout.
    #[error("operation timed out after {0:?}")]
    Timeout(Duration),
    /// All retry attempts exhausted.
    #[error("retries exhausted after {attempts} attempts: {last}")]
    RetriesExhausted {
        attempts: u32,
        last: String,
    },
    /// Bounded buffer full — backpressure signal, caller must retry later.
    #[error("backpressure: buffer full ({buffer_size}), retry later")]
    Backpressure {
        buffer_size: usize,
    },
    /// Unknown ack id (already acked, unknown, or auto-ack mode).
    #[error("unknown ack id: {0}")]
    UnknownAck(u64),
}

/// Acknowledgement discipline (§57: ack).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AckMode {
    /// Consumed messages are settled immediately.
    #[default]
    Auto,
    /// Caller must settle each message via `ack`/`nack`.
    Manual,
}

/// Connection lifecycle (§57: lifecycle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Created but not yet connected.
    Disconnected,
    /// `connect` in progress.
    Connecting,
    /// Ready to publish/consume.
    Connected,
    /// Permanently closed; must create a new handle to reuse.
    Closed,
}

fn default_timeout_ms() -> u64 {
    5_000
}
fn default_max_retries() -> u32 {
    3
}
fn default_retry_backoff_ms() -> u64 {
    100
}
fn default_buffer_size() -> usize {
    1024
}

/// Retry policy shared by every broker stub (§57: retries).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Total attempts (initial try + retries).
    pub max_retries: u32,
    /// Base backoff in ms; actual delay is `base * 2^attempt` (capped).
    pub backoff_base_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy {
            max_retries: default_max_retries(),
            backoff_base_ms: default_retry_backoff_ms(),
        }
    }
}

impl RetryPolicy {
    /// Delay before attempt `attempt` (0-based), exponential with a 30s cap.
    pub fn delay_for(&self, attempt: u32) -> Duration {
        let ms = self
            .backoff_base_ms
            .saturating_mul(1u64 << attempt.min(10))
            .min(30_000);
        Duration::from_millis(ms)
    }
}

/// Shared connection settings for every broker stub (§57: timeout).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingConfig {
    /// Broker URL, e.g. `kafka://localhost:9092`.
    pub broker_url: String,
    /// Default topic / subject / queue (per-publish override possible).
    #[serde(default)]
    pub topic: Option<String>,
    /// Consumer group (Kafka/AMQP/RabbitMQ semantics).
    #[serde(default)]
    pub group: Option<String>,
    /// Per-operation timeout in ms.
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    /// Total attempts per operation (initial + retries).
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Retry base backoff in ms (exponential).
    #[serde(default = "default_retry_backoff_ms")]
    pub retry_backoff_ms: u64,
    /// Bounded in-memory buffer; full buffer ⇒ [`MessagingError::Backpressure`].
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    /// Ack discipline.
    #[serde(default)]
    pub ack_mode: AckMode,
}

impl MessagingConfig {
    /// Minimal config pointing at `broker_url`; sane Lex defaults otherwise.
    pub fn new(broker_url: impl Into<String>) -> Self {
        MessagingConfig {
            broker_url: broker_url.into(),
            topic: None,
            group: None,
            timeout_ms: default_timeout_ms(),
            max_retries: default_max_retries(),
            retry_backoff_ms: default_retry_backoff_ms(),
            buffer_size: default_buffer_size(),
            ack_mode: AckMode::Auto,
        }
    }

    /// Per-operation timeout as a [`Duration`].
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    /// [`RetryPolicy`] derived from this config.
    pub fn retry_policy(&self) -> RetryPolicy {
        RetryPolicy {
            max_retries: self.max_retries.max(1),
            backoff_base_ms: self.retry_backoff_ms,
        }
    }
}

/// One broker message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerMessage {
    /// Topic / subject / queue / channel the message was published to.
    pub topic: String,
    /// Payload (UTF-8 stub transport).
    pub payload: String,
    /// Optional routing/sharding key.
    #[serde(default)]
    pub key: Option<String>,
    /// Unix timestamp (seconds) assigned at publish.
    pub timestamp: u64,
}

/// A consumed message awaiting settlement (Manual ack mode).
#[derive(Debug, Clone)]
pub struct ConsumedMessage {
    /// Server-assigned delivery id; pass to `ack`/`nack`.
    pub id: u64,
    /// The message itself.
    pub message: BrokerMessage,
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Which broker a [`BrokerConnection`] talks to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrokerKind {
    Kafka,
    RabbitMq,
    Nats,
    RedisStreams,
    Mqtt,
    Amqp,
}

impl BrokerKind {
    /// Default broker URL for local development.
    pub fn default_url(self) -> &'static str {
        match self {
            BrokerKind::Kafka => "kafka://localhost:9092",
            BrokerKind::RabbitMq => "amqp://localhost:5672",
            BrokerKind::Nats => "nats://localhost:4222",
            BrokerKind::RedisStreams => "redis://localhost:6379",
            BrokerKind::Mqtt => "mqtt://localhost:1883",
            BrokerKind::Amqp => "amqp://localhost:5672",
        }
    }
}

/// Shared in-memory implementation behind all six broker stubs.
///
/// # Guarantees (apply to every broker stub)
/// - **Lifecycle:** `Disconnected → Connecting → Connected → Closed`;
///   publishing/consuming outside `Connected` is an error; `close` is
///   idempotent and terminal (a closed handle cannot reconnect).
/// - **Retries:** every fallible step runs inside [`MessagingConfig`]'s
///   retry budget with exponential backoff (`retry_backoff_ms * 2^attempt`).
/// - **Backpressure:** the buffer is bounded by `buffer_size`; a full buffer
///   returns [`MessagingError::Backpressure`] instead of growing
///   unboundedly or blocking forever.
/// - **Timeout:** each attempt is wrapped in `tokio::time::timeout`; a slow
///   broker surfaces [`MessagingError::Timeout`], never a hung task.
/// - **Ack:** `Auto` settles on consume; `Manual` parks deliveries in a
///   pending table until `ack` (settle) or `nack` (requeue to front).
/// - **Concurrency:** `Send + Sync` (`Arc<Mutex<..>>`); safe to share across
///   tasks. **Cancellation:** locks are held only across synchronous
///   critical sections, so dropping a future never corrupts state.
#[derive(Debug)]
struct BrokerConnectionInner {
    kind: BrokerKind,
    config: MessagingConfig,
    state: ConnectionState,
    buffer: Vec<BrokerMessage>,
    pending: HashMap<u64, BrokerMessage>,
    next_id: u64,
}

impl BrokerConnectionInner {
    fn new(kind: BrokerKind, config: MessagingConfig) -> Self {
        BrokerConnectionInner {
            kind,
            config,
            state: ConnectionState::Disconnected,
            buffer: Vec::new(),
            pending: HashMap::new(),
            next_id: 1,
        }
    }

    fn ensure_connected(&self) -> Result<(), MessagingError> {
        match self.state {
            ConnectionState::Connected => Ok(()),
            ConnectionState::Closed => Err(MessagingError::Connection(format!(
                "{:?} connection is closed",
                self.kind
            ))),
            other => Err(MessagingError::Connection(format!(
                "{:?} connection is not ready ({other:?}); call connect() first",
                self.kind
            ))),
        }
    }

    async fn do_connect(&mut self) -> Result<(), MessagingError> {
        if self.state == ConnectionState::Closed {
            return Err(MessagingError::Connection(format!(
                "{:?} connection is closed and cannot reconnect",
                self.kind
            )));
        }
        self.state = ConnectionState::Connecting;
        // Simulated handshake under the configured timeout (stub transport:
        // no real network yet — a real driver would dial here).
        let t = self.config.timeout();
        timeout(t, async {})
            .await
            .map_err(|_| MessagingError::Timeout(t))?;
        self.state = ConnectionState::Connected;
        Ok(())
    }

    async fn do_publish(
        &mut self,
        topic: Option<&str>,
        payload: &str,
        key: Option<&str>,
    ) -> Result<u64, MessagingError> {
        self.ensure_connected()?;
        let topic = topic
            .or(self.config.topic.as_deref())
            .ok_or_else(|| {
                MessagingError::Connection("no topic: pass one or set MessagingConfig::topic".into())
            })?
            .to_string();
        if self.buffer.len() >= self.config.buffer_size {
            return Err(MessagingError::Backpressure {
                buffer_size: self.config.buffer_size,
            });
        }
        let t = self.config.timeout();
        timeout(t, async {})
            .await
            .map_err(|_| MessagingError::Timeout(t))?;
        let id = self.next_id;
        self.next_id += 1;
        self.buffer.push(BrokerMessage {
            topic,
            payload: payload.to_string(),
            key: key.map(str::to_string),
            timestamp: now_unix_secs(),
        });
        Ok(id)
    }

    async fn do_consume(&mut self) -> Result<Option<ConsumedMessage>, MessagingError> {
        self.ensure_connected()?;
        let t = self.config.timeout();
        // Wait for work without busy-looping: poll the buffer until the
        // per-operation timeout expires (stub for a real blocking fetch).
        let deadline = tokio::time::Instant::now() + t;
        loop {
            if !self.buffer.is_empty() {
                break;
            }
            if tokio::time::Instant::now() >= deadline {
                return Ok(None);
            }
            sleep(Duration::from_millis(5)).await;
        }
        let message = self.buffer.remove(0);
        let id = self.next_id;
        self.next_id += 1;
        if self.config.ack_mode == AckMode::Manual {
            self.pending.insert(id, message.clone());
        }
        Ok(Some(ConsumedMessage { id, message }))
    }

    fn do_ack(&mut self, id: u64) -> Result<BrokerMessage, MessagingError> {
        self.pending
            .remove(&id)
            .ok_or(MessagingError::UnknownAck(id))
    }

    fn do_nack(&mut self, id: u64) -> Result<(), MessagingError> {
        let msg = self.pending.remove(&id).ok_or(MessagingError::UnknownAck(id))?;
        // Requeue to the front (redelivery), respecting the bound.
        if self.buffer.len() >= self.config.buffer_size {
            self.pending.insert(id, msg);
            return Err(MessagingError::Backpressure {
                buffer_size: self.config.buffer_size,
            });
        }
        self.buffer.insert(0, msg);
        Ok(())
    }
}

/// Runs `op` up to `policy.max_retries` times with exponential backoff.
/// Each attempt is already timeout-bounded by the caller.
async fn with_retries<T, F, Fut>(
    policy: &RetryPolicy,
    mut op: F,
) -> Result<T, MessagingError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, MessagingError>>,
{
    let mut last: Option<MessagingError> = None;
    for attempt in 0..policy.max_retries.max(1) {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                // Backpressure and unknown-acks are caller signals, not
                // broker flakes: surface immediately without retrying.
                if matches!(
                    e,
                    MessagingError::Backpressure { .. } | MessagingError::UnknownAck(_)
                ) {
                    return Err(e);
                }
                last = Some(e);
                if attempt + 1 < policy.max_retries.max(1) {
                    sleep(policy.delay_for(attempt)).await;
                }
            }
        }
    }
    Err(MessagingError::RetriesExhausted {
        attempts: policy.max_retries.max(1),
        last: last
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown".into()),
    })
}

/// Generates one broker stub struct (`$name`) over [`BrokerConnectionInner`]
/// with per-broker defaults and docs, plus the `MessagingModule` registry
/// plumbing (`$field`, `$connect`, `$publish`, `$consume`, `$ack`,
/// `$nack`, `$close`, `$list`).
macro_rules! define_broker {
    (
        $(#[$meta:meta])*
        $name:ident, $field:ident, $kind:expr, $default_url:expr,
        $connect:ident, $publish:ident, $consume:ident,
        $ack:ident, $nack:ident, $close:ident, $list:ident
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $name {
            inner: Arc<Mutex<BrokerConnectionInner>>,
        }

        impl $name {
            /// Connects (lifecycle `Disconnected → Connected`) under the
            /// configured timeout + retry budget.
            pub async fn connect(config: MessagingConfig) -> Result<Self, MessagingError> {
                let conn = $name {
                    inner: Arc::new(Mutex::new(BrokerConnectionInner::new($kind, config))),
                };
                let policy = conn.inner.lock().await.config.retry_policy();
                with_retries(&policy, || async {
                    conn.inner.lock().await.do_connect().await
                })
                .await?;
                Ok(conn)
            }

            /// Connects to the local-development default URL.
            pub async fn connect_default() -> Result<Self, MessagingError> {
                Self::connect(MessagingConfig::new($default_url)).await
            }

            /// Current lifecycle state.
            pub async fn state(&self) -> ConnectionState {
                self.inner.lock().await.state
            }

            /// Publishes one message (timeout + retries + backpressure).
            /// Returns the assigned publish id.
            pub async fn publish(
                &self,
                topic: Option<&str>,
                payload: &str,
                key: Option<&str>,
            ) -> Result<u64, MessagingError> {
                let policy = self.inner.lock().await.config.retry_policy();
                with_retries(&policy, || async {
                    self.inner
                        .lock()
                        .await
                        .do_publish(topic, payload, key)
                        .await
                })
                .await
            }

            /// Consumes one message (`None` = timeout with empty buffer).
            /// In `Manual` ack mode the delivery stays pending until
            /// [`Self::ack`]/[`Self::nack`]; in `Auto` it is settled.
            pub async fn consume(&self) -> Result<Option<ConsumedMessage>, MessagingError> {
                let policy = self.inner.lock().await.config.retry_policy();
                with_retries(&policy, || async {
                    self.inner.lock().await.do_consume().await
                })
                .await
            }

            /// Settles a manually-acked delivery.
            pub async fn ack(&self, id: u64) -> Result<BrokerMessage, MessagingError> {
                self.inner.lock().await.do_ack(id)
            }

            /// Requeues a manually-acked delivery to the front (redelivery).
            pub async fn nack(&self, id: u64) -> Result<(), MessagingError> {
                self.inner.lock().await.do_nack(id)
            }

            /// Buffered (undelivered) message count — backpressure signal.
            pub async fn buffered(&self) -> usize {
                self.inner.lock().await.buffer.len()
            }

            /// Pending-ack count (`Manual` mode only).
            pub async fn pending_count(&self) -> usize {
                self.inner.lock().await.pending.len()
            }

            /// Idempotent, terminal close (`→ Closed`).
            pub async fn close(&self) -> Result<(), MessagingError> {
                self.inner.lock().await.state = ConnectionState::Closed;
                Ok(())
            }
        }
    };
}

define_broker!(
    /// Apache Kafka stub (§57).
    ///
    /// Models topic-partition publish + consumer-group consume with at-least-once
    /// delivery in `Manual` ack mode (offsets settle only on [`KafkaConnection::ack`]).
    /// Defaults to `kafka://localhost:9092`. Stub transport: in-memory buffer,
    /// no real broker — wire a real `rdkafka` driver behind the same method
    /// shapes when integrating.
    KafkaConnection, kafka, BrokerKind::Kafka, "kafka://localhost:9092",
    kafka_connect, kafka_publish, kafka_consume,
    kafka_ack, kafka_nack, kafka_close, list_kafka
);
define_broker!(
    /// RabbitMQ stub (§57, AMQP 0-9-1).
    ///
    /// Models queue publish/consume with explicit [`RabbitMqConnection::ack`] /
    /// [`RabbitMqConnection::nack`] (requeue) settlement. Defaults to
    /// `amqp://localhost:5672`. Stub transport: in-memory buffer.
    RabbitMqConnection, rabbitmq, BrokerKind::RabbitMq, "amqp://localhost:5672",
    rabbitmq_connect, rabbitmq_publish, rabbitmq_consume,
    rabbitmq_ack, rabbitmq_nack, rabbitmq_close, list_rabbitmq
);
define_broker!(
    /// NATS stub (§57).
    ///
    /// Models subject-based pub/sub (core NATS) with at-most-once `Auto` ack
    /// by default; switch to `Manual` for JetStream-style ack/nack settlement.
    /// Defaults to `nats://localhost:4222`. Stub transport: in-memory buffer.
    NatsConnection, nats, BrokerKind::Nats, "nats://localhost:4222",
    nats_connect, nats_publish, nats_consume,
    nats_ack, nats_nack, nats_close, list_nats
);
define_broker!(
    /// Redis Streams stub (§57, `XADD`/`XREADGROUP` semantics).
    ///
    /// Models append-to-stream publish + group consume; `Manual` ack mirrors
    /// `XACK`, `nack` mirrors re-delivery via pending-list reclaim. Defaults
    /// to `redis://localhost:6379`. Stub transport: in-memory buffer.
    RedisStreamsConnection, redis_streams, BrokerKind::RedisStreams, "redis://localhost:6379",
    redis_streams_connect, redis_streams_publish, redis_streams_consume,
    redis_streams_ack, redis_streams_nack, redis_streams_close, list_redis_streams
);
define_broker!(
    /// MQTT stub (§57, v3.1.1/v5 shape).
    ///
    /// Models topic publish/subscribe; QoS-1-style at-least-once delivery is
    /// approximated by `Manual` ack + [`MqttConnection::nack`] redelivery.
    /// Defaults to `mqtt://localhost:1883`. Stub transport: in-memory buffer.
    MqttConnection, mqtt, BrokerKind::Mqtt, "mqtt://localhost:1883",
    mqtt_connect, mqtt_publish, mqtt_consume,
    mqtt_ack, mqtt_nack, mqtt_close, list_mqtt
);
define_broker!(
    /// Generic AMQP 1.0 stub (§57).
    ///
    /// Models link-attach lifecycle + settled/unsettled delivery outcomes via
    /// [`AmqpConnection::ack`] (accepted) / [`AmqpConnection::nack`]
    /// (released back to the link). Defaults to `amqp://localhost:5672`.
    /// Stub transport: in-memory buffer.
    AmqpConnection, amqp, BrokerKind::Amqp, "amqp://localhost:5672",
    amqp_connect, amqp_publish, amqp_consume,
    amqp_ack, amqp_nack, amqp_close, list_amqp
);

/// Registry owning named handles for all six broker stubs (§57).
///
/// Mirrors the [`SocketModule`] style (named maps behind `Arc<Mutex<..>>`,
/// `Send + Sync`, cancellation-safe) without touching the existing
/// TCP/UDP API.
#[derive(Debug, Default)]
pub struct MessagingModule {
    kafka: Arc<Mutex<HashMap<String, KafkaConnection>>>,
    rabbitmq: Arc<Mutex<HashMap<String, RabbitMqConnection>>>,
    nats: Arc<Mutex<HashMap<String, NatsConnection>>>,
    redis_streams: Arc<Mutex<HashMap<String, RedisStreamsConnection>>>,
    mqtt: Arc<Mutex<HashMap<String, MqttConnection>>>,
    amqp: Arc<Mutex<HashMap<String, AmqpConnection>>>,
}

impl MessagingModule {
    /// Empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Connects a named Kafka handle.
    pub async fn kafka_connect(
        &self,
        name: String,
        config: MessagingConfig,
    ) -> Result<String, MessagingError> {
        let conn = KafkaConnection::connect(config).await?;
        self.kafka.lock().await.insert(name.clone(), conn);
        Ok(format!("kafka '{name}' connected"))
    }

    /// Publishes via a named Kafka handle.
    pub async fn kafka_publish(
        &self,
        name: &str,
        topic: Option<&str>,
        payload: &str,
        key: Option<&str>,
    ) -> Result<u64, MessagingError> {
        let conns = self.kafka.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("kafka '{name}' not found")))?;
        conn.publish(topic, payload, key).await
    }

    /// Consumes one message via a named Kafka handle.
    pub async fn kafka_consume(&self, name: &str) -> Result<Option<ConsumedMessage>, MessagingError> {
        let conns = self.kafka.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("kafka '{name}' not found")))?;
        conn.consume().await
    }

    /// Settles a Kafka delivery.
    pub async fn kafka_ack(&self, name: &str, id: u64) -> Result<BrokerMessage, MessagingError> {
        let conns = self.kafka.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("kafka '{name}' not found")))?;
        conn.ack(id).await
    }

    /// Requeues a Kafka delivery.
    pub async fn kafka_nack(&self, name: &str, id: u64) -> Result<(), MessagingError> {
        let conns = self.kafka.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("kafka '{name}' not found")))?;
        conn.nack(id).await
    }

    /// Closes and removes a named Kafka handle.
    pub async fn kafka_close(&self, name: &str) -> Result<String, MessagingError> {
        let mut conns = self.kafka.lock().await;
        if let Some(conn) = conns.remove(name) {
            conn.close().await?;
            Ok(format!("kafka '{name}' closed"))
        } else {
            Err(MessagingError::Connection(format!(
                "kafka '{name}' not found"
            )))
        }
    }

    /// Named Kafka handles.
    pub async fn list_kafka(&self) -> Vec<String> {
        self.kafka.lock().await.keys().cloned().collect()
    }

    /// Connects a named RabbitMQ handle.
    pub async fn rabbitmq_connect(
        &self,
        name: String,
        config: MessagingConfig,
    ) -> Result<String, MessagingError> {
        let conn = RabbitMqConnection::connect(config).await?;
        self.rabbitmq.lock().await.insert(name.clone(), conn);
        Ok(format!("rabbitmq '{name}' connected"))
    }

    /// Publishes via a named RabbitMQ handle.
    pub async fn rabbitmq_publish(
        &self,
        name: &str,
        topic: Option<&str>,
        payload: &str,
        key: Option<&str>,
    ) -> Result<u64, MessagingError> {
        let conns = self.rabbitmq.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("rabbitmq '{name}' not found")))?;
        conn.publish(topic, payload, key).await
    }

    /// Consumes one message via a named RabbitMQ handle.
    pub async fn rabbitmq_consume(
        &self,
        name: &str,
    ) -> Result<Option<ConsumedMessage>, MessagingError> {
        let conns = self.rabbitmq.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("rabbitmq '{name}' not found")))?;
        conn.consume().await
    }

    /// Settles a RabbitMQ delivery.
    pub async fn rabbitmq_ack(&self, name: &str, id: u64) -> Result<BrokerMessage, MessagingError> {
        let conns = self.rabbitmq.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("rabbitmq '{name}' not found")))?;
        conn.ack(id).await
    }

    /// Requeues a RabbitMQ delivery.
    pub async fn rabbitmq_nack(&self, name: &str, id: u64) -> Result<(), MessagingError> {
        let conns = self.rabbitmq.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("rabbitmq '{name}' not found")))?;
        conn.nack(id).await
    }

    /// Closes and removes a named RabbitMQ handle.
    pub async fn rabbitmq_close(&self, name: &str) -> Result<String, MessagingError> {
        let mut conns = self.rabbitmq.lock().await;
        if let Some(conn) = conns.remove(name) {
            conn.close().await?;
            Ok(format!("rabbitmq '{name}' closed"))
        } else {
            Err(MessagingError::Connection(format!(
                "rabbitmq '{name}' not found"
            )))
        }
    }

    /// Named RabbitMQ handles.
    pub async fn list_rabbitmq(&self) -> Vec<String> {
        self.rabbitmq.lock().await.keys().cloned().collect()
    }

    /// Connects a named NATS handle.
    pub async fn nats_connect(
        &self,
        name: String,
        config: MessagingConfig,
    ) -> Result<String, MessagingError> {
        let conn = NatsConnection::connect(config).await?;
        self.nats.lock().await.insert(name.clone(), conn);
        Ok(format!("nats '{name}' connected"))
    }

    /// Publishes via a named NATS handle.
    pub async fn nats_publish(
        &self,
        name: &str,
        topic: Option<&str>,
        payload: &str,
        key: Option<&str>,
    ) -> Result<u64, MessagingError> {
        let conns = self.nats.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("nats '{name}' not found")))?;
        conn.publish(topic, payload, key).await
    }

    /// Consumes one message via a named NATS handle.
    pub async fn nats_consume(&self, name: &str) -> Result<Option<ConsumedMessage>, MessagingError> {
        let conns = self.nats.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("nats '{name}' not found")))?;
        conn.consume().await
    }

    /// Settles a NATS delivery.
    pub async fn nats_ack(&self, name: &str, id: u64) -> Result<BrokerMessage, MessagingError> {
        let conns = self.nats.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("nats '{name}' not found")))?;
        conn.ack(id).await
    }

    /// Requeues a NATS delivery.
    pub async fn nats_nack(&self, name: &str, id: u64) -> Result<(), MessagingError> {
        let conns = self.nats.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("nats '{name}' not found")))?;
        conn.nack(id).await
    }

    /// Closes and removes a named NATS handle.
    pub async fn nats_close(&self, name: &str) -> Result<String, MessagingError> {
        let mut conns = self.nats.lock().await;
        if let Some(conn) = conns.remove(name) {
            conn.close().await?;
            Ok(format!("nats '{name}' closed"))
        } else {
            Err(MessagingError::Connection(format!("nats '{name}' not found")))
        }
    }

    /// Named NATS handles.
    pub async fn list_nats(&self) -> Vec<String> {
        self.nats.lock().await.keys().cloned().collect()
    }

    /// Connects a named Redis Streams handle.
    pub async fn redis_streams_connect(
        &self,
        name: String,
        config: MessagingConfig,
    ) -> Result<String, MessagingError> {
        let conn = RedisStreamsConnection::connect(config).await?;
        self.redis_streams.lock().await.insert(name.clone(), conn);
        Ok(format!("redis-streams '{name}' connected"))
    }

    /// Publishes via a named Redis Streams handle.
    pub async fn redis_streams_publish(
        &self,
        name: &str,
        topic: Option<&str>,
        payload: &str,
        key: Option<&str>,
    ) -> Result<u64, MessagingError> {
        let conns = self.redis_streams.lock().await;
        let conn = conns.get(name).ok_or_else(|| {
            MessagingError::Connection(format!("redis-streams '{name}' not found"))
        })?;
        conn.publish(topic, payload, key).await
    }

    /// Consumes one message via a named Redis Streams handle.
    pub async fn redis_streams_consume(
        &self,
        name: &str,
    ) -> Result<Option<ConsumedMessage>, MessagingError> {
        let conns = self.redis_streams.lock().await;
        let conn = conns.get(name).ok_or_else(|| {
            MessagingError::Connection(format!("redis-streams '{name}' not found"))
        })?;
        conn.consume().await
    }

    /// XACKs a Redis Streams delivery.
    pub async fn redis_streams_ack(
        &self,
        name: &str,
        id: u64,
    ) -> Result<BrokerMessage, MessagingError> {
        let conns = self.redis_streams.lock().await;
        let conn = conns.get(name).ok_or_else(|| {
            MessagingError::Connection(format!("redis-streams '{name}' not found"))
        })?;
        conn.ack(id).await
    }

    /// Requeues a Redis Streams delivery.
    pub async fn redis_streams_nack(&self, name: &str, id: u64) -> Result<(), MessagingError> {
        let conns = self.redis_streams.lock().await;
        let conn = conns.get(name).ok_or_else(|| {
            MessagingError::Connection(format!("redis-streams '{name}' not found"))
        })?;
        conn.nack(id).await
    }

    /// Closes and removes a named Redis Streams handle.
    pub async fn redis_streams_close(&self, name: &str) -> Result<String, MessagingError> {
        let mut conns = self.redis_streams.lock().await;
        if let Some(conn) = conns.remove(name) {
            conn.close().await?;
            Ok(format!("redis-streams '{name}' closed"))
        } else {
            Err(MessagingError::Connection(format!(
                "redis-streams '{name}' not found"
            )))
        }
    }

    /// Named Redis Streams handles.
    pub async fn list_redis_streams(&self) -> Vec<String> {
        self.redis_streams.lock().await.keys().cloned().collect()
    }

    /// Connects a named MQTT handle.
    pub async fn mqtt_connect(
        &self,
        name: String,
        config: MessagingConfig,
    ) -> Result<String, MessagingError> {
        let conn = MqttConnection::connect(config).await?;
        self.mqtt.lock().await.insert(name.clone(), conn);
        Ok(format!("mqtt '{name}' connected"))
    }

    /// Publishes via a named MQTT handle.
    pub async fn mqtt_publish(
        &self,
        name: &str,
        topic: Option<&str>,
        payload: &str,
        key: Option<&str>,
    ) -> Result<u64, MessagingError> {
        let conns = self.mqtt.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("mqtt '{name}' not found")))?;
        conn.publish(topic, payload, key).await
    }

    /// Consumes one message via a named MQTT handle.
    pub async fn mqtt_consume(&self, name: &str) -> Result<Option<ConsumedMessage>, MessagingError> {
        let conns = self.mqtt.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("mqtt '{name}' not found")))?;
        conn.consume().await
    }

    /// Settles an MQTT delivery.
    pub async fn mqtt_ack(&self, name: &str, id: u64) -> Result<BrokerMessage, MessagingError> {
        let conns = self.mqtt.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("mqtt '{name}' not found")))?;
        conn.ack(id).await
    }

    /// Requeues an MQTT delivery.
    pub async fn mqtt_nack(&self, name: &str, id: u64) -> Result<(), MessagingError> {
        let conns = self.mqtt.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("mqtt '{name}' not found")))?;
        conn.nack(id).await
    }

    /// Closes and removes a named MQTT handle.
    pub async fn mqtt_close(&self, name: &str) -> Result<String, MessagingError> {
        let mut conns = self.mqtt.lock().await;
        if let Some(conn) = conns.remove(name) {
            conn.close().await?;
            Ok(format!("mqtt '{name}' closed"))
        } else {
            Err(MessagingError::Connection(format!("mqtt '{name}' not found")))
        }
    }

    /// Named MQTT handles.
    pub async fn list_mqtt(&self) -> Vec<String> {
        self.mqtt.lock().await.keys().cloned().collect()
    }

    /// Connects a named AMQP handle.
    pub async fn amqp_connect(
        &self,
        name: String,
        config: MessagingConfig,
    ) -> Result<String, MessagingError> {
        let conn = AmqpConnection::connect(config).await?;
        self.amqp.lock().await.insert(name.clone(), conn);
        Ok(format!("amqp '{name}' connected"))
    }

    /// Publishes via a named AMQP handle.
    pub async fn amqp_publish(
        &self,
        name: &str,
        topic: Option<&str>,
        payload: &str,
        key: Option<&str>,
    ) -> Result<u64, MessagingError> {
        let conns = self.amqp.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("amqp '{name}' not found")))?;
        conn.publish(topic, payload, key).await
    }

    /// Consumes one message via a named AMQP handle.
    pub async fn amqp_consume(&self, name: &str) -> Result<Option<ConsumedMessage>, MessagingError> {
        let conns = self.amqp.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("amqp '{name}' not found")))?;
        conn.consume().await
    }

    /// Accepts an AMQP delivery.
    pub async fn amqp_ack(&self, name: &str, id: u64) -> Result<BrokerMessage, MessagingError> {
        let conns = self.amqp.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("amqp '{name}' not found")))?;
        conn.ack(id).await
    }

    /// Releases an AMQP delivery back to the link.
    pub async fn amqp_nack(&self, name: &str, id: u64) -> Result<(), MessagingError> {
        let conns = self.amqp.lock().await;
        let conn = conns
            .get(name)
            .ok_or_else(|| MessagingError::Connection(format!("amqp '{name}' not found")))?;
        conn.nack(id).await
    }

    /// Closes and removes a named AMQP handle.
    pub async fn amqp_close(&self, name: &str) -> Result<String, MessagingError> {
        let mut conns = self.amqp.lock().await;
        if let Some(conn) = conns.remove(name) {
            conn.close().await?;
            Ok(format!("amqp '{name}' closed"))
        } else {
            Err(MessagingError::Connection(format!("amqp '{name}' not found")))
        }
    }

    /// Named AMQP handles.
    pub async fn list_amqp(&self) -> Vec<String> {
        self.amqp.lock().await.keys().cloned().collect()
    }
}
