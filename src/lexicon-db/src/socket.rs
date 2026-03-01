use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::Mutex;

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
        })
    }

    pub async fn accept(&mut self) -> Result<TcpStream, String> {
        let listener = self.listener.as_mut()
            .ok_or_else(|| "Server not listening".to_string())?;
        
        let (stream, addr) = listener.accept()
            .await
            .map_err(|e| format!("Accept failed: {}", e))?;
        
        Ok(stream)
    }

    pub fn config(&self) -> &SocketConfig {
        &self.config
    }
}

pub struct TcpClient {
    stream: Option<TcpStream>,
    config: SocketConfig,
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
        })
    }

    pub async fn send(&mut self, data: &str) -> Result<usize, String> {
        let stream = self.stream.as_mut()
            .ok_or_else(|| "Not connected".to_string())?;
        
        stream.write_all(data.as_bytes())
            .await
            .map_err(|e| format!("Send failed: {}", e))?;
        
        Ok(data.len())
    }

    pub async fn receive(&mut self, buffer_size: usize) -> Result<String, String> {
        let stream = self.stream.as_mut()
            .ok_or_else(|| "Not connected".to_string())?;
        
        let mut buffer = vec![0u8; buffer_size];
        let n = stream.read(&mut buffer)
            .await
            .map_err(|e| format!("Receive failed: {}", e))?;
        
        String::from_utf8(buffer[..n].to_vec())
            .map_err(|e| format!("Invalid UTF-8: {}", e))
    }

    pub async fn close(&mut self) -> Result<(), String> {
        if let Some(mut stream) = self.stream.take() {
            stream.shutdown()
                .await
                .map_err(|e| format!("Close failed: {}", e))?;
        }
        Ok(())
    }
}

pub struct UdpSocketConnection {
    socket: UdpSocket,
    config: SocketConfig,
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
        })
    }

    pub async fn send_to(&self, data: &str, target: &str) -> Result<usize, String> {
        let n = self.socket.send_to(data.as_bytes(), target)
            .await
            .map_err(|e| format!("Send failed: {}", e))?;
        Ok(n)
    }

    pub async fn receive_from(&self, buffer_size: usize) -> Result<(String, String), String> {
        let mut buffer = vec![0u8; buffer_size];
        let (n, addr) = self.socket.recv_from(&mut buffer)
            .await
            .map_err(|e| format!("Receive failed: {}", e))?;
        
        let data = String::from_utf8(buffer[..n].to_vec())
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;
        
        Ok((data, addr.to_string()))
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
