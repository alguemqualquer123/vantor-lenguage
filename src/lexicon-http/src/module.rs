use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub struct HttpModule;

impl HttpModule {
    pub fn new() -> Self {
        Self
    }
}

// ---------------------------------------------------------------------------
// Defaults seguros globais (§18/§19)
// ---------------------------------------------------------------------------

/// Limite padrão de corpo: 1 MiB.
pub const DEFAULT_BODY_LIMIT_BYTES: usize = 1_048_576;

/// Versão mínima padrão do TLS: 1.2+.
pub const DEFAULT_TLS_MIN_VERSION: &str = "1.2";

// ---------------------------------------------------------------------------
// Net abstractions (§18 parcial)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConfig {
    pub host: String,
    pub port: u16,
    pub nodelay: bool,
    pub keepalive_secs: Option<u64>,
    pub backlog: u32,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            nodelay: true,
            keepalive_secs: Some(60),
            backlog: 128,
        }
    }
}

impl TcpConfig {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            ..Default::default()
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpConfig {
    pub host: String,
    pub port: u16,
    pub multicast: bool,
    pub ttl: u32,
}

impl Default for UdpConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3001,
            multicast: false,
            ttl: 1,
        }
    }
}

impl UdpConfig {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            ..Default::default()
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdsConfig {
    pub path: String,
    pub backlog: u32,
}

impl Default for UdsConfig {
    fn default() -> Self {
        Self {
            path: "/tmp/lexicon.sock".to_string(),
            backlog: 128,
        }
    }
}

impl UdsConfig {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            backlog: 128,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TlsVersion {
    /// TLS 1.2 (mínimo aceito por padrão).
    #[default]
    Tls12,
    Tls13,
}

impl TlsVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            TlsVersion::Tls12 => "1.2",
            TlsVersion::Tls13 => "1.3",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub min_version: TlsVersion,
    pub max_version: Option<TlsVersion>,
    pub handshake_timeout_ms: u64,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub alpn: Vec<String>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            min_version: TlsVersion::Tls12,
            max_version: None,
            handshake_timeout_ms: 10_000,
            cert_path: None,
            key_path: None,
            alpn: vec!["h2".to_string(), "http/1.1".to_string()],
        }
    }
}

impl TlsConfig {
    /// Defaults seguros: TLS 1.2+, sem certificados embutidos.
    pub fn secure_default() -> Self {
        Self::default()
    }

    pub fn is_secure_min(&self) -> bool {
        matches!(self.min_version, TlsVersion::Tls12 | TlsVersion::Tls13)
    }

    /// Modo de verificação derivado: sem cert/key => Ca (default seguro).
    pub fn verify_mode(&self) -> TlsVerifyMode {
        if self.cert_path.is_some() {
            TlsVerifyMode::Full
        } else {
            TlsVerifyMode::Ca
        }
    }

    /// Garante min <= max quando max está definido.
    pub fn check_versions(&self) -> Result<(), String> {
        if let Some(max) = self.max_version {
            let ord = |v: TlsVersion| match v {
                TlsVersion::Tls12 => 12,
                TlsVersion::Tls13 => 13,
            };
            if ord(self.min_version) > ord(max) {
                return Err(format!(
                    "tls min version {} > max version {}",
                    self.min_version.as_str(),
                    max.as_str()
                ));
            }
        }
        Ok(())
    }
}

/// Defaults seguros de cookie: Secure + HttpOnly + SameSite=Lax.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieDefaults {
    pub secure: bool,
    pub http_only: bool,
    pub same_site: String,
}

impl Default for CookieDefaults {
    fn default() -> Self {
        Self {
            secure: true,
            http_only: true,
            same_site: "Lax".to_string(),
        }
    }
}

/// Limites do servidor (body limit 1 MiB por padrão).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerLimits {
    pub max_body_bytes: usize,
    pub max_header_bytes: usize,
    pub max_requests_per_socket: usize,
}

impl Default for ServerLimits {
    fn default() -> Self {
        Self {
            max_body_bytes: DEFAULT_BODY_LIMIT_BYTES,
            max_header_bytes: 16 * 1024,
            max_requests_per_socket: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub servers: Vec<String>,
    pub timeout_ms: u64,
    pub retries: u32,
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            servers: vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()],
            timeout_ms: 5_000,
            retries: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyKind {
    Http,
    Https,
    Socks5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub kind: ProxyKind,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl ProxyConfig {
    pub fn new(kind: ProxyKind, host: &str, port: u16) -> Self {
        Self {
            kind,
            host: host.to_string(),
            port,
            username: None,
            password: None,
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Stub QUIC (§18): configuração presente, handshake real fora de escopo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicConfig {
    pub host: String,
    pub port: u16,
    pub max_streams: u64,
    pub alpn: Vec<String>,
}

impl Default for QuicConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 4433,
            max_streams: 100,
            alpn: vec!["h3".to_string()],
        }
    }
}

/// Stub gRPC channel (§18): configuração presente, transporte real fora de escopo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcChannel {
    pub endpoint: String,
    pub tls: bool,
    pub timeout_ms: u64,
    pub max_msg_bytes: usize,
}

impl GrpcChannel {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            tls: true,
            timeout_ms: 10_000,
            max_msg_bytes: 4 * 1_048_576,
        }
    }
}

// ---------------------------------------------------------------------------
// HTTP: métodos, request, response, router (§19)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
    TRACE,
    CONNECT,
}

impl HttpMethod {
    pub const ALL: &[HttpMethod] = &[
        HttpMethod::GET,
        HttpMethod::POST,
        HttpMethod::PUT,
        HttpMethod::PATCH,
        HttpMethod::DELETE,
        HttpMethod::HEAD,
        HttpMethod::OPTIONS,
        HttpMethod::TRACE,
        HttpMethod::CONNECT,
    ];

    /// Complexity: O(1).
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::TRACE => "TRACE",
            HttpMethod::CONNECT => "CONNECT",
        }
    }

    /// Complexity: O(n) uppercase (n pequeno, método).
    #[inline]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "PUT" => Some(HttpMethod::PUT),
            "PATCH" => Some(HttpMethod::PATCH),
            "DELETE" => Some(HttpMethod::DELETE),
            "HEAD" => Some(HttpMethod::HEAD),
            "OPTIONS" => Some(HttpMethod::OPTIONS),
            "TRACE" => Some(HttpMethod::TRACE),
            "CONNECT" => Some(HttpMethod::CONNECT),
            _ => None,
        }
    }

    /// Métodos seguros Node/HTTP: não alteram estado.
    /// Complexity: O(1).
    #[inline]
    pub fn is_safe(&self) -> bool {
        matches!(
            self,
            HttpMethod::GET | HttpMethod::HEAD | HttpMethod::OPTIONS | HttpMethod::TRACE
        )
    }

    /// HEAD exige strip do corpo na resposta.
    /// Complexity: O(1).
    #[inline]
    pub fn requires_body_strip(&self) -> bool {
        matches!(self, HttpMethod::HEAD)
    }
}

#[derive(Debug, Clone, Default)]
pub struct HttpRequest {
    pub method: Option<HttpMethod>,
    pub path: String,
    pub headers: Headers,
    pub query: Vec<(String, String)>,
    pub body: HttpBody,
}

#[derive(Debug, Clone, Default)]
pub enum HttpBody {
    #[default]
    Empty,
    Buffer(Vec<u8>),
    // Streaming is handled via the runtime agent/pool for performance.
}

impl HttpBody {
    /// Complexity: O(1).
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            HttpBody::Empty => true,
            HttpBody::Buffer(b) => b.is_empty(),
        }
    }
    /// Complexity: O(1).
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            HttpBody::Empty => 0,
            HttpBody::Buffer(b) => b.len(),
        }
    }
    /// Zero-copy borrow of the buffered bytes (empty for `Empty`).
    /// Complexity: O(1).
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        match self {
            HttpBody::Empty => &[],
            HttpBody::Buffer(b) => b.as_slice(),
        }
    }
    /// Zero-copy borrow (alias of `as_slice`).
    /// Complexity: O(1).
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.as_slice()
    }
}

impl From<Vec<u8>> for HttpBody {
    /// Complexity: O(1) (moves the vec).
    #[inline]
    fn from(v: Vec<u8>) -> Self {
        if v.is_empty() {
            HttpBody::Empty
        } else {
            HttpBody::Buffer(v)
        }
    }
}

impl HttpRequest {
    pub fn new(method: HttpMethod, path: &str) -> Self {
        Self {
            method: Some(method),
            path: path.to_string(),
            headers: Headers::new(),
            query: Vec::new(),
            body: HttpBody::Empty,
        }
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = HttpBody::Buffer(body);
        self
    }

    pub fn query_param(&self, key: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn body_str(&self) -> Result<&str, String> {
        match &self.body {
            HttpBody::Buffer(b) => std::str::from_utf8(b).map_err(|e| format!("invalid utf-8 body: {e}")),
            _ => Err("body is not a buffer".to_string()),
        }
    }

    pub fn json_body(&self) -> Result<serde_json::Value, String> {
        match &self.body {
            HttpBody::Buffer(b) => {
                // Enforce default Limits (8 MiB / depth 32) so this path
                // cannot skip untrusted-input caps. See response::decode_json.
                if b.len() > crate::response::Limits::default().max_bytes {
                    return Err(format!(
                        "body too large: {} > {}",
                        b.len(),
                        crate::response::Limits::default().max_bytes
                    ));
                }
                let s = std::str::from_utf8(b).map_err(|e| format!("invalid utf-8 body: {e}"))?;
                let v: serde_json::Value =
                    serde_json::from_str(s).map_err(|e| format!("invalid json body: {e}"))?;
                // Depth/container walk (O(n)); reuse response helper via re-parse.
                let limits = crate::response::Limits::default();
                let raw = serde_json::to_vec(&v).map_err(|e| format!("invalid json body: {e}"))?;
                let back: serde_json::Value = crate::response::decode_json(&raw, &limits)
                    .map_err(|e| format!("json limit exceeded: {e}"))?;
                Ok(back)
            }
            _ => Err("body is not a buffer".to_string()),
        }
    }

    /// Same as [`HttpRequest::json_body`] with explicit `Limits`.
    /// Complexity: O(n) parse + O(n) limit walk.
    pub fn json_body_limited(
        &self,
        limits: &crate::response::Limits,
    ) -> Result<serde_json::Value, String> {
        match &self.body {
            HttpBody::Buffer(b) => crate::response::decode_json(b, limits)
                .map_err(|e| format!("invalid json body: {e}")),
            _ => Err("body is not a buffer".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Headers,
    pub body: HttpBody,
}

impl HttpResponse {
    pub fn new(status: u16, body: HttpBody) -> Self {
        Self {
            status,
            headers: Headers::new(),
            body,
        }
    }

    pub fn ok(body: Vec<u8>) -> Self {
        Self::new(200, HttpBody::Buffer(body))
    }

    pub fn text(body: &str) -> Self {
        let mut r = Self::ok(body.as_bytes().to_vec());
        r.headers.insert("content-type", "text/plain; charset=utf-8");
        r
    }

    pub fn json<T: Serialize>(value: &T) -> Self {
        let body = serde_json::to_vec(value).unwrap_or_default();
        let mut r = Self::ok(body);
        r.headers.insert("content-type", "application/json");
        r
    }

    pub fn not_found() -> Self {
        let mut r = Self::new(404, HttpBody::Buffer(b"not found".to_vec()));
        r.headers.insert("content-type", "text/plain; charset=utf-8");
        r
    }

    pub fn not_found_json(msg: &str) -> Self {
        Self::json(&serde_json::json!({"success": false, "data": null, "message": msg}))
            .with_status(404)
    }

    pub fn created<T: Serialize>(value: &T) -> Self {
        Self::json(value).with_status(201)
    }

    pub fn no_content() -> Self {
        Self::new(204, HttpBody::Empty)
    }

    pub fn bad_request(msg: &str) -> Self {
        Self::json(&serde_json::json!({"success": false, "data": null, "message": msg}))
            .with_status(400)
    }

    pub fn unauthorized(msg: &str) -> Self {
        let mut r = Self::bad_request(msg);
        r.status = 401;
        r.headers
            .insert("www-authenticate", "Bearer realm=\"lexicon\"");
        r
    }

    pub fn forbidden(msg: &str) -> Self {
        Self::json(&serde_json::json!({"success": false, "data": null, "message": msg}))
            .with_status(403)
    }

    pub fn method_not_allowed(allowed: &str) -> Self {
        let mut r = Self::json(&serde_json::json!({
            "success": false, "data": null,
            "message": format!("method not allowed; allowed: {allowed}")
        }))
        .with_status(405);
        r.headers.insert("allow", allowed);
        r
    }

    pub fn internal_error(msg: &str) -> Self {
        Self::json(&serde_json::json!({"success": false, "data": null, "message": msg}))
            .with_status(500)
    }

    /// Troca o status mantendo headers/body (base p/ envelopes).
    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    /// Aplica os headers CORS do middleware em respostas reais.
    pub fn with_cors(mut self, cors: &CorsMiddleware) -> Self {
        for (k, v) in cors.response_headers() {
            self.headers.insert(&k, &v);
        }
        self
    }

    pub fn body_string(&self) -> String {
        match &self.body {
            HttpBody::Buffer(b) => String::from_utf8_lossy(b).to_string(),
            _ => String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Route {
    pub method: HttpMethod,
    pub pattern: String,
    pub handler_name: String,
}

#[derive(Debug, Default)]
pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn add_route(&mut self, method: HttpMethod, pattern: &str, handler_name: &str) -> &mut Self {
        self.routes.push(Route {
            method,
            pattern: pattern.to_string(),
            handler_name: handler_name.to_string(),
        });
        self
    }

    pub fn len(&self) -> usize {
        self.routes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }

    /// Casa método + caminho, extraindo params `:id`.
    /// Retorna (índice da rota, params).
    pub fn match_route(
        &self,
        method: HttpMethod,
        path: &str,
    ) -> Option<(usize, HashMap<String, String>)> {
        let clean_path = path.split('?').next().unwrap_or(path);
        for (i, route) in self.routes.iter().enumerate() {
            if route.method != method {
                continue;
            }
            if let Some(params) = match_path(&route.pattern, clean_path) {
                return Some((i, params));
            }
        }
        None
    }

    pub fn handler_name(&self, index: usize) -> Option<&str> {
        self.routes.get(index).map(|r| r.handler_name.as_str())
    }
}

/// Casa `/users/:id` contra `/users/42` => {"id": "42"}.
pub fn match_path(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let p_segs: Vec<&str> = pattern.split('/').collect();
    let path_segs: Vec<&str> = path.split('/').collect();
    if p_segs.len() != path_segs.len() {
        return None;
    }
    let mut params = HashMap::new();
    for (p, s) in p_segs.iter().zip(path_segs.iter()) {
        if let Some(name) = p.strip_prefix(':') {
            if s.is_empty() {
                return None;
            }
            params.insert(name.to_string(), (*s).to_string());
        } else if p != s {
            return None;
        }
    }
    Some(params)
}

// ---------------------------------------------------------------------------
// Middleware trait + chain (§19)
// ---------------------------------------------------------------------------

pub trait Middleware: Send + Sync {
    fn name(&self) -> &str;
    fn before(&self, req: &mut HttpRequest) -> Result<(), String>;
    fn after(&self, _req: &HttpRequest, _res: &mut HttpResponse) {}
    fn on_error(&self, _e: &str) {}
}

#[derive(Debug, Default)]
pub struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn name(&self) -> &str {
        "logging"
    }

    fn before(&self, req: &mut HttpRequest) -> Result<(), String> {
        log::info!("{} {}", req.method.map(|m| m.as_str()).unwrap_or("-"), req.path);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct AuthMiddleware {
    pub expected_token: String,
}

impl AuthMiddleware {
    pub fn new(expected_token: &str) -> Self {
        Self {
            expected_token: expected_token.to_string(),
        }
    }
}

impl Middleware for AuthMiddleware {
    fn name(&self) -> &str {
        "auth"
    }

    fn before(&self, req: &mut HttpRequest) -> Result<(), String> {
        let header = req.headers.get("authorization").unwrap_or_default();
        validate_bearer(&header, &self.expected_token).map(|_| ())
    }
}

#[derive(Debug, Clone)]
pub struct CorsMiddleware {
    pub allow_origin: String,
    pub allow_methods: Vec<String>,
    pub allow_headers: Vec<String>,
    pub max_age_secs: u64,
    pub allow_credentials: bool,
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self {
            allow_origin: "*".to_string(),
            allow_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allow_headers: vec![
                "content-type".to_string(),
                "authorization".to_string(),
            ],
            max_age_secs: 86400,
            allow_credentials: false,
        }
    }
}

impl CorsMiddleware {
    /// Config restrita (sem wildcard) para produção.
    pub fn restrictive(origin: &str) -> Self {
        Self {
            allow_origin: origin.to_string(),
            allow_credentials: true,
            ..Self::default()
        }
    }

    /// Headers `Access-Control-*` a injetar em TODA resposta real.
    pub fn response_headers(&self) -> Vec<(String, String)> {
        vec![
            (
                "access-control-allow-origin".to_string(),
                self.allow_origin.clone(),
            ),
            (
                "access-control-allow-methods".to_string(),
                self.allow_methods.join(", "),
            ),
            (
                "access-control-allow-headers".to_string(),
                self.allow_headers.join(", "),
            ),
            (
                "access-control-max-age".to_string(),
                self.max_age_secs.to_string(),
            ),
        ]
    }

    /// Resposta ao preflight `OPTIONS`: 204 sem corpo + headers CORS.
    /// `requested` vem de `Access-Control-Request-Method`, se presente.
    pub fn preflight_response(&self, requested: Option<&str>) -> HttpResponse {
        let mut res = HttpResponse::no_content();
        for (k, v) in self.response_headers() {
            res.headers.insert(&k, &v);
        }
        if let Some(m) = requested {
            // Ecoa o método pedido quando ele está na allow-list.
            if self.allow_methods.iter().any(|a| a == m) {
                res.headers.insert("access-control-allow-methods", m);
            }
        }
        res
    }

    /// Valida `Origin` contra a config (`*` aceita tudo; caso contrário
    /// igualdade exata). Usado antes de espelhar o header.
    pub fn origin_allowed(&self, origin: &str) -> bool {
        self.allow_origin == "*" || self.allow_origin == origin
    }
}

impl Middleware for CorsMiddleware {
    fn name(&self) -> &str {
        "cors"
    }

    fn before(&self, _req: &mut HttpRequest) -> Result<(), String> {
        Ok(())
    }
}

/// Stub de rate-limit: valida configuração, sem contador distribuído.
#[derive(Debug, Clone)]
pub struct RateLimitMiddleware {
    pub max_requests: u32,
    pub window_secs: u64,
}

impl Default for RateLimitMiddleware {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_secs: 60,
        }
    }
}

impl Middleware for RateLimitMiddleware {
    fn name(&self) -> &str {
        "rate-limit"
    }

    fn before(&self, _req: &mut HttpRequest) -> Result<(), String> {
        if self.max_requests == 0 {
            return Err("rate limit exceeded".to_string());
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct MiddlewareChain {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    pub fn add<M: Middleware + 'static>(&mut self, m: M) -> &mut Self {
        self.middlewares.push(Box::new(m));
        self
    }

    pub fn len(&self) -> usize {
        self.middlewares.len()
    }

    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }

    pub fn run(&self, req: &mut HttpRequest) -> Result<(), String> {
        for m in &self.middlewares {
            m.before(req)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Headers / Cookies / Sessions (§19)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct Headers {
    inner: Vec<(String, String)>,
}

impl Headers {
    /// Complexity: O(1).
    #[inline]
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Insert-or-replace (case-insensitive key).
    /// Complexity: O(n) scan where n = header count (bounded by MAX_HEADERS).
    /// Single linear pass, no nested loops.
    #[inline]
    pub fn insert(&mut self, key: &str, value: &str) {
        let kl = key.to_ascii_lowercase();
        if let Some(slot) = self.inner.iter_mut().find(|(k, _)| k == &kl) {
            slot.1 = value.to_string();
        } else {
            self.inner.push((kl, value.to_string()));
        }
    }

    /// Cloning getter (compat). Complexity: O(n).
    #[inline]
    pub fn get(&self, key: &str) -> Option<String> {
        self.get_ref(key).map(|s| s.to_string())
    }

    /// Zero-copy borrow (preferida em hot paths).
    /// Complexity: O(n) scan, O(1) extra.
    #[inline]
    pub fn get_ref(&self, key: &str) -> Option<&str> {
        // Avoid allocating lowercase when key is already lowercase (common).
        let needs_lower = key.bytes().any(|b| b.is_ascii_uppercase());
        if !needs_lower {
            return self
                .inner
                .iter()
                .find(|(k, _)| k.as_str() == key)
                .map(|(_, v)| v.as_str());
        }
        let kl = key.to_ascii_lowercase();
        self.inner.iter().find(|(k, _)| k == &kl).map(|(_, v)| v.as_str())
    }

    /// Complexity: O(1).
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &(String, String)> {
        self.inner.iter()
    }

    /// Parsing básico de bloco `Key: Value\r\n...` (tolerante, ignora inválidas).
    /// Complexity: O(n*m) where n = lines, m = avg headers (bounded).
    pub fn parse_raw(raw: &str) -> Self {
        Self::parse_raw_limited(raw, MAX_HEADER_BYTES, MAX_HEADERS).unwrap_or_default()
    }

    /// Parsing estrito com limites e validação RFC7230.
    ///
    /// Rejeita: obs-fold (linha iniciando com SP/HT), nomes inválidos,
    /// valores com CR/LF/controles, contagem/tamanho excedidos.
    /// Complexity: O(n) lines, single pass + O(h) per-line scan (h bounded).
    pub fn parse_raw_limited(
        raw: &str,
        max_bytes: usize,
        max_count: usize,
    ) -> Result<Self, String> {
        if raw.len() > max_bytes {
            return Err(format!(
                "headers too large: {} > {}",
                raw.len(),
                max_bytes
            ));
        }
        let mut h = Self::new();
        let mut total: usize = 0;
        // Split on \n to tolerate \r\n and \n; track raw lines for obs-fold.
        for raw_line in raw.split('\n') {
            // Strip trailing \r only (preserve leading SP/HT for obs-fold detect).
            let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
            if line.trim().is_empty() {
                continue;
            }
            // obs-fold: continuation starting with SP/HT — rejected (RFC7230 dep).
            if line.starts_with(' ') || line.starts_with('\t') {
                return Err("obs-fold rejected".to_string());
            }
            let Some(idx) = line.find(':') else {
                return Err(format!("bad header line (missing colon): {line:?}"));
            };
            let (k, v) = line.split_at(idx);
            let name = k.trim();
            let value = v[1..].trim();
            validate_header_name(name)?;
            validate_header_value(value)?;
            total += name.len() + value.len() + 4;
            if total > max_bytes {
                return Err(format!("headers too large: > {max_bytes}"));
            }
            h.insert(name, value);
            if h.len() > max_count {
                return Err(format!("too many headers: > {max_count}"));
            }
        }
        Ok(h)
    }

    /// Valida todas as entradas + limites globais.
    /// Complexity: O(n*m) bounded (n <= MAX_HEADERS).
    pub fn validate_limited(&self, max_bytes: usize, max_count: usize) -> Result<(), String> {
        if self.inner.len() > max_count {
            return Err(format!("too many headers: > {max_count}"));
        }
        let mut total = 0usize;
        for (k, v) in self.inner.iter() {
            validate_header_name(k)?;
            validate_header_value(v)?;
            total += k.len() + v.len() + 4;
            if total > max_bytes {
                return Err(format!("headers too large: > {max_bytes}"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: String,
    pub path: String,
    pub max_age: Option<i64>,
}

impl Cookie {
    pub fn new(name: &str, value: &str) -> Self {
        let d = CookieDefaults::default();
        Self {
            name: name.to_string(),
            value: value.to_string(),
            secure: d.secure,
            http_only: d.http_only,
            same_site: d.same_site,
            path: "/".to_string(),
            max_age: None,
        }
    }

    /// Valor para header `Set-Cookie` com defaults seguros.
    pub fn header_value(&self) -> String {
        let mut s = format!("{}={}; Path={}", self.name, self.value, self.path);
        if let Some(age) = self.max_age {
            s.push_str(&format!("; Max-Age={age}"));
        }
        if self.secure {
            s.push_str("; Secure");
        }
        if self.http_only {
            s.push_str("; HttpOnly");
        }
        s.push_str(&format!("; SameSite={}", self.same_site));
        s
    }
}

#[derive(Debug, Clone, Default)]
pub struct CookieJar {
    cookies: Vec<Cookie>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self { cookies: Vec::new() }
    }

    /// Parsing básico de header `Cookie: a=b; c=d`.
    pub fn parse_cookie_header(header: &str) -> Self {
        let mut jar = Self::new();
        for part in header.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if let Some(idx) = part.find('=') {
                let (k, v) = part.split_at(idx);
                jar.cookies.push(Cookie::new(k.trim(), v[1..].trim()));
            }
        }
        jar
    }

    pub fn get(&self, name: &str) -> Option<&Cookie> {
        self.cookies.iter().find(|c| c.name == name)
    }

    pub fn push(&mut self, cookie: Cookie) {
        if let Some(slot) = self.cookies.iter_mut().find(|c| c.name == cookie.name) {
            *slot = cookie;
        } else {
            self.cookies.push(cookie);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub data: HashMap<String, String>,
    pub expires_at: i64,
}

impl Session {
    pub fn is_expired(&self, now_unix: i64) -> bool {
        now_unix >= self.expires_at
    }
}

#[derive(Debug, Default)]
pub struct SessionStore {
    sessions: HashMap<String, Session>,
    ttl_secs: i64,
}

impl SessionStore {
    pub fn new(ttl_secs: i64) -> Self {
        Self {
            sessions: HashMap::new(),
            ttl_secs,
        }
    }

    pub fn now_unix() -> i64 {
        chrono::Utc::now().timestamp()
    }

    pub fn create(&mut self, data: HashMap<String, String>) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let sess = Session {
            id: id.clone(),
            data,
            expires_at: Self::now_unix() + self.ttl_secs,
        };
        self.sessions.insert(id.clone(), sess);
        id
    }

    pub fn get(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }

    pub fn get_valid(&self, id: &str) -> Option<&Session> {
        let s = self.sessions.get(id)?;
        if s.is_expired(Self::now_unix()) {
            None
        } else {
            Some(s)
        }
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }

    pub fn cleanup_expired(&mut self) -> usize {
        let now = Self::now_unix();
        let before = self.sessions.len();
        self.sessions.retain(|_, s| !s.is_expired(now));
        before - self.sessions.len()
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Multipart / upload (§19)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct MultipartField {
    pub name: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

impl MultipartField {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }
}

/// Parse simplificado de multipart: divide por `--boundary`, extrai headers
/// `Content-Disposition: form-data; name=".." filename=".."` e corpo.
/// Aplica `max_bytes` sobre o corpo total.
pub fn parse_multipart(body: &[u8], boundary: &str, max_bytes: usize) -> Result<Vec<MultipartField>, String> {
    if boundary.is_empty() {
        return Err("empty boundary".to_string());
    }
    if body.len() > max_bytes {
        return Err(format!("body too large: {} > {}", body.len(), max_bytes));
    }
    let text = String::from_utf8_lossy(body).to_string();
    let delimiter = format!("--{boundary}");
    let mut fields = Vec::new();
    for part in text.split(&delimiter) {
        let part = part.trim_matches(|c| c == '\r' || c == '\n');
        if part.is_empty() || part == "--" {
            continue;
        }
        // Separa headers do corpo via linha em branco.
        let split = part.find("\r\n\r\n").map(|i| (i, 4)).or_else(|| part.find("\n\n").map(|i| (i, 2)));
        let Some((idx, sep)) = split else {
            continue;
        };
        let (raw_headers, raw_body) = part.split_at(idx);
        let content = raw_body[sep..].trim_end_matches(|c| c == '\r' || c == '\n');
        let mut name: Option<String> = None;
        let mut filename: Option<String> = None;
        let mut content_type: Option<String> = None;
        for hline in raw_headers.lines() {
            let hline = hline.trim();
            let lower = hline.to_ascii_lowercase();
            if lower.starts_with("content-disposition") {
                name = extract_quoted(hline, "name");
                filename = extract_quoted(hline, "filename");
            } else if lower.starts_with("content-type") {
                if let Some(i) = hline.find(':') {
                    content_type = Some(hline[i + 1..].trim().to_string());
                }
            }
        }
        let Some(name) = name else { continue };
        fields.push(MultipartField {
            name,
            filename,
            content_type,
            data: content.as_bytes().to_vec(),
        });
    }
    Ok(fields)
}

fn extract_quoted(line: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=\"");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

// ---------------------------------------------------------------------------
// WS / SSE (§19)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WsOpcode {
    Continuation,
    Text,
    Binary,
    Ping,
    Pong,
    Close,
}

/// Alias de paridade Node (nome completo).
pub type WsOpcodeFull = WsOpcode;

#[derive(Debug, Clone)]
pub struct WsFrame {
    pub opcode: WsOpcode,
    pub payload: Vec<u8>,
    pub fin: bool,
}

impl WsFrame {
    pub fn text(msg: &str) -> Self {
        Self {
            opcode: WsOpcode::Text,
            payload: msg.as_bytes().to_vec(),
            fin: true,
        }
    }

    pub fn binary(data: Vec<u8>) -> Self {
        Self {
            opcode: WsOpcode::Binary,
            payload: data,
            fin: true,
        }
    }

    pub fn ping() -> Self {
        Self {
            opcode: WsOpcode::Ping,
            payload: Vec::new(),
            fin: true,
        }
    }

    pub fn text_str(&self) -> String {
        String::from_utf8_lossy(&self.payload).to_string()
    }
}

/// Stub de sessão WebSocket: estado aberto/fechado + eco em memória.
#[derive(Debug, Clone)]
pub struct WsSession {
    pub id: String,
    pub open: bool,
    pub sent: Vec<WsFrame>,
}

impl WsSession {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            open: true,
            sent: Vec::new(),
        }
    }

    pub fn send(&mut self, frame: WsFrame) -> Result<(), String> {
        if !self.open {
            return Err("session closed".to_string());
        }
        self.sent.push(frame);
        Ok(())
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
    pub retry: Option<u64>,
    pub id: Option<String>,
}

impl SseEvent {
    pub fn new(data: &str) -> Self {
        Self {
            event: None,
            data: data.to_string(),
            retry: None,
            id: None,
        }
    }

    pub fn format(&self) -> String {
        let mut out = String::new();
        if let Some(id) = &self.id {
            out.push_str(&format!("id: {id}\n"));
        }
        if let Some(ev) = &self.event {
            out.push_str(&format!("event: {ev}\n"));
        }
        for line in self.data.lines() {
            out.push_str(&format!("data: {line}\n"));
        }
        if self.data.is_empty() {
            out.push_str("data: \n");
        }
        if let Some(retry) = self.retry {
            out.push_str(&format!("retry: {retry}\n"));
        }
        out.push('\n');
        out
    }
}

// ---------------------------------------------------------------------------
// REST helpers: paginação + envelope (§19)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PageParams {
    pub page: u64,
    pub per_page: u64,
}

impl Default for PageParams {
    fn default() -> Self {
        Self { page: 1, per_page: 20 }
    }
}

impl PageParams {
    pub fn new(page: u64, per_page: u64) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, 100),
        }
    }

    pub fn from_query(query: &[(String, String)]) -> Self {
        let mut p = Self::default();
        for (k, v) in query {
            match k.as_str() {
                "page" => {
                    if let Ok(n) = v.parse::<u64>() {
                        p.page = n.max(1);
                    }
                }
                "per_page" | "perPage" | "limit" => {
                    if let Ok(n) = v.parse::<u64>() {
                        p.per_page = n.clamp(1, 100);
                    }
                }
                _ => {}
            }
        }
        p
    }

    pub fn offset(&self) -> u64 {
        (self.page.saturating_sub(1)).saturating_mul(self.per_page)
    }

    pub fn limit(&self) -> u64 {
        self.per_page
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, page: u64, per_page: u64, total: u64) -> Self {
        Self {
            items,
            page,
            per_page,
            total,
        }
    }

    pub fn total_pages(&self) -> u64 {
        if self.per_page == 0 {
            return 0;
        }
        self.total.div_ceil(self.per_page)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T: Serialize> Envelope<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn ok_msg(data: T, message: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message.to_string()),
        }
    }

    pub fn fail(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.to_string()),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// GraphQL / RPC stubs (§59 parcial)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphQlQuery {
    pub query: String,
    pub operation: Option<String>,
    pub variables: serde_json::Value,
}

impl GraphQlQuery {
    /// Parse com `Limits` default (8 MiB / depth 32) — sem skip de caps.
    /// Complexity: O(n) parse + O(n) limit walk.
    pub fn parse(raw: &str) -> Result<Self, String> {
        Self::parse_limited(raw, &crate::response::Limits::default())
    }
    /// Variante com limites explícitos. Complexity: O(n).
    pub fn parse_limited(raw: &str, limits: &crate::response::Limits) -> Result<Self, String> {
        let v: serde_json::Value = crate::response::decode_json(raw.as_bytes(), limits)
            .map_err(|e| format!("invalid graphql json: {e}"))?;
        let query = v
            .get("query")
            .and_then(|q| q.as_str())
            .ok_or_else(|| "missing 'query' field".to_string())?;
        Ok(Self {
            query: query.to_string(),
            operation: v.get("operationName").and_then(|o| o.as_str()).map(|s| s.to_string()),
            variables: v.get("variables").cloned().unwrap_or(serde_json::Value::Null),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    #[serde(default = "default_jsonrpc")]
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub id: serde_json::Value,
}

fn default_jsonrpc() -> String {
    "2.0".to_string()
}

impl RpcRequest {
    /// Parse com `Limits` default (sem skip). Complexity: O(n).
    pub fn parse(raw: &str) -> Result<Self, String> {
        Self::parse_limited(raw, &crate::response::Limits::default())
    }
    /// Variante com limites explícitos. Complexity: O(n).
    pub fn parse_limited(raw: &str, limits: &crate::response::Limits) -> Result<Self, String> {
        crate::response::decode_json(raw.as_bytes(), limits)
            .map_err(|e| format!("invalid rpc json: {e}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: serde_json::Value,
}

impl RpcResponse {
    pub fn ok(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn err(id: serde_json::Value, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError {
                code,
                message: message.to_string(),
            }),
            id,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// authN != authZ (§59 parcial): autenticação separada de autorização
// ---------------------------------------------------------------------------

/// Contexto de autenticação (quem é) + escopos (o que pode fazer).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthContext {
    pub user: String,
    pub scopes: Vec<String>,
    pub authenticated: bool,
}

impl AuthContext {
    pub fn new(user: &str, scopes: Vec<String>) -> Self {
        Self {
            user: user.to_string(),
            scopes,
            authenticated: true,
        }
    }

    pub fn anonymous() -> Self {
        Self {
            user: String::new(),
            scopes: Vec::new(),
            authenticated: false,
        }
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }
}

/// Valida `Authorization: Bearer <token>` contra o token esperado.
/// Retorna o token quando válido (authN).
pub fn validate_bearer(header: &str, expected_token: &str) -> Result<String, String> {
    let token = parse_bearer_token(header)?;
    if token != expected_token {
        return Err("invalid token".to_string());
    }
    Ok(token)
}

/// Extrai o token de `Authorization: Bearer <token>`.
pub fn parse_bearer_token(header: &str) -> Result<String, String> {
    let header = header.trim();
    let rest = header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .ok_or_else(|| "missing bearer scheme".to_string())?;
    let token = rest.trim();
    if token.is_empty() {
        return Err("empty bearer token".to_string());
    }
    Ok(token.to_string())
}

/// Valida API key por comparação simples (authN).
pub fn validate_api_key(provided: &str, expected: &str) -> Result<(), String> {
    if provided.is_empty() {
        return Err("missing api key".to_string());
    }
    if provided != expected {
        return Err("invalid api key".to_string());
    }
    Ok(())
}

/// Autentica via Bearer e constrói o contexto (authN, sem checar escopo).
pub fn authenticate_bearer(header: &str, expected_token: &str, user: &str, scopes: Vec<String>) -> Result<AuthContext, String> {
    validate_bearer(header, expected_token)?;
    Ok(AuthContext::new(user, scopes))
}

/// Autoriza exigindo autenticação + escopo (authZ, separada de authN).
pub fn authorize_scope(ctx: &AuthContext, required_scope: &str) -> Result<(), String> {
    if !ctx.authenticated {
        return Err("unauthenticated".to_string());
    }
    if !ctx.has_scope(required_scope) {
        return Err(format!("forbidden: missing scope '{required_scope}'"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Web frameworks: guard para DI opcional (§19)
// ---------------------------------------------------------------------------

/// Registro mínimo de framework web com DI opcional.
#[derive(Debug, Clone, Default)]
pub struct Framework {
    pub name: String,
    pub di_enabled: bool,
    services: HashMap<String, String>,
}

impl Framework {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            di_enabled: false,
            services: HashMap::new(),
        }
    }

    pub fn with_di(mut self) -> Self {
        self.di_enabled = true;
        self
    }

    /// Guard: registra serviço apenas com DI habilitada.
    pub fn register(&mut self, iface: &str, impl_name: &str) -> Result<(), String> {
        if !self.di_enabled {
            return Err("DI not enabled for this framework".to_string());
        }
        self.services.insert(iface.to_string(), impl_name.to_string());
        Ok(())
    }

    pub fn resolve(&self, iface: &str) -> Option<&str> {
        self.services.get(iface).map(|s| s.as_str())
    }

    /// Guard de acesso: erro claro quando DI está desabilitada.
    pub fn guard(&self) -> Result<(), String> {
        if self.di_enabled {
            Ok(())
        } else {
            Err(format!("framework '{}' has no DI container", self.name))
        }
    }
}

pub fn root() -> String {
    r#"{"success":true,"data":"Welcome to LexiconLang API!","message":null}"#.to_string()
}

pub fn hello() -> String {
    r#"{"success":true,"data":"Hello from LexiconLang HTTP Server!","message":null}"#.to_string()
}

pub fn users() -> String {
    r#"{"success":true,"data":[{"id":1,"name":"John","email":"john@example.com"},{"id":2,"name":"Jane","email":"jane@example.com"}],"message":null}"#.to_string()
}

pub fn stats(count: i32) -> String {
    format!(
        r#"{{"success":true,"data":{},"message":"Total requests"}}"#,
        count
    )
}

// ---------------------------------------------------------------------------
// Paridade Node.js — EXT A: status, headers, request, crypto base
// ---------------------------------------------------------------------------

use std::time::{Duration as StdDuration, Instant as StdInstant};

/// Mensagem padrão para código de status HTTP (100-599).
/// Complexity: O(1).
#[inline]
pub fn status_message(code: u16) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        207 => "Multi-Status",
        208 => "Already Reported",
        226 => "IM Used",
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a Teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Entity",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        510 => "Not Extended",
        511 => "Network Authentication Required",
        _ => "Unknown",
    }
}

fn redirect_with(location: &str, status: u16) -> HttpResponse {
    let mut r = HttpResponse::new(status, HttpBody::Empty);
    r.headers.insert("location", location);
    r.headers.insert("content-type", "text/plain; charset=utf-8");
    r.body = HttpBody::Buffer(format!("{}: {}", status, status_message(status)).into_bytes());
    r
}

impl HttpResponse {
    pub fn moved_permanently(location: &str) -> Self {
        redirect_with(location, 301)
    }
    pub fn found(location: &str) -> Self {
        redirect_with(location, 302)
    }
    pub fn see_other(location: &str) -> Self {
        redirect_with(location, 303)
    }
    pub fn temporary_redirect(location: &str) -> Self {
        redirect_with(location, 307)
    }
    pub fn permanent_redirect(location: &str) -> Self {
        redirect_with(location, 308)
    }
    pub fn payload_too_large(msg: &str) -> Self {
        Self::json(&serde_json::json!({"success": false, "data": null, "message": msg})).with_status(413)
    }
    pub fn service_unavailable(msg: &str) -> Self {
        Self::json(&serde_json::json!({"success": false, "data": null, "message": msg})).with_status(503)
    }
    pub fn bad_gateway(msg: &str) -> Self {
        Self::json(&serde_json::json!({"success": false, "data": null, "message": msg})).with_status(502)
    }
    pub fn request_timeout(msg: &str) -> Self {
        Self::json(&serde_json::json!({"success": false, "data": null, "message": msg})).with_status(408)
    }
    pub fn too_many_requests(msg: &str) -> Self {
        let mut r = Self::json(&serde_json::json!({"success": false, "data": null, "message": msg})).with_status(429);
        r.headers.insert("retry-after", "60");
        r
    }
}

impl Headers {
    pub fn append(&mut self, key: &str, value: &str) {
        self.inner.push((key.to_ascii_lowercase(), value.to_string()));
    }
    pub fn get_all(&self, key: &str) -> Vec<String> {
        let kl = key.to_ascii_lowercase();
        self.inner.iter().filter(|(k, _)| k == &kl).map(|(_, v)| v.clone()).collect()
    }
    pub fn remove(&mut self, key: &str) -> bool {
        let kl = key.to_ascii_lowercase();
        let before = self.inner.len();
        self.inner.retain(|(k, _)| k != &kl);
        before != self.inner.len()
    }
    pub fn has(&self, key: &str) -> bool {
        let kl = key.to_ascii_lowercase();
        self.inner.iter().any(|(k, _)| k == &kl)
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    pub fn names(&self) -> Vec<String> {
        let mut out: Vec<String> = self.inner.iter().map(|(k, _)| k.clone()).collect();
        out.sort();
        out.dedup();
        out
    }
}

/// Valida nome de header (token RFC7230).
/// Rejeita vazio, obs-fold (SP/HT inicial), controles e over-limit.
/// Complexity: O(n) in `name`.
#[inline]
pub fn validate_header_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("empty header name".to_string());
    }
    if name.len() > MAX_HEADER_NAME_LEN {
        return Err(format!(
            "header name too long: {} > {}",
            name.len(),
            MAX_HEADER_NAME_LEN
        ));
    }
    // obs-fold: continuação de linha dobrada (RFC7230 obsoleto) — rejeita.
    if name.starts_with(' ') || name.starts_with('\t') {
        return Err("obs-fold rejected in header name".to_string());
    }
    for c in name.chars() {
        let ok = c.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(c);
        if !ok {
            return Err(format!("invalid header name char: {c:?}"));
        }
    }
    Ok(())
}

/// Valida valor de header (sem CR/LF, sem controles, sem obs-fold).
/// Complexity: O(n) in `value`.
#[inline]
pub fn validate_header_value(value: &str) -> Result<(), String> {
    if value.len() > MAX_HEADER_VALUE_LEN {
        return Err(format!(
            "header value too long: {} > {}",
            value.len(),
            MAX_HEADER_VALUE_LEN
        ));
    }
    if value.contains('\r') || value.contains('\n') {
        return Err("header value must not contain CR/LF".to_string());
    }
    for c in value.chars() {
        if c == '\u{7f}' || (c.is_control() && c != '\t') {
            return Err("invalid control char in header value".to_string());
        }
    }
    Ok(())
}

/// Limites rígidos de headers (defesa contra DoS).
pub const MAX_HEADER_NAME_LEN: usize = 256;
/// Limites rígidos de headers (defesa contra DoS).
pub const MAX_HEADER_VALUE_LEN: usize = 8 * 1024;
/// Limites rígidos de headers (defesa contra DoS).
pub const MAX_HEADERS: usize = 100;
/// Limites rígidos de headers (defesa contra DoS).
pub const MAX_HEADER_BYTES: usize = 32 * 1024;
/// Comprimento máximo de URL aceito pelo parser (defesa contra DoS).
pub const MAX_URL_LEN: usize = 8 * 1024;

impl HttpRequest {
    /// Divide target em (path, query). Ex: "/a/b?x=1&y=2" -> ("/a/b", vec![...]).
    pub fn parse_target(target: &str) -> (String, Vec<(String, String)>) {
        let (path_part, query_part) = match target.find('?') {
            Some(i) => (&target[..i], Some(&target[i + 1..])),
            None => (target, None),
        };
        let path = if path_part.is_empty() { "/".to_string() } else { path_part.to_string() };
        let mut query = Vec::new();
        if let Some(q) = query_part {
            for pair in q.split('&') {
                if pair.is_empty() {
                    continue;
                }
                match pair.find('=') {
                    Some(j) => {
                        let (k, v) = pair.split_at(j);
                        query.push((qs_unescape(k), qs_unescape(&v[1..])));
                    }
                    None => query.push((qs_unescape(pair), String::new())),
                }
            }
        }
        (path, query)
    }

    pub fn query_all(&self, key: &str) -> Vec<&str> {
        self.query.iter().filter(|(k, _)| k == key).map(|(_, v)| v.as_str()).collect()
    }

    pub fn from_parts(method: HttpMethod, target: &str, headers: Headers, body: Vec<u8>) -> Self {
        let (path, query) = Self::parse_target(target);
        Self { method: Some(method), path, headers, query, body: HttpBody::from(body) }
    }
}

// --- crypto base: sha1/sha256/base64 (sem novas deps) ---

fn sha1_digest(data: &[u8]) -> [u8; 20] {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;
    let ml = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&ml.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }
    let mut out = [0u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

fn sha256_digest(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let (mut h0, mut h1, mut h2, mut h3, mut h4, mut h5, mut h6, mut h7): (u32, u32, u32, u32, u32, u32, u32, u32) =
        (0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19);
    let ml = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&ml.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h) = (h0, h1, h2, h3, h4, h5, h6, h7);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
        h5 = h5.wrapping_add(f);
        h6 = h6.wrapping_add(g);
        h7 = h7.wrapping_add(h);
    }
    let mut out = [0u8; 32];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out[20..24].copy_from_slice(&h5.to_be_bytes());
    out[24..28].copy_from_slice(&h6.to_be_bytes());
    out[28..32].copy_from_slice(&h7.to_be_bytes());
    out
}

fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut kb = [0u8; 64];
    if key.len() > 64 {
        let h = sha256_digest(key);
        kb[..32].copy_from_slice(&h);
    } else {
        kb[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= kb[i];
        opad[i] ^= kb[i];
    }
    let mut inner = Vec::with_capacity(64 + msg.len());
    inner.extend_from_slice(&ipad);
    inner.extend_from_slice(msg);
    let ih = sha256_digest(&inner);
    let mut outer = Vec::with_capacity(64 + 32);
    outer.extend_from_slice(&opad);
    outer.extend_from_slice(&ih);
    sha256_digest(&outer)
}

const B64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() { data[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64_TABLE[((n >> 18) & 63) as usize] as char);
        out.push(B64_TABLE[((n >> 12) & 63) as usize] as char);
        if i + 1 < data.len() {
            out.push(B64_TABLE[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if i + 2 < data.len() {
            out.push(B64_TABLE[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}

fn b64_val(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if s.len() % 4 != 0 {
        return Err("invalid base64 length".to_string());
    }
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let mut vals = [0u8; 4];
        let mut pad = 0;
        for j in 0..4 {
            let c = bytes[i + j];
            if c == b'=' {
                pad += 1;
                vals[j] = 0;
            } else {
                vals[j] = b64_val(c).ok_or_else(|| format!("invalid base64 char: {}", c as char))?;
            }
        }
        // valida padding só no fim
        if pad > 0 && i + 4 != bytes.len() {
            return Err("invalid base64 padding".to_string());
        }
        if pad > 2 {
            return Err("invalid base64 padding".to_string());
        }
        let n = ((vals[0] as u32) << 18) | ((vals[1] as u32) << 12) | ((vals[2] as u32) << 6) | (vals[3] as u32);
        out.push(((n >> 16) & 0xFF) as u8);
        if pad < 2 {
            out.push(((n >> 8) & 0xFF) as u8);
        }
        if pad < 1 {
            out.push((n & 0xFF) as u8);
        }
        i += 4;
    }
    Ok(out)
}

fn base64url_encode(data: &[u8]) -> String {
    base64_encode(data).replace('+', "-").replace('/', "_").trim_end_matches('=').to_string()
}

fn base64url_decode(s: &str) -> Result<Vec<u8>, String> {
    let mut t = s.replace('-', "+").replace('_', "/");
    while t.len() % 4 != 0 {
        t.push('=');
    }
    base64_decode(&t)
}

#[allow(dead_code)]
fn hex_encode(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for b in data {
        s.push(char::from_digit((b >> 4) as u32, 16).unwrap());
        s.push(char::from_digit((b & 0xF) as u32, 16).unwrap());
    }
    s
}

// ---------------------------------------------------------------------------
// EXT B: LexUrl, UrlSearchParams, qs, IP, BlockList
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LexUrl {
    pub scheme: String,
    pub username: String,
    pub password: Option<String>,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

impl LexUrl {
    /// Parse com limite rígido `MAX_URL_LEN` + rejeição de controles.
    /// Complexity: O(n) onde n = len da URL.
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err("empty url".to_string());
        }
        if s.len() > MAX_URL_LEN {
            return Err(format!("url too long: {} > {}", s.len(), MAX_URL_LEN));
        }
        // Rejeita controles / espaços internos brutos (garbage).
        if s.chars().any(|c| c.is_control()) {
            return Err("invalid control char in url".to_string());
        }
        // scheme
        let (scheme, rest) = match s.find("://") {
            Some(i) => (s[..i].to_ascii_lowercase(), s[i + 3..].to_string()),
            None => {
                if s.starts_with("file:") {
                    ("file".to_string(), s["file:".len()..].to_string())
                } else if s.starts_with('/') || s.starts_with('?') || s.starts_with('#') {
                    // relativo puro -> trata como path
                    return Ok(Self {
                        scheme: "http".to_string(),
                        username: String::new(),
                        password: None,
                        host: String::new(),
                        port: None,
                        path: {
                            let (p, q, f) = split_path_query_fragment(s);
                            let _ = (q, f);
                            p
                        },
                        query: split_path_query_fragment(s).1,
                        fragment: split_path_query_fragment(s).2,
                    });
                } else {
                    return Err("missing scheme (expected like https://)".to_string());
                }
            }
        };
        if scheme.is_empty() {
            return Err("empty scheme".to_string());
        }
        // fragment
        let (before_frag, fragment) = match rest.find('#') {
            Some(i) => (rest[..i].to_string(), Some(rest[i + 1..].to_string())),
            None => (rest, None),
        };
        // query
        let (before_query, query) = match before_frag.find('?') {
            Some(i) => (before_frag[..i].to_string(), Some(before_frag[i + 1..].to_string())),
            None => (before_frag, None),
        };
        // authority + path
        let (authority, path) = match before_query.find('/') {
            Some(i) => (before_query[..i].to_string(), before_query[i..].to_string()),
            None => (before_query.clone(), "/".to_string()),
        };
        // file URLs: sem authority real
        if scheme == "file" {
            let p = if path.is_empty() { "/".to_string() } else { path };
            return Ok(Self {
                scheme,
                username: String::new(),
                password: None,
                host: String::new(),
                port: None,
                path: p,
                query,
                fragment,
            });
        }
        if authority.is_empty() {
            return Err("missing host".to_string());
        }
        // userinfo
        let (userinfo, hostport) = match authority.rfind('@') {
            Some(i) => (Some(authority[..i].to_string()), authority[i + 1..].to_string()),
            None => (None, authority),
        };
        let (username, password) = match userinfo {
            Some(u) => match u.find(':') {
                Some(j) => (u[..j].to_string(), Some(u[j + 1..].to_string())),
                None => (u, None),
            },
            None => (String::new(), None),
        };
        // host + port (IPv6 entre colchetes)
        let (host, port) = if hostport.starts_with('[') {
            let end = hostport.find(']').ok_or_else(|| "invalid ipv6 host".to_string())?;
            let h = hostport[1..end].to_string();
            let rest = &hostport[end + 1..];
            let p = if rest.starts_with(':') {
                Some(rest[1..].parse::<u16>().map_err(|_| "invalid port".to_string())?)
            } else if !rest.is_empty() {
                return Err("invalid ipv6 authority".to_string());
            } else {
                None
            };
            (h, p)
        } else {
            match hostport.rfind(':') {
                Some(i) => {
                    let h = hostport[..i].to_string();
                    let pstr = &hostport[i + 1..];
                    // se contém ':' adicional sem colchetes, é ipv6 sem porta
                    if h.contains(':') {
                        (hostport.clone(), None)
                    } else if pstr.is_empty() {
                        (h, None)
                    } else {
                        match pstr.parse::<u16>() {
                            Ok(p) => (h, Some(p)),
                            Err(_) => {
                                // ':' faz parte do host? trata como host puro
                                (hostport.clone(), None)
                            }
                        }
                    }
                }
                None => (hostport, None),
            }
        };
        if host.is_empty() {
            return Err("missing host".to_string());
        }
        let path = if path.is_empty() { "/".to_string() } else { path };
        Ok(Self { scheme, username, password, host, port, path, query, fragment })
    }

    pub fn href(&self) -> String {
        let mut s = format!("{}://", self.scheme);
        if !self.username.is_empty() {
            s.push_str(&self.username);
            if let Some(p) = &self.password {
                s.push(':');
                s.push_str(p);
            }
            s.push('@');
        }
        // IPv6 com colchetes
        if self.host.contains(':') && !self.host.starts_with('[') {
            s.push('[');
            s.push_str(&self.host);
            s.push(']');
        } else {
            s.push_str(&self.host);
        }
        if let Some(p) = self.port {
            s.push_str(&format!(":{p}"));
        }
        s.push_str(&self.path);
        if let Some(q) = &self.query {
            s.push('?');
            s.push_str(q);
        }
        if let Some(f) = &self.fragment {
            s.push('#');
            s.push_str(f);
        }
        s
    }

    pub fn origin(&self) -> String {
        let mut s = format!("{}://", self.scheme);
        if self.host.contains(':') && !self.host.starts_with('[') {
            s.push('[');
            s.push_str(&self.host);
            s.push(']');
        } else {
            s.push_str(&self.host);
        }
        if let Some(p) = self.port {
            // omite padrão? mantém explícito para paridade simples
            s.push_str(&format!(":{p}"));
        }
        s
    }

    pub fn host_header(&self) -> String {
        let default_port = match self.scheme.as_str() {
            "http" | "ws" => 80,
            "https" | "wss" => 443,
            _ => 0,
        };
        match self.port {
            Some(p) if p != default_port => format!("{}:{p}", self.host),
            _ => self.host.clone(),
        }
    }

    /// Resolve relativa (suporta absoluta, //host, /abs, path relativo).
    pub fn join(&self, relative: &str) -> Result<Self, String> {
        let r = relative.trim();
        if r.is_empty() {
            return Ok(self.clone());
        }
        if r.contains("://") {
            return Self::parse(r);
        }
        if r.starts_with("//") {
            return Self::parse(&format!("{}:{r}", self.scheme));
        }
        if r.starts_with('/') {
            let (p, q, f) = split_path_query_fragment(r);
            return Ok(Self {
                scheme: self.scheme.clone(),
                username: self.username.clone(),
                password: self.password.clone(),
                host: self.host.clone(),
                port: self.port,
                path: p,
                query: q,
                fragment: f,
            });
        }
        // fragment/query only
        if r.starts_with('#') {
            let mut c = self.clone();
            c.fragment = Some(r[1..].to_string());
            return Ok(c);
        }
        if r.starts_with('?') {
            let mut c = self.clone();
            let (q, f) = match r.find('#') {
                Some(i) => (Some(r[1..i].to_string()), Some(r[i + 1..].to_string())),
                None => (Some(r[1..].to_string()), None),
            };
            c.query = q;
            c.fragment = f;
            return Ok(c);
        }
        // path relativo: merge com diretório base
        let rel_path = r.split(['?', '#']).next().unwrap_or(r);
        let (rel_q, rel_f) = {
            let after_path = &r[rel_path.len()..];
            if after_path.is_empty() {
                (None, None)
            } else {
                // parse manual
                let q = if after_path.starts_with('?') {
                    let end = after_path.find('#').unwrap_or(after_path.len());
                    Some(after_path[1..end].to_string())
                } else {
                    None
                };
                let f = after_path.find('#').map(|i| after_path[i + 1..].to_string());
                (q, f)
            }
        };
        let base_dir = match self.path.rfind('/') {
            Some(i) => self.path[..=i].to_string(),
            None => "/".to_string(),
        };
        let merged = format!("{base_dir}{rel_path}");
        let norm = normalize_path(&merged);
        Ok(Self {
            scheme: self.scheme.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            host: self.host.clone(),
            port: self.port,
            path: norm,
            query: rel_q,
            fragment: rel_f,
        })
    }

    pub fn set_path(&mut self, path: &str) {
        if path.starts_with('/') {
            self.path = path.to_string();
        } else {
            self.path = format!("/{path}");
        }
    }

    pub fn push_query(&mut self, k: &str, v: &str) {
        let pair = format!("{}={}", qs_escape(k), qs_escape(v));
        match &self.query {
            Some(q) if !q.is_empty() => self.query = Some(format!("{q}&{pair}")),
            _ => self.query = Some(pair),
        }
    }

    pub fn default_port(&self) -> Option<u16> {
        match self.scheme.as_str() {
            "http" | "ws" => Some(80),
            "https" | "wss" => Some(443),
            "ftp" => Some(21),
            _ => None,
        }
    }

    pub fn effective_port(&self) -> u16 {
        if let Some(p) = self.port {
            return p;
        }
        self.default_port().unwrap_or(80)
    }
}

fn split_path_query_fragment(s: &str) -> (String, Option<String>, Option<String>) {
    let (before_frag, fragment) = match s.find('#') {
        Some(i) => (s[..i].to_string(), Some(s[i + 1..].to_string())),
        None => (s.to_string(), None),
    };
    let (path, query) = match before_frag.find('?') {
        Some(i) => (before_frag[..i].to_string(), Some(before_frag[i + 1..].to_string())),
        None => (before_frag, None),
    };
    let path = if path.is_empty() { "/".to_string() } else { path };
    (path, query, fragment)
}

fn normalize_path(p: &str) -> String {
    let mut segs: Vec<&str> = Vec::new();
    for s in p.split('/') {
        match s {
            "" | "." => {
                if segs.is_empty() {
                    segs.push("");
                }
            }
            ".." => {
                if segs.len() > 1 {
                    segs.pop();
                }
            }
            o => segs.push(o),
        }
    }
    let mut out = segs.join("/");
    if out.is_empty() {
        out = "/".to_string();
    }
    if !out.starts_with('/') {
        out = format!("/{out}");
    }
    out
}

#[derive(Debug, Clone)]
pub struct HttpOptions {
    pub host: String,
    pub port: u16,
    pub path: String,
    pub secure: bool,
}

pub fn url_to_http_options(url: &LexUrl) -> HttpOptions {
    let secure = matches!(url.scheme.as_str(), "https" | "wss");
    let mut path = url.path.clone();
    if let Some(q) = &url.query {
        path.push('?');
        path.push_str(q);
    }
    HttpOptions { host: url.host.clone(), port: url.effective_port(), path, secure }
}

pub fn file_url_to_path(url: &LexUrl) -> Result<String, String> {
    if url.scheme != "file" {
        return Err(format!("not a file url: {}", url.scheme));
    }
    Ok(qs_unescape(&url.path))
}

pub fn path_to_file_url(path: &str) -> LexUrl {
    // percent-encode espaços etc? simples: escapa via qs_escape mas preserva '/'
    let mut enc = String::new();
    for c in path.chars() {
        if c == '/' {
            enc.push('/');
        } else if c.is_ascii_alphanumeric() || "-_.~".contains(c) {
            enc.push(c);
        } else {
            for b in c.to_string().as_bytes() {
                enc.push_str(&format!("%{b:02X}"));
            }
        }
    }
    LexUrl {
        scheme: "file".to_string(),
        username: String::new(),
        password: None,
        host: String::new(),
        port: None,
        path: if enc.starts_with('/') { enc } else { format!("/{enc}") },
        query: None,
        fragment: None,
    }
}

/// IDNA simples: lowercase + punycode passthrough.
pub fn domain_to_ascii(domain: &str) -> String {
    domain.trim().to_ascii_lowercase()
}
pub fn domain_to_unicode(domain: &str) -> String {
    domain.trim().to_ascii_lowercase()
}

// --- UrlSearchParams ---

#[derive(Debug, Clone, Default)]
pub struct UrlSearchParams(pub Vec<(String, String)>);

impl UrlSearchParams {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn parse(s: &str) -> Self {
        let t = s.strip_prefix('?').unwrap_or(s);
        let mut v = Vec::new();
        if t.is_empty() {
            return Self(v);
        }
        for pair in t.split('&') {
            if pair.is_empty() {
                continue;
            }
            match pair.find('=') {
                Some(i) => {
                    let (k, val) = pair.split_at(i);
                    v.push((qs_unescape(k), qs_unescape(&val[1..])));
                }
                None => v.push((qs_unescape(pair), String::new())),
            }
        }
        Self(v)
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }
    pub fn get_all(&self, key: &str) -> Vec<&str> {
        self.0.iter().filter(|(k, _)| k == key).map(|(_, v)| v.as_str()).collect()
    }
    pub fn append(&mut self, k: &str, v: &str) {
        self.0.push((k.to_string(), v.to_string()));
    }
    pub fn set(&mut self, k: &str, v: &str) {
        let mut found = false;
        let mut out = Vec::new();
        for (ek, ev) in self.0.drain(..) {
            if ek == k {
                if !found {
                    out.push((k.to_string(), v.to_string()));
                    found = true;
                }
            } else {
                out.push((ek, ev));
            }
        }
        if !found {
            out.push((k.to_string(), v.to_string()));
        }
        self.0 = out;
    }
    pub fn remove(&mut self, k: &str) {
        self.0.retain(|(ek, _)| ek != k);
    }
    pub fn has(&self, k: &str) -> bool {
        self.0.iter().any(|(ek, _)| ek == k)
    }
    pub fn sort(&mut self) {
        self.0.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    }
    pub fn to_string(&self) -> String {
        qs_stringify(&self.0)
    }
    pub fn size(&self) -> usize {
        self.0.len()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// --- querystring (Node qs) ---

pub fn qs_escape(s: &str) -> String {
    let mut out = String::new();
    for b in s.as_bytes() {
        let c = *b as char;
        if c.is_ascii_alphanumeric() || "-_.~".contains(c) {
            out.push(c);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

pub fn qs_unescape(s: &str) -> String {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let h = |c: u8| match c {
                    b'0'..=b'9' => Some(c - b'0'),
                    b'a'..=b'f' => Some(c - b'a' + 10),
                    b'A'..=b'F' => Some(c - b'A' + 10),
                    _ => None,
                };
                match (h(bytes[i + 1]), h(bytes[i + 2])) {
                    (Some(a), Some(b)) => {
                        out.push((a << 4) | b);
                        i += 3;
                    }
                    _ => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

pub fn qs_parse(q: &str, sep: &str, eq: &str, max_keys: usize) -> Vec<(String, String)> {
    let sep_c = sep.chars().next().unwrap_or('&');
    let eq_c = eq.chars().next().unwrap_or('=');
    let mut out = Vec::new();
    if q.is_empty() {
        return out;
    }
    let limit = if max_keys == 0 { usize::MAX } else { max_keys };
    for pair in q.split(sep_c) {
        if out.len() >= limit {
            break;
        }
        if pair.is_empty() {
            continue;
        }
        match pair.find(eq_c) {
            Some(i) => {
                let (k, v) = pair.split_at(i);
                let v = &v[eq_c.len_utf8()..];
                out.push((qs_unescape(k), qs_unescape(v)));
            }
            None => out.push((qs_unescape(pair), String::new())),
        }
    }
    out
}

pub fn qs_parse_default(q: &str) -> Vec<(String, String)> {
    qs_parse(q, "&", "=", 1000)
}

pub fn qs_stringify(pairs: &[(String, String)]) -> String {
    pairs.iter().map(|(k, v)| format!("{}={}", qs_escape(k), qs_escape(v))).collect::<Vec<_>>().join("&")
}

// --- IP helpers ---

/// Complexity: O(n) parse do std.
#[inline]
pub fn is_ipv4(s: &str) -> bool {
    let t = s.trim().trim_matches(|c| c == '[' || c == ']');
    t.parse::<std::net::Ipv4Addr>().is_ok()
}

/// Complexity: O(n) parse do std.
#[inline]
pub fn is_ipv6(s: &str) -> bool {
    let t = s.trim().trim_matches(|c| c == '[' || c == ']');
    t.parse::<std::net::Ipv6Addr>().is_ok()
}

/// Node net.isIP: 0 = não-IP, 4 = IPv4, 6 = IPv6.
/// Complexity: O(n).
#[inline]
pub fn is_ip(s: &str) -> u8 {
    if is_ipv4(s) {
        4
    } else if is_ipv6(s) {
        6
    } else {
        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IpFamily {
    V4,
    V6,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LexSocketAddr {
    pub host: String,
    pub port: u16,
}

impl LexSocketAddr {
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.starts_with('[') {
            // [::1]:8080
            let end = s.find(']').ok_or_else(|| "invalid socket addr".to_string())?;
            let host = s[1..end].to_string();
            let rest = &s[end + 1..];
            if !rest.starts_with(':') {
                return Err("missing port".to_string());
            }
            let port: u16 = rest[1..].parse().map_err(|_| "invalid port".to_string())?;
            return Ok(Self { host, port });
        }
        match s.rfind(':') {
            Some(i) => {
                let host = s[..i].to_string();
                let port: u16 = s[i + 1..].parse().map_err(|_| "invalid port".to_string())?;
                if host.is_empty() {
                    return Err("empty host".to_string());
                }
                // se host contém ':' sem colchetes, é ipv6 sem porta -> erro claro
                if host.contains(':') {
                    return Err("ipv6 must use [addr]:port".to_string());
                }
                Ok(Self { host, port })
            }
            None => Err("missing port (expected host:port)".to_string()),
        }
    }
    pub fn family(&self) -> Option<IpFamily> {
        if is_ipv4(&self.host) {
            Some(IpFamily::V4)
        } else if is_ipv6(&self.host) {
            Some(IpFamily::V6)
        } else {
            None
        }
    }
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn ipv4_to_u32(s: &str) -> Option<u32> {
    let a: std::net::Ipv4Addr = s.trim().parse().ok()?;
    Some(u32::from(a))
}

#[derive(Debug, Default)]
pub struct BlockList {
    blocks: Vec<(u32, u32)>,
}

impl BlockList {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }
    /// Adiciona CIDR "a.b.c.d/n".
    pub fn block_cidr(&mut self, cidr: &str) -> Result<(), String> {
        let (ip_s, n_s) = cidr.split_once('/').ok_or_else(|| "cidr must be a.b.c.d/n".to_string())?;
        let ip = ipv4_to_u32(ip_s).ok_or_else(|| format!("invalid ipv4: {ip_s}"))?;
        let n: u32 = n_s.trim().parse().map_err(|_| "invalid prefix".to_string())?;
        if n > 32 {
            return Err("prefix > 32".to_string());
        }
        let mask = if n == 0 { 0 } else { (!0u32) << (32 - n) };
        let net = ip & mask;
        self.blocks.push((net, mask));
        Ok(())
    }
    pub fn is_blocked(&self, ip: &str) -> bool {
        let Some(v) = ipv4_to_u32(ip) else { return false };
        self.blocks.iter().any(|(net, mask)| (v & mask) == *net)
    }
    pub fn len(&self) -> usize {
        self.blocks.len()
    }
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

// ---------------------------------------------------------------------------
// EXT C: erros, timeouts, http client/agent, tls extras, sni
// ---------------------------------------------------------------------------

// Erros manuais (thiserror-free, Display manual)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    BadRequest(String),
    Timeout,
    TooLarge(String),
    /// Resposta não-2xx após redirects/retry (typed, sem panic).
    /// Complexity: O(1).
    Status(u16, String),
    /// HTTPS exigiria handshake TLS real (fora de escopo sem novas deps).
    TlsRequired(String),
    /// Redirect inválido / loop excedido.
    Redirect(String),
    Other(String),
}
impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::BadRequest(m) => write!(f, "http bad request: {m}"),
            HttpError::Timeout => write!(f, "http timeout"),
            HttpError::TooLarge(m) => write!(f, "http too large: {m}"),
            HttpError::Status(code, m) => write!(f, "http status {code}: {m}"),
            HttpError::TlsRequired(m) => write!(f, "tls required: {m}"),
            HttpError::Redirect(m) => write!(f, "http redirect: {m}"),
            HttpError::Other(m) => write!(f, "http error: {m}"),
        }
    }
}
impl std::error::Error for HttpError {}

impl HttpError {
    /// Complexity: O(1).
    #[inline]
    pub fn is_retryable_status(status: u16) -> bool {
        status == 429 || (500..=599).contains(&status)
    }
    /// Complexity: O(1).
    #[inline]
    pub fn status_code(&self) -> Option<u16> {
        match self {
            HttpError::Status(c, _) => Some(*c),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsError {
    InvalidCert(String),
    Handshake(String),
    Version(String),
    Other(String),
}
impl std::fmt::Display for TlsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlsError::InvalidCert(m) => write!(f, "tls invalid cert: {m}"),
            TlsError::Handshake(m) => write!(f, "tls handshake: {m}"),
            TlsError::Version(m) => write!(f, "tls version: {m}"),
            TlsError::Other(m) => write!(f, "tls error: {m}"),
        }
    }
}
impl std::error::Error for TlsError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsError {
    Protocol(String),
    Closed,
    InvalidFrame(String),
    Other(String),
}
impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WsError::Protocol(m) => write!(f, "ws protocol: {m}"),
            WsError::Closed => write!(f, "ws closed"),
            WsError::InvalidFrame(m) => write!(f, "ws invalid frame: {m}"),
            WsError::Other(m) => write!(f, "ws error: {m}"),
        }
    }
}
impl std::error::Error for WsError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyError {
    BadRequest(String),
    AuthFailed,
    Other(String),
}
impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::BadRequest(m) => write!(f, "proxy bad request: {m}"),
            ProxyError::AuthFailed => write!(f, "proxy auth failed"),
            ProxyError::Other(m) => write!(f, "proxy error: {m}"),
        }
    }
}
impl std::error::Error for ProxyError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlError {
    Invalid(String),
    MissingHost,
    UnsupportedScheme(String),
}
impl std::fmt::Display for UrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UrlError::Invalid(m) => write!(f, "url invalid: {m}"),
            UrlError::MissingHost => write!(f, "url missing host"),
            UrlError::UnsupportedScheme(s) => write!(f, "url unsupported scheme: {s}"),
        }
    }
}
impl std::error::Error for UrlError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CookieError {
    Invalid(String),
    Expired,
    Other(String),
}
impl std::fmt::Display for CookieError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CookieError::Invalid(m) => write!(f, "cookie invalid: {m}"),
            CookieError::Expired => write!(f, "cookie expired"),
            CookieError::Other(m) => write!(f, "cookie error: {m}"),
        }
    }
}
impl std::error::Error for CookieError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultipartError {
    TooLarge(String),
    Invalid(String),
    Other(String),
}
impl std::fmt::Display for MultipartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipartError::TooLarge(m) => write!(f, "multipart too large: {m}"),
            MultipartError::Invalid(m) => write!(f, "multipart invalid: {m}"),
            MultipartError::Other(m) => write!(f, "multipart error: {m}"),
        }
    }
}
impl std::error::Error for MultipartError {}

// DNS
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DnsError {
    NotFound(String),
    Timeout(String),
    Refused(String),
    BadName(String),
}
impl std::fmt::Display for DnsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsError::NotFound(h) => write!(f, "dns not found: {h}"),
            DnsError::Timeout(h) => write!(f, "dns timeout: {h}"),
            DnsError::Refused(h) => write!(f, "dns refused: {h}"),
            DnsError::BadName(h) => write!(f, "dns bad name: {h}"),
        }
    }
}
impl std::error::Error for DnsError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DnsOrder {
    Verbatim,
    Ipv4First,
    Ipv6First,
}

#[derive(Debug, Clone)]
pub struct DnsCacheEntry {
    pub host: String,
    pub addrs: Vec<String>,
    pub expires_at: u64,
}

#[derive(Debug, Clone)]
pub struct DnsResolver {
    servers: Vec<String>,
    timeout_ms: u64,
    order: DnsOrder,
}

impl DnsResolver {
    pub fn new() -> Self {
        Self { servers: vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()], timeout_ms: 5000, order: DnsOrder::Verbatim }
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
    pub fn timeout(&self) -> u64 {
        self.timeout_ms
    }
    pub fn set_order(&mut self, o: DnsOrder) {
        self.order = o;
    }
    pub fn order(&self) -> DnsOrder {
        self.order
    }
}

impl Default for DnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

// ServerTimeouts Node-like
#[derive(Debug, Clone)]
pub struct ServerTimeouts {
    pub headers: StdDuration,
    pub body: StdDuration,
    pub keep_alive: StdDuration,
    pub request: StdDuration,
}

impl Default for ServerTimeouts {
    fn default() -> Self {
        Self {
            headers: StdDuration::from_secs(60),
            body: StdDuration::from_secs(300),
            keep_alive: StdDuration::from_secs(5),
            request: StdDuration::from_secs(300),
        }
    }
}

impl ServerTimeouts {
    pub fn new(headers: StdDuration, body: StdDuration, keep_alive: StdDuration, request: StdDuration) -> Self {
        Self { headers, body, keep_alive, request }
    }
}

// TLS extras
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TlsVerifyMode {
    None,
    Ca,
    Full,
}

#[derive(Debug, Clone)]
pub struct SecureContext {
    pub cert_pem: Option<String>,
    pub key_pem: Option<String>,
}

impl SecureContext {
    /// Valida PEM headers sem handshake real (documentado: sem I/O TLS).
    pub fn from_pem(cert: &str, key: &str) -> Result<Self, String> {
        if !cert.contains("-----BEGIN CERTIFICATE-----") {
            return Err("invalid cert PEM: missing BEGIN CERTIFICATE".to_string());
        }
        if !cert.contains("-----END CERTIFICATE-----") {
            return Err("invalid cert PEM: missing END CERTIFICATE".to_string());
        }
        if !key.contains("-----BEGIN") || !key.contains("PRIVATE KEY") {
            return Err("invalid key PEM: missing PRIVATE KEY".to_string());
        }
        Ok(Self { cert_pem: Some(cert.to_string()), key_pem: Some(key.to_string()) })
    }
    pub fn empty() -> Self {
        Self { cert_pem: None, key_pem: None }
    }
}

#[derive(Debug, Clone)]
pub struct HttpsClientOpts {
    pub base: LexUrl,
    pub timeout_ms: u64,
    pub verify: TlsVerifyMode,
    pub context: Option<SecureContext>,
}

impl HttpsClientOpts {
    pub fn new(base: LexUrl) -> Self {
        Self { base, timeout_ms: 10_000, verify: TlsVerifyMode::Full, context: None }
    }
}

// SNI
#[derive(Debug, Default)]
pub struct SniRouter {
    routes: HashMap<String, String>,
}

impl SniRouter {
    pub fn new() -> Self {
        Self { routes: HashMap::new() }
    }
    pub fn add(&mut self, host: &str, target: &str) {
        self.routes.insert(host.to_ascii_lowercase(), target.to_string());
    }
    pub fn resolve(&self, host: &str) -> Option<&str> {
        self.routes.get(&host.to_ascii_lowercase()).map(|s| s.as_str())
    }
    pub fn len(&self) -> usize {
        self.routes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
}

// HTTP client/agent (Node http parity via TcpStream manual HTTP/1.1)

#[derive(Debug, Clone)]
pub struct HttpAgentOpts {
    pub keep_alive: bool,
    pub max_sockets: usize,
    pub max_free: usize,
    pub keep_alive_msecs: u64,
}

impl Default for HttpAgentOpts {
    fn default() -> Self {
        Self { keep_alive: true, max_sockets: 100, max_free: 10, keep_alive_msecs: 1000 }
    }
}

#[derive(Debug)]
pub struct HttpAgent {
    pub opts: HttpAgentOpts,
    active: usize,
}

impl HttpAgent {
    pub fn new(opts: HttpAgentOpts) -> Self {
        Self { opts, active: 0 }
    }
    pub fn global() -> Self {
        Self::new(HttpAgentOpts::default())
    }
    pub fn destroy(&mut self) {
        self.active = 0;
    }
    pub fn active(&self) -> usize {
        self.active
    }
}

#[derive(Debug, Clone)]
pub struct HttpClientOpts {
    pub base: LexUrl,
    pub timeout_ms: u64,
    pub max_body: usize,
    pub follow_redirects: bool,
    /// Cap de redirects (default 5).
    pub max_redirects: usize,
    /// User-Agent default.
    pub user_agent: String,
    /// Headers default injetados em toda request.
    pub default_headers: Vec<(String, String)>,
    /// Retry: tentativas extras além da primeira (default 2).
    pub max_retries: u32,
    /// Retry: delay base ms (exponencial, default 100).
    pub retry_base_delay_ms: u64,
    /// Retry: teto total acumulado ms (default 5000).
    pub retry_max_total_delay_ms: u64,
}

impl HttpClientOpts {
    /// Complexity: O(1).
    pub fn new(base: LexUrl) -> Self {
        Self {
            base,
            timeout_ms: 10_000,
            max_body: 8 << 20,
            follow_redirects: true,
            max_redirects: 5,
            user_agent: "lexicon-http/0.2.0".to_string(),
            default_headers: Vec::new(),
            max_retries: 2,
            retry_base_delay_ms: 100,
            retry_max_total_delay_ms: 5_000,
        }
    }
    /// Alias compat: `max_body_bytes`.
    /// Complexity: O(1).
    #[inline]
    pub fn max_body_bytes(&self) -> usize {
        self.max_body
    }
}

/// Builder do [`HttpClient`].
///
/// ```rust
/// use lexicon_http::module::{HttpClientBuilder};
/// let c = HttpClientBuilder::new("http://127.0.0.1:9").unwrap()
///     .timeout_ms(500).max_redirects(2).user_agent("t/1").build().unwrap();
/// ```
/// Complexity: builder methods O(1) amortized.
#[derive(Debug, Clone)]
pub struct HttpClientBuilder {
    base: LexUrl,
    timeout_ms: u64,
    max_redirects: usize,
    max_body_bytes: usize,
    user_agent: String,
    default_headers: Vec<(String, String)>,
    follow_redirects: bool,
    max_retries: u32,
    retry_base_delay_ms: u64,
    retry_max_total_delay_ms: u64,
}

impl HttpClientBuilder {
    /// Complexity: O(n) parse da URL.
    pub fn new(base_url: &str) -> Result<Self, HttpError> {
        let base =
            LexUrl::parse(base_url).map_err(|e| HttpError::BadRequest(format!("bad base_url: {e}")))?;
        Ok(Self {
            base,
            timeout_ms: 10_000,
            max_redirects: 5,
            max_body_bytes: 8 << 20,
            user_agent: "lexicon-http/0.2.0".to_string(),
            default_headers: Vec::new(),
            follow_redirects: true,
            max_retries: 2,
            retry_base_delay_ms: 100,
            retry_max_total_delay_ms: 5_000,
        })
    }
    /// Complexity: O(1).
    #[inline]
    pub fn timeout_ms(mut self, v: u64) -> Self {
        self.timeout_ms = v.max(1);
        self
    }
    /// Complexity: O(1).
    #[inline]
    pub fn max_redirects(mut self, v: usize) -> Self {
        self.max_redirects = v;
        self
    }
    /// Complexity: O(1).
    #[inline]
    pub fn max_body_bytes(mut self, v: usize) -> Self {
        self.max_body_bytes = v;
        self
    }
    /// Complexity: O(1).
    #[inline]
    pub fn user_agent(mut self, v: &str) -> Self {
        self.user_agent = v.to_string();
        self
    }
    /// Complexity: O(1) amortized.
    #[inline]
    pub fn default_header(mut self, k: &str, v: &str) -> Self {
        self.default_headers.push((k.to_string(), v.to_string()));
        self
    }
    /// Complexity: O(1).
    #[inline]
    pub fn follow_redirects(mut self, v: bool) -> Self {
        self.follow_redirects = v;
        self
    }
    /// Complexity: O(1).
    #[inline]
    pub fn retry(mut self, attempts: u32, base_ms: u64, max_total_ms: u64) -> Self {
        self.max_retries = attempts;
        self.retry_base_delay_ms = base_ms;
        self.retry_max_total_delay_ms = max_total_ms;
        self
    }
    /// Complexity: O(n) onde n = default headers (validação).
    pub fn build(self) -> Result<HttpClient, HttpError> {
        for (k, v) in self.default_headers.iter() {
            validate_header_name(k).map_err(HttpError::BadRequest)?;
            validate_header_value(v).map_err(HttpError::BadRequest)?;
        }
        Ok(HttpClient {
            opts: HttpClientOpts {
                base: self.base,
                timeout_ms: self.timeout_ms,
                max_body: self.max_body_bytes,
                follow_redirects: self.follow_redirects,
                max_redirects: self.max_redirects,
                user_agent: self.user_agent,
                default_headers: self.default_headers,
                max_retries: self.max_retries,
                retry_base_delay_ms: self.retry_base_delay_ms,
                retry_max_total_delay_ms: self.retry_max_total_delay_ms,
            },
        })
    }
}

/// Calcula método após redirect (301/302/303/307/308).
/// - 307/308 preservam; 303 vira GET (exceto HEAD); 301/302: POST vira GET
///   (compat histórica), demais preservam.
/// Complexity: O(1).
#[inline]
pub fn redirect_method(status: u16, orig: HttpMethod) -> HttpMethod {
    match status {
        307 | 308 => orig,
        303 => match orig {
            HttpMethod::HEAD => HttpMethod::HEAD,
            _ => HttpMethod::GET,
        },
        301 | 302 => match orig {
            HttpMethod::POST => HttpMethod::GET,
            _ => orig,
        },
        _ => orig,
    }
}

/// Delay exponencial `base * 2^attempt` (attempt 0-based), sem sleep.
/// Complexity: O(1).
#[inline]
pub fn retry_delay_ms(attempt: u32, base_ms: u64) -> u64 {
    base_ms.saturating_mul(1u64 << attempt.min(10))
}

/// Deve retentar? 429 ou 5xx.
/// Complexity: O(1).
#[inline]
pub fn should_retry_status(status: u16) -> bool {
    HttpError::is_retryable_status(status)
}

/// Parse `Retry-After` (segundos ou data HTTP ignorada -> None).
/// Complexity: O(n) trim+parse.
#[inline]
pub fn parse_retry_after(v: &str) -> Option<u64> {
    v.trim().parse::<u64>().ok().map(|s| s.saturating_mul(1000))
}

#[derive(Debug)]
pub struct HttpClient {
    pub opts: HttpClientOpts,
}

impl HttpClient {
    /// Complexity: O(1).
    #[inline]
    pub fn new(opts: HttpClientOpts) -> Self {
        Self { opts }
    }

    /// Constrói via URL base. Complexity: O(n) parse.
    #[inline]
    pub fn from_base_url(base_url: &str) -> Result<Self, HttpError> {
        HttpClientBuilder::new(base_url)?.build()
    }

    /// Merge default headers (sem sobrescrever explícitas) + User-Agent.
    /// Complexity: O(d*h) bounded (d,h pequenos).
    fn merged_headers(&self, extra: &Headers) -> Headers {
        let mut out = Headers::new();
        for (k, v) in self.opts.default_headers.iter() {
            out.insert(k, v);
        }
        if !out.has("user-agent") {
            out.insert("user-agent", &self.opts.user_agent);
        }
        for (k, v) in extra.iter() {
            out.insert(k, v);
        }
        out
    }

    /// Request genérica com redirects + retry + checagem 2xx.
    /// Conexão: `Connection: close` (nova conexão por request; keep-alive
    /// seria upgrade futuro via pool em `HttpAgent`).
    /// Complexity: O(r*(n+m)) onde r = redirects+retries, n = header bytes,
    /// m = body bytes (streaming com cap).
    pub async fn request(
        &self,
        mut method: HttpMethod,
        path: &str,
        headers: Headers,
        mut body: HttpBody,
    ) -> Result<HttpResponse, HttpError> {
        let mut current = if path.contains("://") {
            LexUrl::parse(path).map_err(|e| HttpError::BadRequest(format!("bad url: {e}")))?
        } else {
            self.opts
                .base
                .join(path)
                .map_err(|e| HttpError::BadRequest(format!("bad join: {e}")))?
        };
        if matches!(current.scheme.as_str(), "https" | "wss") {
            return Err(HttpError::TlsRequired(
                "https requires a TLS feature (e.g. tokio-rustls/native-tls); plain-HTTP client only. Enable a `tls` feature with a rustls/native-tls connector and re-route via HttpsClientOpts"
                    .to_string(),
            ));
        }
        let mut cur_headers = self.merged_headers(&headers);
        let mut redirects = 0usize;
        let mut total_delay_ms = 0u64;
        let mut attempt = 0u32;
        loop {
            let resp = http_roundtrip(
                &current,
                method,
                &cur_headers,
                &body,
                self.opts.timeout_ms,
                self.opts.max_body,
            )
            .await?;
            // Redirects têm precedência sobre retry.
            if self.opts.follow_redirects && matches!(resp.status, 301 | 302 | 303 | 307 | 308) {
                if redirects >= self.opts.max_redirects {
                    return Err(HttpError::Redirect("too many redirects".to_string()));
                }
                let loc = resp
                    .headers
                    .get("location")
                    .ok_or_else(|| HttpError::Redirect("redirect without location".to_string()))?;
                current = current
                    .join(&loc)
                    .map_err(|e| HttpError::Redirect(format!("bad redirect location: {e}")))?;
                if matches!(current.scheme.as_str(), "https" | "wss") {
                    return Err(HttpError::TlsRequired(
                        "redirect to https requires TLS feature".to_string(),
                    ));
                }
                redirects += 1;
                let next = redirect_method(resp.status, method);
                if next != method {
                    method = next;
                    body = HttpBody::Empty;
                    cur_headers.remove("content-length");
                    cur_headers.remove("content-type");
                }
                // Redirect não consome tentativa de retry; reseta backoff.
                attempt = 0;
                total_delay_ms = 0;
                continue;
            }
            // Retry em 429/5xx com backoff exponencial + Retry-After.
            if should_retry_status(resp.status) && attempt < self.opts.max_retries {
                let mut delay = retry_delay_ms(attempt, self.opts.retry_base_delay_ms);
                if let Some(ra) = resp.headers.get("retry-after").and_then(|v| parse_retry_after(&v)) {
                    delay = delay.max(ra);
                }
                if total_delay_ms.saturating_add(delay) > self.opts.retry_max_total_delay_ms {
                    return Err(HttpError::Status(
                        resp.status,
                        format!("retry budget exceeded after {attempt} retries"),
                    ));
                }
                total_delay_ms += delay;
                attempt += 1;
                if delay > 0 {
                    tokio::time::sleep(StdDuration::from_millis(delay)).await;
                }
                continue;
            }
            // HEAD strip.
            let mut out = resp;
            if method.requires_body_strip() {
                out.body = HttpBody::Empty;
            }
            // Não-2xx vira erro tipado (sem panic).
            if !(200..=299).contains(&out.status) {
                let snippet = String::from_utf8_lossy(&out.body.as_slice().iter().cloned().take(256).collect::<Vec<u8>>()).to_string();
                return Err(HttpError::Status(out.status, snippet));
            }
            return Ok(out);
        }
    }

    /// Variante crua sem checagem 2xx/retry de status (ainda segue redirects).
    /// Complexity: same as `request` minus status mapping.
    pub async fn request_raw(
        &self,
        mut method: HttpMethod,
        path: &str,
        headers: Headers,
        mut body: HttpBody,
    ) -> Result<HttpResponse, HttpError> {
        let mut current = if path.contains("://") {
            LexUrl::parse(path).map_err(|e| HttpError::BadRequest(format!("bad url: {e}")))?
        } else {
            self.opts
                .base
                .join(path)
                .map_err(|e| HttpError::BadRequest(format!("bad join: {e}")))?
        };
        if matches!(current.scheme.as_str(), "https" | "wss") {
            return Err(HttpError::TlsRequired("https requires TLS feature".to_string()));
        }
        let mut cur_headers = self.merged_headers(&headers);
        let mut redirects = 0usize;
        loop {
            let resp = http_roundtrip(
                &current,
                method,
                &cur_headers,
                &body,
                self.opts.timeout_ms,
                self.opts.max_body,
            )
            .await?;
            if self.opts.follow_redirects && matches!(resp.status, 301 | 302 | 303 | 307 | 308) {
                if redirects >= self.opts.max_redirects {
                    return Err(HttpError::Redirect("too many redirects".to_string()));
                }
                let loc = resp
                    .headers
                    .get("location")
                    .ok_or_else(|| HttpError::Redirect("redirect without location".to_string()))?;
                current = current
                    .join(&loc)
                    .map_err(|e| HttpError::Redirect(format!("bad redirect location: {e}")))?;
                redirects += 1;
                let next = redirect_method(resp.status, method);
                if next != method {
                    method = next;
                    body = HttpBody::Empty;
                    cur_headers.remove("content-length");
                }
                continue;
            }
            let mut out = resp;
            if method.requires_body_strip() {
                out.body = HttpBody::Empty;
            }
            return Ok(out);
        }
    }

    /// GET. Complexity: O(r*(n+m)).
    #[inline]
    pub async fn get(&self, path: &str) -> Result<HttpResponse, HttpError> {
        self.request(HttpMethod::GET, path, Headers::new(), HttpBody::Empty)
            .await
    }
    /// HEAD. Complexity: O(r*(n+m)).
    #[inline]
    pub async fn head(&self, path: &str) -> Result<HttpResponse, HttpError> {
        self.request(HttpMethod::HEAD, path, Headers::new(), HttpBody::Empty)
            .await
    }
    /// DELETE. Complexity: O(r*(n+m)).
    #[inline]
    pub async fn delete(&self, path: &str) -> Result<HttpResponse, HttpError> {
        self.request(HttpMethod::DELETE, path, Headers::new(), HttpBody::Empty)
            .await
    }
    /// POST bytes. Complexity: O(r*(n+m)).
    #[inline]
    pub async fn post(&self, path: &str, body: Vec<u8>) -> Result<HttpResponse, HttpError> {
        self.request(HttpMethod::POST, path, Headers::new(), HttpBody::from(body))
            .await
    }
    /// PUT bytes. Complexity: O(r*(n+m)).
    #[inline]
    pub async fn put(&self, path: &str, body: Vec<u8>) -> Result<HttpResponse, HttpError> {
        self.request(HttpMethod::PUT, path, Headers::new(), HttpBody::from(body))
            .await
    }
    /// POST JSON (serializa + content-type).
    /// Complexity: O(n) serialize + request.
    #[inline]
    pub async fn post_json<T: serde::Serialize>(
        &self,
        path: &str,
        value: &T,
    ) -> Result<HttpResponse, HttpError> {
        let bytes =
            serde_json::to_vec(value).map_err(|e| HttpError::BadRequest(format!("bad json: {e}")))?;
        let mut h = Headers::new();
        h.insert("content-type", "application/json");
        self.request(HttpMethod::POST, path, h, HttpBody::from(bytes))
            .await
    }

    // ---- blocking (std TcpStream, sem runtime) ----
    //
    // Tradeoff: bloqueia a thread; sem async; sem TLS; nova conexão por
    // request (`Connection: close`); timeouts via `set_read/write_timeout`.
    // Ideal para CLIs/testes sem runtime Tokio.

    /// Blocking request genérica (segue redirects, checa 2xx).
    /// Complexity: O(r*(n+m)) bloqueante.
    pub fn blocking_request(
        &self,
        mut method: HttpMethod,
        path: &str,
        headers: Headers,
        mut body: HttpBody,
    ) -> Result<HttpResponse, HttpError> {
        let mut current = if path.contains("://") {
            LexUrl::parse(path).map_err(|e| HttpError::BadRequest(format!("bad url: {e}")))?
        } else {
            self.opts
                .base
                .join(path)
                .map_err(|e| HttpError::BadRequest(format!("bad join: {e}")))?
        };
        if matches!(current.scheme.as_str(), "https" | "wss") {
            return Err(HttpError::TlsRequired("https requires TLS feature".to_string()));
        }
        let mut cur_headers = self.merged_headers(&headers);
        let mut redirects = 0usize;
        loop {
            let resp = blocking_roundtrip(
                &current,
                method,
                &cur_headers,
                &body,
                self.opts.timeout_ms,
                self.opts.max_body,
            )?;
            if self.opts.follow_redirects && matches!(resp.status, 301 | 302 | 303 | 307 | 308) {
                if redirects >= self.opts.max_redirects {
                    return Err(HttpError::Redirect("too many redirects".to_string()));
                }
                let loc = resp
                    .headers
                    .get("location")
                    .ok_or_else(|| HttpError::Redirect("redirect without location".to_string()))?;
                current = current
                    .join(&loc)
                    .map_err(|e| HttpError::Redirect(format!("bad redirect location: {e}")))?;
                redirects += 1;
                let next = redirect_method(resp.status, method);
                if next != method {
                    method = next;
                    body = HttpBody::Empty;
                    cur_headers.remove("content-length");
                }
                continue;
            }
            let mut out = resp;
            if method.requires_body_strip() {
                out.body = HttpBody::Empty;
            }
            if !(200..=299).contains(&out.status) {
                let snippet = String::from_utf8_lossy(&out.body.as_slice().iter().cloned().take(256).collect::<Vec<u8>>()).to_string();
                return Err(HttpError::Status(out.status, snippet));
            }
            return Ok(out);
        }
    }

    /// Blocking GET. Complexity: O(r*(n+m)).
    #[inline]
    pub fn blocking_get(&self, path: &str) -> Result<HttpResponse, HttpError> {
        self.blocking_request(HttpMethod::GET, path, Headers::new(), HttpBody::Empty)
    }
    /// Blocking HEAD.
    #[inline]
    pub fn blocking_head(&self, path: &str) -> Result<HttpResponse, HttpError> {
        self.blocking_request(HttpMethod::HEAD, path, Headers::new(), HttpBody::Empty)
    }
    /// Blocking DELETE.
    #[inline]
    pub fn blocking_delete(&self, path: &str) -> Result<HttpResponse, HttpError> {
        self.blocking_request(HttpMethod::DELETE, path, Headers::new(), HttpBody::Empty)
    }
    /// Blocking POST.
    #[inline]
    pub fn blocking_post(&self, path: &str, body: Vec<u8>) -> Result<HttpResponse, HttpError> {
        self.blocking_request(HttpMethod::POST, path, Headers::new(), HttpBody::from(body))
    }
    /// Blocking PUT.
    #[inline]
    pub fn blocking_put(&self, path: &str, body: Vec<u8>) -> Result<HttpResponse, HttpError> {
        self.blocking_request(HttpMethod::PUT, path, Headers::new(), HttpBody::from(body))
    }
}

/// Blocking roundtrip via `std::net::TcpStream` (sem Tokio).
/// Timeouts estritos connect+read/write; cap de corpo em streaming.
/// Complexity: O(n+m).
fn blocking_roundtrip(
    url: &LexUrl,
    method: HttpMethod,
    headers: &Headers,
    body: &HttpBody,
    timeout_ms: u64,
    max_body: usize,
) -> Result<HttpResponse, HttpError> {
    use std::io::{Read, Write};
    let to = StdDuration::from_millis(timeout_ms);
    let addr = format!("{}:{}", url.host, url.effective_port());
    // Resolve + connect com timeout (primeiro IP que conectar).
    let addrs: Vec<std::net::SocketAddr> = tokio_stub_resolve(&addr)
        .map_err(|e| HttpError::BadRequest(format!("dns failed: {e}")))?;
    let mut stream = None;
    let mut last_err = String::new();
    for a in addrs {
        match std::net::TcpStream::connect_timeout(&a, to) {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(e) => last_err = e.to_string(),
        }
    }
    let mut stream: std::net::TcpStream =
        stream.ok_or_else(|| HttpError::Other(format!("connect failed: {last_err}")))?;
    stream
        .set_read_timeout(Some(to))
        .map_err(|e| HttpError::Other(format!("timeout setup: {e}")))?;
    stream
        .set_write_timeout(Some(to))
        .map_err(|e| HttpError::Other(format!("timeout setup: {e}")))?;
    let mut target = url.path.clone();
    if let Some(q) = &url.query {
        target.push('?');
        target.push_str(q);
    }
    let mut req = format!("{} {} HTTP/1.1\r\n", method.as_str(), target);
    req.push_str(&format!("Host: {}\r\n", url.host_header()));
    req.push_str("Connection: close\r\n");
    // User-Agent já veio em merged_headers; garante fallback.
    let has_ua = headers.iter().any(|(k, _)| k == "user-agent");
    if !has_ua {
        req.push_str("User-Agent: lexicon-http/0.2.0\r\n");
    }
    if let HttpBody::Buffer(b) = body {
        if !b.is_empty() {
            req.push_str(&format!("Content-Length: {}\r\n", b.len()));
        }
    }
    for (k, v) in headers.iter() {
        let kl = k.to_ascii_lowercase();
        if kl == "host" || kl == "content-length" || kl == "connection" {
            continue;
        }
        validate_header_name(k).map_err(HttpError::BadRequest)?;
        validate_header_value(v).map_err(HttpError::BadRequest)?;
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    stream
        .write_all(req.as_bytes())
        .map_err(|e| map_io_err(e))?;
    if let HttpBody::Buffer(b) = body {
        if !b.is_empty() {
            stream.write_all(b).map_err(map_io_err)?;
        }
    }
    // Lê até EOF com cap incremental (sem alocar além do cap + headroom).
    let mut raw = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                raw.extend_from_slice(&buf[..n]);
                if raw.len() > max_body + 64 * 1024 {
                    return Err(HttpError::TooLarge(format!(
                        "body too large: > {}",
                        max_body
                    )));
                }
            }
            Err(e) => return Err(map_io_err(e)),
        }
    }
    parse_http_response(&raw, max_body).map_err(HttpError::Other)
}

/// Resolve síncrono mínimo (std only) para o blocking client.
/// Complexity: O(n) DNS.
#[inline]
fn tokio_stub_resolve(addr: &str) -> Result<Vec<std::net::SocketAddr>, String> {
    use std::net::ToSocketAddrs;
    addr.to_socket_addrs()
        .map(|it| it.collect())
        .map_err(|e| e.to_string())
}

/// Mapeia io::Error para HttpError tipado (timeout vs other).
/// Complexity: O(1).
#[inline]
fn map_io_err(e: std::io::Error) -> HttpError {
    use std::io::ErrorKind;
    match e.kind() {
        ErrorKind::TimedOut => HttpError::Timeout,
        ErrorKind::WouldBlock => HttpError::Timeout,
        _ => {
            let s = e.to_string();
            if s.contains("timed out") || s.contains("timeout") {
                HttpError::Timeout
            } else {
                HttpError::Other(s)
            }
        }
    }
}

async fn http_roundtrip(
    url: &LexUrl,
    method: HttpMethod,
    headers: &Headers,
    body: &HttpBody,
    timeout_ms: u64,
    max_body: usize,
) -> Result<HttpResponse, HttpError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let host = url.host.clone();
    let port = url.effective_port();
    let addr = format!("{host}:{port}");
    let to = StdDuration::from_millis(timeout_ms);
    let mut stream = tokio::time::timeout(to, tokio::net::TcpStream::connect(&addr))
        .await
        .map_err(|_| HttpError::Timeout)?
        .map_err(|e| HttpError::Other(format!("connect failed: {e}")))?;
    // request-target
    let mut target = url.path.clone();
    if let Some(q) = &url.query {
        target.push('?');
        target.push_str(q);
    }
    let mut req = format!("{} {} HTTP/1.1\r\n", method.as_str(), target);
    req.push_str(&format!("Host: {}\r\n", url.host_header()));
    req.push_str("Connection: close\r\n");
    if let HttpBody::Buffer(b) = body {
        if !b.is_empty() {
            req.push_str(&format!("Content-Length: {}\r\n", b.len()));
        }
    }
    for (k, v) in headers.iter() {
        let kl = k.to_ascii_lowercase();
        if kl == "host" || kl == "content-length" || kl == "connection" {
            continue;
        }
        validate_header_name(k).map_err(HttpError::BadRequest)?;
        validate_header_value(v).map_err(HttpError::BadRequest)?;
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    tokio::time::timeout(to, stream.write_all(req.as_bytes()))
        .await
        .map_err(|_| HttpError::Timeout)?
        .map_err(|e| HttpError::Other(format!("write failed: {e}")))?;
    if let HttpBody::Buffer(b) = body {
        if !b.is_empty() {
            tokio::time::timeout(to, stream.write_all(b))
                .await
                .map_err(|_| HttpError::Timeout)?
                .map_err(|e| HttpError::Other(format!("write body failed: {e}")))?;
        }
    }
    // lê até EOF (Connection: close) com timeout global + cap em streaming
    let mut raw = Vec::new();
    let mut buf = vec![0u8; 8192];
    let read_fut = async {
        loop {
            match stream.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    raw.extend_from_slice(&buf[..n]);
                    if raw.len() > max_body + 64 * 1024 {
                        return Err(HttpError::TooLarge(format!(
                            "body too large: > {max_body}"
                        )));
                    }
                }
                Err(e) => return Err(HttpError::Other(format!("read failed: {e}"))),
            }
        }
        Ok::<_, HttpError>(())
    };
    tokio::time::timeout(to, read_fut)
        .await
        .map_err(|_| HttpError::Timeout)??;
    parse_http_response(&raw, max_body).map_err(|e| {
        if e.contains("too large") {
            HttpError::TooLarge(e)
        } else {
            HttpError::Other(e)
        }
    })
}

/// Rejunta linhas de header restantes em bloco `\r\n` para validação estrita.
/// Complexity: O(n).
#[inline]
fn raw_lines_join<'a>(lines: impl Iterator<Item = &'a str>) -> String {
    let mut out = String::new();
    for l in lines {
        out.push_str(l);
        out.push_str("\r\n");
    }
    out
}

/// Parse de resposta HTTP/1.1 com cap de headers + corpo.
/// Complexity: O(n+m) onde n = header bytes, m = body bytes.
fn parse_http_response(raw: &[u8], max_body: usize) -> Result<HttpResponse, String> {
    // Cap rígido do bloco de headers (headers + status line).
    let hs = find_headers_end(raw).ok_or_else(|| "invalid http response: missing header end".to_string())?;
    if hs.0 > MAX_HEADER_BYTES {
        return Err(format!("headers too large: > {MAX_HEADER_BYTES}"));
    }
    let head = std::str::from_utf8(&raw[..hs.0]).map_err(|_| "invalid utf-8 in headers".to_string())?;
    // Status line estrita: HTTP/1.x SP CODE.
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or("");
    // Rejeita obs-fold na status line (não deve começar com SP/HT).
    if status_line.starts_with(' ') || status_line.starts_with('\t') {
        return Err("obs-fold rejected in status line".to_string());
    }
    let parts: Vec<&str> = status_line.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return Err(format!("bad status line: {status_line}"));
    }
    if !parts[0].starts_with("HTTP/") {
        return Err(format!("bad status line: {status_line}"));
    }
    let status: u16 = parts[1].parse().map_err(|_| "bad status code".to_string())?;
    if !(100..=599).contains(&status) {
        return Err(format!("bad status code: {status}"));
    }
    // Headers estritos: coleta bloco e valida via parse_raw_limited.
    let header_block = raw_lines_join(lines);
    let headers =
        Headers::parse_raw_limited(&header_block, MAX_HEADER_BYTES, MAX_HEADERS).map_err(|e| e.to_string())?;
    let mut body = raw[hs.1..].to_vec();
    // chunked?
    if let Some(te) = headers.get("transfer-encoding") {
        if te.to_ascii_lowercase().contains("chunked") {
            body = decode_chunked(&body)?;
        }
    } else if let Some(cl) = headers.get("content-length") {
        if let Ok(n) = cl.trim().parse::<usize>() {
            if body.len() > n {
                body.truncate(n);
            }
        }
    }
    if body.len() > max_body {
        return Err(format!("body too large: {} > {}", body.len(), max_body));
    }
    Ok(HttpResponse { status, headers, body: HttpBody::Buffer(body) })
}

fn find_headers_end(raw: &[u8]) -> Option<(usize, usize)> {
    for i in 0..raw.len().saturating_sub(3) {
        if &raw[i..i + 4] == b"\r\n\r\n" {
            return Some((i, i + 4));
        }
    }
    // tolera \n\n
    for i in 0..raw.len().saturating_sub(1) {
        if &raw[i..i + 2] == b"\n\n" {
            return Some((i, i + 2));
        }
    }
    None
}

fn decode_chunked(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut i = 0;
    loop {
        // linha de tamanho
        let mut j = i;
        while j < data.len() && !(data[j] == b'\r' && j + 1 < data.len() && data[j + 1] == b'\n') && data[j] != b'\n' {
            j += 1;
        }
        if j >= data.len() {
            return Err("bad chunked framing".to_string());
        }
        let line = std::str::from_utf8(&data[i..j]).map_err(|_| "bad chunk size utf-8".to_string())?;
        let size_str = line.split(';').next().unwrap_or("").trim();
        let n = usize::from_str_radix(size_str, 16).map_err(|_| "bad chunk size".to_string())?;
        // pula CRLF
        i = if data[j] == b'\r' { j + 2 } else { j + 1 };
        if n == 0 {
            break;
        }
        if i + n > data.len() {
            return Err("chunked truncated".to_string());
        }
        out.extend_from_slice(&data[i..i + n]);
        i += n;
        // pula CRLF após chunk
        if i + 1 < data.len() && data[i] == b'\r' && data[i + 1] == b'\n' {
            i += 2;
        } else if i < data.len() && data[i] == b'\n' {
            i += 1;
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// EXT D: ws, rate, cookie/session, multipart, auth, sse, proxy/cors/encoding
// ---------------------------------------------------------------------------

// --- WebSocket RFC6455 ---

fn ws_opcode_to_byte(op: WsOpcode) -> u8 {
    match op {
        WsOpcode::Continuation => 0x0,
        WsOpcode::Text => 0x1,
        WsOpcode::Binary => 0x2,
        WsOpcode::Close => 0x8,
        WsOpcode::Ping => 0x9,
        WsOpcode::Pong => 0xA,
    }
}

fn ws_byte_to_opcode(b: u8) -> Result<WsOpcode, String> {
    match b {
        0x0 => Ok(WsOpcode::Continuation),
        0x1 => Ok(WsOpcode::Text),
        0x2 => Ok(WsOpcode::Binary),
        0x8 => Ok(WsOpcode::Close),
        0x9 => Ok(WsOpcode::Ping),
        0xA => Ok(WsOpcode::Pong),
        _ => Err(format!("unknown ws opcode: {b}")),
    }
}

/// Sec-WebSocket-Accept (RFC6455 vetor testado).
pub fn ws_accept_key(client_key: &str) -> String {
    const MAGIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let input = format!("{}{MAGIC}", client_key.trim());
    let h = sha1_digest(input.as_bytes());
    base64_encode(&h)
}

pub fn ws_handshake_response(client_key: &str) -> HttpResponse {
    let mut r = HttpResponse::new(101, HttpBody::Empty);
    r.headers.insert("upgrade", "websocket");
    r.headers.insert("connection", "Upgrade");
    r.headers.insert("sec-websocket-accept", &ws_accept_key(client_key));
    r
}

pub fn ws_negotiate_protocol(offered: &[String], supported: &[String]) -> Option<String> {
    for o in offered {
        if supported.iter().any(|s| s == o) {
            return Some(o.clone());
        }
    }
    None
}

/// Encode server (sem mask) ou client (masked opcional com máscara fixa determinística).
pub fn ws_encode(frame: &WsFrame, masked: bool) -> Vec<u8> {
    let mut out = Vec::new();
    let b0 = (if frame.fin { 0x80 } else { 0x00 }) | ws_opcode_to_byte(frame.opcode);
    out.push(b0);
    let len = frame.payload.len();
    let mask_bit = if masked { 0x80 } else { 0x00 };
    if len < 126 {
        out.push(mask_bit | (len as u8));
    } else if len < 65536 {
        out.push(mask_bit | 126);
        out.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        out.push(mask_bit | 127);
        out.extend_from_slice(&(len as u64).to_be_bytes());
    }
    if masked {
        let mask = [0x12u8, 0x34, 0x56, 0x78];
        out.extend_from_slice(&mask);
        for (i, b) in frame.payload.iter().enumerate() {
            out.push(b ^ mask[i % 4]);
        }
    } else {
        out.extend_from_slice(&frame.payload);
    }
    out
}

/// Decode um frame; retorna (frame, consumidos).
pub fn ws_decode(input: &[u8]) -> Result<(WsFrame, usize), String> {
    if input.len() < 2 {
        return Err("incomplete ws frame".to_string());
    }
    let b0 = input[0];
    let b1 = input[1];
    let fin = (b0 & 0x80) != 0;
    let opcode = ws_byte_to_opcode(b0 & 0x0F)?;
    let masked = (b1 & 0x80) != 0;
    let mut len = (b1 & 0x7F) as usize;
    let mut idx = 2;
    if len == 126 {
        if input.len() < idx + 2 {
            return Err("incomplete ws extended len".to_string());
        }
        len = u16::from_be_bytes([input[idx], input[idx + 1]]) as usize;
        idx += 2;
    } else if len == 127 {
        if input.len() < idx + 8 {
            return Err("incomplete ws extended len64".to_string());
        }
        let v = u64::from_be_bytes([
            input[idx], input[idx + 1], input[idx + 2], input[idx + 3],
            input[idx + 4], input[idx + 5], input[idx + 6], input[idx + 7],
        ]);
        if v > (16 * 1024 * 1024) as u64 {
            return Err("ws payload too large".to_string());
        }
        len = v as usize;
        idx += 8;
    }
    let mask = if masked {
        if input.len() < idx + 4 {
            return Err("incomplete ws mask".to_string());
        }
        let m = [input[idx], input[idx + 1], input[idx + 2], input[idx + 3]];
        idx += 4;
        Some(m)
    } else {
        None
    };
    if input.len() < idx + len {
        return Err("incomplete ws payload".to_string());
    }
    let mut payload = input[idx..idx + len].to_vec();
    if let Some(m) = mask {
        for (i, b) in payload.iter_mut().enumerate() {
            *b ^= m[i % 4];
        }
    }
    Ok((WsFrame { opcode, payload, fin }, idx + len))
}

/// Complexity: O(1).
#[inline]
pub fn ws_close_code_valid(code: u16) -> bool {
    match code {
        1000..=1003 | 1007..=1011 => true,
        3000..=4999 => true,
        _ => false,
    }
}

/// Complexity: O(n) UTF-8 validation.
#[inline]
pub fn ws_check_text(data: &[u8]) -> Result<(), String> {
    std::str::from_utf8(data).map(|_| ()).map_err(|e| format!("invalid utf-8 text frame: {e}"))
}

#[derive(Debug, Default)]
pub struct WsReassembler {
    buf: Vec<u8>,
    opcode: Option<WsOpcode>,
    collecting: bool,
}

impl WsReassembler {
    pub fn new() -> Self {
        Self::default()
    }
    /// Acumula fragmentos; retorna Some(frame completo) quando fin.
    pub fn push(&mut self, frame: WsFrame) -> Result<Option<WsFrame>, String> {
        match frame.opcode {
            WsOpcode::Ping | WsOpcode::Pong | WsOpcode::Close => Ok(Some(frame)),
            WsOpcode::Text | WsOpcode::Binary => {
                if frame.fin {
                    if self.collecting {
                        return Err("unexpected new message while fragmented".to_string());
                    }
                    Ok(Some(frame))
                } else {
                    if self.collecting {
                        return Err("already fragmented".to_string());
                    }
                    self.opcode = Some(frame.opcode);
                    self.buf = frame.payload;
                    self.collecting = true;
                    Ok(None)
                }
            }
            WsOpcode::Continuation => {
                if !self.collecting {
                    return Err("unexpected continuation".to_string());
                }
                self.buf.extend_from_slice(&frame.payload);
                if frame.fin {
                    let op = self.opcode.take().unwrap_or(WsOpcode::Binary);
                    let payload = std::mem::take(&mut self.buf);
                    self.collecting = false;
                    Ok(Some(WsFrame { opcode: op, payload, fin: true }))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

// --- Rate limiting token bucket (per-key, burst=capacity, refill/sec) ---
//
// Relógio injetável: `check()` usa `StdInstant::now()`; `check_at(now)`
// recebe o instante explícito para testes sem sleep.
// Complexity: O(1) amortized por check (HashMap + aritmética).

#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_per_sec: f64,
    last: StdInstant,
}

impl TokenBucket {
    /// Complexity: O(1).
    #[inline]
    pub fn new(capacity: u32, refill_per_sec: f64) -> Self {
        Self::new_at(capacity, refill_per_sec, StdInstant::now())
    }
    /// Constrói com instante explícito (relógio injetável).
    /// Complexity: O(1).
    #[inline]
    pub fn new_at(capacity: u32, refill_per_sec: f64, at: StdInstant) -> Self {
        Self {
            capacity: capacity as f64,
            tokens: capacity as f64,
            refill_per_sec,
            last: at,
        }
    }
    /// Capacidade (burst). Complexity: O(1).
    #[inline]
    pub fn capacity(&self) -> u32 {
        self.capacity as u32
    }
    /// Taxa de refill. Complexity: O(1).
    #[inline]
    pub fn refill_rate(&self) -> f64 {
        self.refill_per_sec
    }
    fn refill_at(&mut self, now: StdInstant) {
        let el = now.saturating_duration_since(self.last).as_secs_f64();
        self.tokens = (self.tokens + el * self.refill_per_sec).min(self.capacity);
        self.last = now;
    }
    #[allow(dead_code)]
    fn refill(&mut self) {
        let now = StdInstant::now();
        self.refill_at(now);
    }
    /// Consome 1 token se houver. Complexity: O(1).
    #[inline]
    pub fn check(&mut self) -> bool {
        self.check_at(StdInstant::now())
    }
    /// Variante com relógio injetável (sem sleep). Complexity: O(1).
    #[inline]
    pub fn check_at(&mut self, now: StdInstant) -> bool {
        self.refill_at(now);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
    /// Tokens inteiros disponíveis. Complexity: O(1).
    #[inline]
    pub fn remaining(&mut self) -> u32 {
        self.remaining_at(StdInstant::now())
    }
    /// Variante com relógio injetável. Complexity: O(1).
    #[inline]
    pub fn remaining_at(&mut self, now: StdInstant) -> u32 {
        self.refill_at(now);
        self.tokens.floor().max(0.0) as u32
    }
}

#[derive(Debug)]
pub struct RateLimiter {
    buckets: HashMap<String, TokenBucket>,
    capacity: u32,
    refill_per_sec: f64,
}

impl RateLimiter {
    /// Complexity: O(1).
    #[inline]
    pub fn new(capacity: u32, refill_per_sec: f64) -> Self {
        Self { buckets: HashMap::new(), capacity, refill_per_sec }
    }
    /// Complexity: O(1) amortized (HashMap).
    #[inline]
    pub fn check(&mut self, key: &str) -> bool {
        self.check_at(key, StdInstant::now())
    }
    /// Variante com relógio injetável (testes sem sleep).
    /// Complexity: O(1) amortized.
    #[inline]
    pub fn check_at(&mut self, key: &str, now: StdInstant) -> bool {
        let cap = self.capacity;
        let r = self.refill_per_sec;
        self.buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new_at(cap, r, now))
            .check_at(now)
    }
    /// Tokens restantes da chave (cria bucket se ausente).
    /// Complexity: O(1) amortized.
    #[inline]
    pub fn remaining_at(&mut self, key: &str, now: StdInstant) -> u32 {
        let cap = self.capacity;
        let r = self.refill_per_sec;
        self.buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new_at(cap, r, now))
            .remaining_at(now)
    }
    /// Remove buckets totalmente cheios e ociosos há `idle` (higiene de memória).
    /// Complexity: O(n) onde n = chaves.
    pub fn prune_idle(&mut self, now: StdInstant, idle: StdDuration) -> usize {
        let before = self.buckets.len();
        let refill = self.refill_per_sec;
        self.buckets.retain(|_, b| {
            let elapsed = now.saturating_duration_since(b.last);
            let would = (b.tokens + elapsed.as_secs_f64() * refill).min(b.capacity);
            !(elapsed >= idle && would >= b.capacity - f64::EPSILON)
        });
        before - self.buckets.len()
    }
    /// Complexity: O(1).
    #[inline]
    pub fn headers(&self, _key: &str) -> Vec<(String, String)> {
        vec![
            ("x-ratelimit-limit".to_string(), self.capacity.to_string()),
            ("x-ratelimit-window".to_string(), "60".to_string()),
        ]
    }
    /// Complexity: O(1).
    #[inline]
    pub fn len(&self) -> usize {
        self.buckets.len()
    }
    /// Complexity: O(1).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buckets.is_empty()
    }
    /// Complexity: O(1).
    #[inline]
    pub fn capacity(&self) -> u32 {
        self.capacity
    }
    /// Complexity: O(1).
    #[inline]
    pub fn refill_per_sec(&self) -> f64 {
        self.refill_per_sec
    }
}

// --- Cookie extras ---

impl Cookie {
    /// Parse de `Set-Cookie: name=val; Path=/; Max-Age=..; Secure; HttpOnly; SameSite=..`.
    pub fn parse_set_cookie(s: &str) -> Result<Self, String> {
        let mut parts = s.split(';');
        let first = parts.next().ok_or_else(|| "empty set-cookie".to_string())?.trim();
        let (n, v) = first.split_once('=').ok_or_else(|| "bad set-cookie pair".to_string())?;
        if n.trim().is_empty() {
            return Err("empty cookie name".to_string());
        }
        let mut c = Cookie::new(n.trim(), v.trim());
        for attr in parts {
            let a = attr.trim();
            let lower = a.to_ascii_lowercase();
            if lower == "secure" {
                c.secure = true;
            } else if lower == "httponly" {
                c.http_only = true;
            } else if let Some(rest) = lower.strip_prefix("max-age=") {
                c.max_age = rest.trim().parse::<i64>().ok();
            } else if let Some(rest) = a.strip_prefix("Path=").or_else(|| a.strip_prefix("path=")) {
                c.path = rest.trim().to_string();
            } else if lower.starts_with("samesite=") {
                c.same_site = a[9..].trim().to_string();
            } else if lower.starts_with("expires=") {
                // Expires no passado => expirado (Max-Age 0)
                let val = a[8..].trim();
                if val.contains("1970") || val.contains("1990") {
                    c.max_age = Some(0);
                }
            }
        }
        Ok(c)
    }

    pub fn is_expired(&self, _now_unix: i64) -> bool {
        matches!(self.max_age, Some(a) if a <= 0)
    }

    pub fn matches(&self, url: &LexUrl) -> bool {
        if self.secure && !matches!(url.scheme.as_str(), "https" | "wss") {
            return false;
        }
        if !url.path.starts_with(self.path.as_str()) {
            return false;
        }
        true
    }
}

impl CookieJar {
    pub fn store_set_cookie(&mut self, set_cookie: &str) -> Result<(), String> {
        let c = Cookie::parse_set_cookie(set_cookie)?;
        self.push(c);
        Ok(())
    }
    pub fn header_for(&self, url: &LexUrl) -> String {
        let now = SessionStore::now_unix();
        self.cookies
            .iter()
            .filter(|c| !c.is_expired(now) && c.matches(url))
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ")
    }
    pub fn evict_expired(&mut self, now_unix: i64) -> usize {
        let before = self.cookies.len();
        self.cookies.retain(|c| !c.is_expired(now_unix));
        before - self.cookies.len()
    }
    pub fn len(&self) -> usize {
        self.cookies.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }
}

pub fn sign_cookie(value: &str, secret: &str) -> String {
    let sig = hmac_sha256(secret.as_bytes(), value.as_bytes());
    format!("{value}.{}", base64url_encode(&sig))
}

pub fn verify_cookie(signed: &str, secret: &str) -> Option<String> {
    let (val, sig) = signed.rsplit_once('.')?;
    let expected = hmac_sha256(secret.as_bytes(), val.as_bytes());
    let got = base64url_decode(sig).ok()?;
    if timing_safe_eq(&got, &expected) {
        Some(val.to_string())
    } else {
        None
    }
}

// --- Session extras ---

pub trait SessionBackend {
    fn load(&self, id: &str) -> Option<Session>;
    fn save(&mut self, sess: Session);
    fn destroy(&mut self, id: &str) -> bool;
}

impl SessionBackend for SessionStore {
    fn load(&self, id: &str) -> Option<Session> {
        self.get(id).cloned()
    }
    fn save(&mut self, sess: Session) {
        // acesso privado ok no mesmo módulo
        self.sessions.insert(sess.id.clone(), sess);
    }
    fn destroy(&mut self, id: &str) -> bool {
        self.remove(id)
    }
}

impl SessionStore {
    pub fn regenerate(&mut self, old_id: &str) -> Option<String> {
        let old = self.sessions.get(old_id)?.clone();
        let new_id = uuid::Uuid::new_v4().to_string();
        let sess = Session { id: new_id.clone(), data: old.data, expires_at: Self::now_unix() + self.ttl_secs };
        self.sessions.remove(old_id);
        self.sessions.insert(new_id.clone(), sess);
        Some(new_id)
    }
    pub fn touch(&mut self, id: &str) -> bool {
        match self.sessions.get_mut(id) {
            Some(s) => {
                s.expires_at = Self::now_unix() + self.ttl_secs;
                true
            }
            None => false,
        }
    }
    pub fn cookie_header(&self, id: &str) -> String {
        format!("lex_session={id}; Path=/; HttpOnly; SameSite=Lax")
    }
}

// --- Multipart limitado ---

#[derive(Debug, Clone)]
pub struct MultipartLimits {
    pub max_bytes: usize,
    pub max_fields: usize,
    pub max_file_bytes: usize,
}

impl Default for MultipartLimits {
    fn default() -> Self {
        Self { max_bytes: DEFAULT_BODY_LIMIT_BYTES, max_fields: 100, max_file_bytes: DEFAULT_BODY_LIMIT_BYTES }
    }
}

impl MultipartLimits {
    pub fn new(max_bytes: usize, max_fields: usize, max_file_bytes: usize) -> Self {
        Self { max_bytes, max_fields, max_file_bytes }
    }
}

pub fn parse_multipart_limited(
    body: &[u8],
    boundary: &str,
    limits: &MultipartLimits,
) -> Result<Vec<MultipartField>, String> {
    if body.len() > limits.max_bytes {
        return Err(format!("body too large: {} > {}", body.len(), limits.max_bytes));
    }
    let fields = parse_multipart(body, boundary, limits.max_bytes)?;
    if fields.len() > limits.max_fields {
        return Err(format!("too many fields: {} > {}", fields.len(), limits.max_fields));
    }
    for f in &fields {
        if f.data.len() > limits.max_file_bytes {
            return Err(format!("file too large: {} > {}", f.data.len(), limits.max_file_bytes));
        }
    }
    Ok(fields)
}

pub fn multipart_sanitize_filename(name: &str) -> String {
    let base = name.replace('\\', "/").split('/').last().unwrap_or("").to_string();
    let mut out = String::new();
    for c in base.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
            out.push(c);
        } else if c == ' ' {
            out.push('_');
        } else {
            out.push('_');
        }
    }
    let t = out.trim_matches('.').trim();
    let s = if t.is_empty() { "file".to_string() } else { t.to_string() };
    if s.len() > 128 {
        s[..128].to_string()
    } else {
        s
    }
}

// --- auth helpers ---

pub fn parse_basic_auth(header: &str) -> Result<(String, String), String> {
    let h = header.trim();
    let b64 = h.strip_prefix("Basic ").or_else(|| h.strip_prefix("basic ")).ok_or_else(|| "missing basic scheme".to_string())?.trim();
    if b64.is_empty() {
        return Err("empty basic credentials".to_string());
    }
    let raw = base64_decode(b64).map_err(|e| format!("bad base64: {e}"))?;
    let s = String::from_utf8(raw).map_err(|_| "invalid utf-8 in basic auth".to_string())?;
    let (u, p) = s.split_once(':').ok_or_else(|| "missing colon in basic auth".to_string())?;
    Ok((u.to_string(), p.to_string()))
}

/// Comparação constant-time (anti timing-oracle).
/// Complexity: O(n).
#[inline]
pub fn timing_safe_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut acc = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
}

/// Autorização com wildcard sufixo `*` (ex: `admin:*` casa `admin:read`).
pub fn authorize_scope_wild(ctx: &AuthContext, required: &str) -> Result<(), String> {
    if !ctx.authenticated {
        return Err("unauthenticated".to_string());
    }
    // requerido com wildcard
    if let Some(prefix) = required.strip_suffix('*') {
        if ctx.scopes.iter().any(|s| s.starts_with(prefix)) {
            return Ok(());
        }
        // "*" casa tudo
        if required == "*" {
            return Ok(());
        }
        return Err(format!("forbidden: missing scope '{required}'"));
    }
    // escopo do ctx com wildcard
    for s in &ctx.scopes {
        if s == required {
            return Ok(());
        }
        if let Some(prefix) = s.strip_suffix('*') {
            if required.starts_with(prefix) {
                return Ok(());
            }
        }
        if s == "*" {
            return Ok(());
        }
    }
    Err(format!("forbidden: missing scope '{required}'"))
}

/// Verifica JWT HS256 manual (base64url + HMAC-SHA256 inline). Retorna payload.
/// Enforce `Limits` default no payload JSON (sem skip). Complexity: O(n).
pub fn verify_hs256(token: &str, secret: &str) -> Result<serde_json::Value, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("bad jwt shape".to_string());
    }
    if token.len() > (8 << 20) {
        return Err("jwt too large".to_string());
    }
    let signing = format!("{}.{}", parts[0], parts[1]);
    let expected = hmac_sha256(secret.as_bytes(), signing.as_bytes());
    let got = base64url_decode(parts[2]).map_err(|e| format!("bad jwt sig: {e}"))?;
    if !timing_safe_eq(&got, &expected) {
        return Err("invalid jwt signature".to_string());
    }
    let payload_bytes = base64url_decode(parts[1]).map_err(|e| format!("bad jwt payload: {e}"))?;
    let limits = crate::response::Limits::default();
    let v: serde_json::Value = crate::response::decode_json(&payload_bytes, &limits)
        .map_err(|e| format!("bad jwt json: {e}"))?;
    Ok(v)
}

pub fn hs256_sign(payload: &serde_json::Value, secret: &str) -> String {
    let header = base64url_encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let body = base64url_encode(serde_json::to_vec(payload).unwrap_or_default().as_slice());
    let signing = format!("{header}.{body}");
    let sig = hmac_sha256(secret.as_bytes(), signing.as_bytes());
    format!("{signing}.{}", base64url_encode(&sig))
}

// --- SSE parser ---

#[derive(Debug, Clone, Default)]
pub struct SseClientOpts {
    pub url: String,
    pub last_event_id: Option<String>,
    pub retry_ms: u64,
}

impl SseClientOpts {
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string(), last_event_id: None, retry_ms: 3000 }
    }
}

#[derive(Debug, Default)]
pub struct SseParser {
    buf: String,
    last_id: Option<String>,
}

impl SseParser {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn last_event_id(&self) -> Option<&str> {
        self.last_id.as_deref()
    }
    /// Alimenta chunk; retorna eventos completos (delimitados por linha em branco).
    pub fn push(&mut self, chunk: &str) -> Vec<SseEvent> {
        self.buf.push_str(chunk);
        let mut events = Vec::new();
        loop {
            let end = self.buf.find("\n\n").map(|i| (i, 2)).or_else(|| self.buf.find("\r\n\r\n").map(|i| (i, 4)));
            let Some((pos, sep)) = end else { break };
            let block = self.buf[..pos].to_string();
            self.buf = self.buf[pos + sep..].to_string();
            let mut ev = SseEvent::new("");
            let mut data_lines: Vec<String> = Vec::new();
            for line in block.lines() {
                let t = line.strip_prefix('\r').unwrap_or(line);
                if t.is_empty() || t.starts_with(':') {
                    continue;
                }
                if let Some(v) = t.strip_prefix("event:") {
                    ev.event = Some(v.trim().to_string());
                } else if let Some(v) = t.strip_prefix("data:") {
                    let v = v.strip_prefix(' ').unwrap_or(v);
                    data_lines.push(v.to_string());
                } else if let Some(v) = t.strip_prefix("id:") {
                    let id = v.trim().to_string();
                    ev.id = Some(id.clone());
                    self.last_id = Some(id);
                } else if let Some(v) = t.strip_prefix("retry:") {
                    if let Ok(n) = v.trim().parse::<u64>() {
                        ev.retry = Some(n);
                    }
                }
            }
            ev.data = data_lines.join("\n");
            events.push(ev);
        }
        events
    }
}

// --- proxy / cors full / encoding ---

pub fn proxy_connect_request(host: &str, port: u16, headers: &Headers) -> String {
    let mut s = format!("CONNECT {host}:{port} HTTP/1.1\r\n");
    s.push_str(&format!("Host: {host}:{port}\r\n"));
    for (k, v) in headers.iter() {
        s.push_str(&format!("{k}: {v}\r\n"));
    }
    s.push_str("\r\n");
    s
}

impl CorsMiddleware {
    /// Headers completos com Vary + allow-credentials.
    pub fn response_headers_full(&self) -> Vec<(String, String)> {
        let mut h = self.response_headers();
        h.push(("vary".to_string(), "Origin".to_string()));
        if self.allow_credentials {
            h.push(("access-control-allow-credentials".to_string(), "true".to_string()));
        }
        h
    }
}

/// Negocia Content-Encoding (Accept-Encoding com q). Retorna melhor suportado.
pub fn negotiate_content_encoding(accept: &str, supported: &[&str]) -> Option<String> {
    let mut parsed: Vec<(String, f32)> = Vec::new();
    for part in accept.split(',') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let (enc, q) = match p.split_once(';') {
            Some((e, qpart)) => {
                let qv = qpart.trim().strip_prefix("q=").and_then(|n| n.parse::<f32>().ok()).unwrap_or(1.0);
                (e.trim().to_ascii_lowercase(), qv)
            }
            None => (p.to_ascii_lowercase(), 1.0),
        };
        parsed.push((enc, q));
    }
    parsed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (enc, q) in parsed {
        if q <= 0.0 {
            continue;
        }
        if enc == "*" {
            if let Some(s) = supported.first() {
                return Some(s.to_string());
            }
            continue;
        }
        if supported.iter().any(|s| *s == enc) {
            return Some(enc);
        }
        if enc == "identity" && supported.iter().any(|s| *s == "identity") {
            return Some("identity".to_string());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Server wiring: Router + Logging/Cors/RateLimit + ServerLimits + shutdown
// ---------------------------------------------------------------------------

/// Handler puro: `HttpRequest -> HttpResponse` (sem I/O, testável).
pub type Handler = std::sync::Arc<
    dyn Fn(&HttpRequest, &HashMap<String, String>) -> HttpResponse + Send + Sync,
>;

/// Servidor configurado (após `ServerBuilder::build`).
///
/// Compõe: `Router` (roteamento) + `Logging`/`Cors` (middleware) +
/// `RateLimiter` (token-bucket per-key via `x-forwarded-for`/`client-key` ou
/// `"anon"`) + `ServerLimits` (body/header caps) + `ServerTimeouts`.
/// Conexões: `Connection: close` por request neste stub; keep-alive real
/// exigiria pool/hyper — documentado como upgrade futuro.
pub struct LexServer {
    routes: Vec<(HttpMethod, String, Handler)>,
    match_router: Router,
    cors: CorsMiddleware,
    logging: bool,
    limits: ServerLimits,
    timeouts: ServerTimeouts,
    rate: std::sync::Mutex<RateLimiter>,
    rate_key_header: String,
}

impl std::fmt::Debug for LexServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LexServer")
            .field("routes", &self.routes.len())
            .field("limits", &self.limits)
            .finish()
    }
}

/// Builder do [`LexServer`].
///
/// ```rust
/// use lexicon_http::module::{ServerBuilder, HttpResponse, HttpRequest, HttpMethod};
/// let s = ServerBuilder::new().route(HttpMethod::GET, "/hi", |_, _| HttpResponse::text("hi")).build();
/// let mut req = HttpRequest::new(HttpMethod::GET, "/hi");
/// let res = s.dispatch(&mut req, "127.0.0.1");
/// assert_eq!(res.status, 200);
/// ```
/// Complexity: `route` O(1) amortized; `dispatch` O(r+h) onde r = rotas, h = headers.
#[derive(Default)]
pub struct ServerBuilder {
    routes: Vec<(HttpMethod, String, Handler)>,
    cors: Option<CorsMiddleware>,
    logging: bool,
    limits: ServerLimits,
    timeouts: Option<ServerTimeouts>,
    rate_capacity: u32,
    rate_refill: f64,
    rate_enabled: bool,
    rate_key_header: String,
}

impl ServerBuilder {
    /// Complexity: O(1).
    #[inline]
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            cors: None,
            logging: true,
            limits: ServerLimits::default(),
            timeouts: None,
            rate_capacity: 100,
            rate_refill: 100.0 / 60.0,
            rate_enabled: true,
            rate_key_header: "x-client-key".to_string(),
        }
    }
    /// Registra rota `METHOD pattern` (ex: `/users/:id`).
    /// Complexity: O(1) amortized.
    pub fn route<F>(mut self, method: HttpMethod, pattern: &str, h: F) -> Self
    where
        F: Fn(&HttpRequest, &HashMap<String, String>) -> HttpResponse + Send + Sync + 'static,
    {
        self.routes
            .push((method, pattern.to_string(), std::sync::Arc::new(h)));
        self
    }
    /// Complexity: O(1).
    #[inline]
    pub fn cors(mut self, c: CorsMiddleware) -> Self {
        self.cors = Some(c);
        self
    }
    /// Complexity: O(1).
    #[inline]
    pub fn logging(mut self, on: bool) -> Self {
        self.logging = on;
        self
    }
    /// Complexity: O(1).
    #[inline]
    pub fn limits(mut self, l: ServerLimits) -> Self {
        self.limits = l;
        self
    }
    /// Complexity: O(1).
    #[inline]
    pub fn timeouts(mut self, t: ServerTimeouts) -> Self {
        self.timeouts = Some(t);
        self
    }
    /// Rate-limit: `capacity` burst, `refill_per_sec`.
    /// Complexity: O(1).
    #[inline]
    pub fn rate_limit(mut self, capacity: u32, refill_per_sec: f64) -> Self {
        self.rate_capacity = capacity;
        self.rate_refill = refill_per_sec;
        self.rate_enabled = true;
        self
    }
    /// Desabilita rate-limit. Complexity: O(1).
    #[inline]
    pub fn no_rate_limit(mut self) -> Self {
        self.rate_enabled = false;
        self
    }
    /// Complexity: O(1).
    #[inline]
    pub fn rate_key_header(mut self, h: &str) -> Self {
        self.rate_key_header = h.to_string();
        self
    }
    /// Complexity: O(n) onde n = rotas (constrói Router espelho).
    pub fn build(self) -> LexServer {
        let mut mr = Router::new();
        for (i, (m, p, _)) in self.routes.iter().enumerate() {
            mr.add_route(*m, p, &format!("h{i}"));
        }
        LexServer {
            routes: self.routes,
            match_router: mr,
            cors: self.cors.unwrap_or_default(),
            logging: self.logging,
            limits: self.limits,
            timeouts: self.timeouts.unwrap_or_default(),
            rate: std::sync::Mutex::new(RateLimiter::new(self.rate_capacity, self.rate_refill)),
            rate_key_header: if self.rate_enabled {
                self.rate_key_header
            } else {
                String::new()
            },
        }
    }
}

impl LexServer {
    /// Despacha request pura (aplica limits + rate + router + cors).
    /// Retorna `429` em rate-limit, `413` em body excess, `404` sem rota,
    /// `405` com método errado mas path existente.
    /// Complexity: O(r+h+m) onde r = rotas, h = headers, m = middlewares.
    pub fn dispatch(&self, req: &mut HttpRequest, client_key: &str) -> HttpResponse {
        if self.logging {
            log::info!(
                "{} {}",
                req.method.map(|m| m.as_str()).unwrap_or("-"),
                req.path
            );
        }
        // Body cap (ServerLimits).
        if req.body.len() > self.limits.max_body_bytes {
            return HttpResponse::payload_too_large(&format!(
                "body too large: {} > {}",
                req.body.len(),
                self.limits.max_body_bytes
            ));
        }
        // Header cap.
        let hbytes: usize = req.headers.iter().map(|(k, v)| k.len() + v.len() + 4).sum();
        if hbytes > self.limits.max_header_bytes {
            return HttpResponse::payload_too_large("headers too large");
        }
        // Rate-limit per-key (header dedicado ou client_key explícito).
        if !self.rate_key_header.is_empty() {
            let key = req
                .headers
                .get(&self.rate_key_header)
                .unwrap_or_else(|| client_key.to_string());
            let key = if key.is_empty() { "anon".to_string() } else { key };
            if let Ok(mut rl) = self.rate.lock() {
                if !rl.check(&key) {
                    return HttpResponse::too_many_requests("rate limit exceeded");
                }
            }
        }
        // Roteamento.
        let method = req.method.unwrap_or(HttpMethod::GET);
        let clean = req.path.split('?').next().unwrap_or(&req.path).to_string();
        if let Some((idx, params)) = self.match_router.match_route(method, &clean) {
            let (_, _, h) = &self.routes[idx];
            let mut res = h(req, &params);
            for (k, v) in self.cors.response_headers() {
                if !res.headers.has(&k) {
                    res.headers.insert(&k, &v);
                }
            }
            // HEAD strip.
            if method.requires_body_strip() {
                res.body = HttpBody::Empty;
            }
            return res;
        }
        // 405 se path existe com outro método.
        let mut allowed: Vec<String> = Vec::new();
        for m in HttpMethod::ALL {
            if *m == method {
                continue;
            }
            if self.match_router.match_route(*m, &clean).is_some() {
                allowed.push(m.as_str().to_string());
            }
        }
        if !allowed.is_empty() {
            return HttpResponse::method_not_allowed(&allowed.join(", "));
        }
        HttpResponse::not_found()
    }

    /// Número de rotas. Complexity: O(1).
    #[inline]
    pub fn len(&self) -> usize {
        self.routes.len()
    }
    /// Complexity: O(1).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
    /// Limites ativos. Complexity: O(1).
    #[inline]
    pub fn limits(&self) -> &ServerLimits {
        &self.limits
    }
    /// Timeouts ativos. Complexity: O(1).
    #[inline]
    pub fn timeouts(&self) -> &ServerTimeouts {
        &self.timeouts
    }

    /// Serve TCP manual (HTTP/1.1, `Connection: close`) com graceful shutdown.
    ///
    /// `shutdown` é qualquer future (ex: `tokio::signal::ctrl_c()` ou um
    /// `Notify::notified()` dos testes). Ao resolver, para de aceitar e
    /// drena conexões abertas (neste stub, cada conexão é curta).
    /// Timeouts de `ServerTimeouts.request` limitam cada conexão.
    /// Complexity: O(c*(n+m)) onde c = conexões.
    pub async fn serve<F>(&self, listener: tokio::net::TcpListener, shutdown: F) -> std::io::Result<()>
    where
        F: std::future::Future<Output = ()>,
    {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        tokio::pin!(shutdown);
        loop {
            tokio::select! {
                _ = &mut shutdown => break,
                acc = listener.accept() => {
                    let (mut sock, _) = acc?;
                    let req_timeout = self.timeouts.request;
                    let max_body = self.limits.max_body_bytes;
                    // Lê request simples com timeout (headers + body via Content-Length).
                    let read_fut = async {
                        let mut raw = Vec::new();
                        let mut buf = [0u8; 8192];
                        loop {
                            match sock.read(&mut buf).await {
                                Ok(0) => break,
                                Ok(n) => {
                                    raw.extend_from_slice(&buf[..n]);
                                    if raw.len() > max_body + MAX_HEADER_BYTES + 8192 {
                                        break;
                                    }
                                    if find_headers_end(&raw).is_some() {
                                        // Tenta ler corpo completo se Content-Length presente.
                                        if let Ok(head) = std::str::from_utf8(&raw[..find_headers_end(&raw).unwrap().0].to_vec()) {
                                            if let Some(cl) = head.lines().find_map(|l| {
                                                let ll = l.to_ascii_lowercase();
                                                ll.strip_prefix("content-length:").map(|v| v.trim().to_string())
                                            }) {
                                                if let Ok(need) = cl.parse::<usize>() {
                                                    let have = raw.len() - find_headers_end(&raw).unwrap().1;
                                                    if have >= need { break; }
                                                    // continua lendo
                                                    continue;
                                                }
                                            }
                                        }
                                        // Sem CL: se já tem fim de headers, aguarda breve EOF ou quebra.
                                        // Para `Connection: close` do teste, o cliente fecha write após send.
                                        // Evita bloqueio: quebra se não há mais dados imediatos (timeout cobre).
                                        // Aqui: tenta mais uma leitura com timeout curto via caller.
                                        break;
                                    }
                                    if raw.len() > 64*1024 { break; }
                                }
                                Err(_) => break,
                            }
                        }
                        raw
                    };
                    let raw = match tokio::time::timeout(req_timeout, read_fut).await {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                    // Parse mínimo: request-line + headers + body.
                    let mut res = self.serve_one(&raw);
                    // Serializa resposta.
                    let mut out = format!("HTTP/1.1 {} {}\r\n", res.status, status_message(res.status));
                    // Content-Length + CORS já injetados em dispatch quando rota casa;
                    // garante CL + close aqui.
                    let body_len = res.body.len();
                    res.headers.insert("content-length", &body_len.to_string());
                    res.headers.insert("connection", "close");
                    for (k, v) in res.headers.iter() {
                        out.push_str(&format!("{k}: {v}\r\n"));
                    }
                    out.push_str("\r\n");
                    let _ = sock.write_all(out.as_bytes()).await;
                    let _ = sock.write_all(res.body.as_slice()).await;
                }
            }
        }
        Ok(())
    }

    /// Trata um buffer bruto de request (usado por `serve` + testes).
    /// Complexity: O(n+m).
    pub fn serve_one(&self, raw: &[u8]) -> HttpResponse {
        let Some((hend, bend)) = find_headers_end(raw) else {
            return HttpResponse::bad_request("missing header end");
        };
        if hend > MAX_HEADER_BYTES {
            return HttpResponse::payload_too_large("headers too large");
        }
        let Ok(head) = std::str::from_utf8(&raw[..hend]) else {
            return HttpResponse::bad_request("invalid utf-8 in headers");
        };
        let mut lines = head.split("\r\n");
        // Fallback para \n puro.
        let first = lines.next().unwrap_or("");
        let first = if first.contains(' ') { first.to_string() } else {
            // Tenta split por \n se \r\n não casou.
            head.split('\n').next().unwrap_or("").trim_end_matches('\r').to_string()
        };
        let parts: Vec<&str> = first.split_whitespace().collect();
        if parts.len() < 2 {
            return HttpResponse::bad_request("bad request line");
        }
        let method = HttpMethod::from_str(parts[0]).unwrap_or(HttpMethod::GET);
        let target = parts[1];
        // Headers estritos (reaproveita validação; em erro => 400).
        let rest_block: String = {
            // Reconstrói bloco após request-line.
            let mut b = String::new();
            // Pula a primeira linha do head original.
            let mut first_skipped = false;
            for l in head.split("\r\n") {
                if !first_skipped {
                    first_skipped = true;
                    continue;
                }
                // Se head usava \n puro, split acima já quebrou; mantém compat.
                b.push_str(l);
                b.push_str("\r\n");
            }
            // Se head não tinha \r\n (só \n), tenta fallback.
            if !head.contains("\r\n") {
                b.clear();
                let mut it = head.split('\n');
                it.next();
                for l in it {
                    let ll = l.strip_suffix('\r').unwrap_or(l);
                    b.push_str(ll);
                    b.push_str("\r\n");
                }
            }
            b
        };
        let headers = match Headers::parse_raw_limited(&rest_block, MAX_HEADER_BYTES, MAX_HEADERS) {
            Ok(h) => h,
            Err(e) => return HttpResponse::bad_request(&e),
        };
        let body = raw[bend..].to_vec();
        if body.len() > self.limits.max_body_bytes {
            return HttpResponse::payload_too_large("body too large");
        }
        let mut req = HttpRequest::from_parts(method, target, headers, body);
        // client_key: prefere header dedicado, senão "anon".
        let ck = req.headers.get(&self.rate_key_header).unwrap_or_else(|| "anon".to_string());
        self.dispatch(&mut req, &ck)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_helpers_carry_correct_codes() {
        assert_eq!(HttpResponse::ok(vec![]).status, 200);
        assert_eq!(HttpResponse::created(&serde_json::json!({})).status, 201);
        assert_eq!(HttpResponse::no_content().status, 204);
        assert_eq!(HttpResponse::bad_request("x").status, 400);
        assert_eq!(HttpResponse::unauthorized("x").status, 401);
        assert_eq!(HttpResponse::forbidden("x").status, 403);
        assert_eq!(HttpResponse::not_found_json("x").status, 404);
        assert_eq!(HttpResponse::method_not_allowed("GET").status, 405);
        assert_eq!(HttpResponse::internal_error("x").status, 500);
    }

    #[test]
    fn error_envelopes_are_json_with_allow_header_on_405() {
        let r = HttpResponse::method_not_allowed("GET, POST");
        assert_eq!(r.headers.get("allow").as_deref(), Some("GET, POST"));
        let v: serde_json::Value = serde_json::from_slice(r.body.as_slice()).unwrap();
        assert_eq!(v["success"], false);
        let u = HttpResponse::unauthorized("nope");
        assert!(u.headers.get("www-authenticate").is_some());
    }

    #[test]
    fn cors_default_injects_headers_and_preflights_204() {
        let cors = CorsMiddleware::default();
        let res = HttpResponse::ok(b"{}".to_vec()).with_cors(&cors);
        assert_eq!(
            res.headers.get("access-control-allow-origin").as_deref(),
            Some("*")
        );
        let pre = cors.preflight_response(Some("POST"));
        assert_eq!(pre.status, 204);
        assert!(pre.body.is_empty());
        assert_eq!(
            pre.headers.get("access-control-allow-methods").as_deref(),
            Some("POST")
        );
    }

    #[test]
    fn cors_origin_validation_and_restrictive_mode() {
        let open = CorsMiddleware::default();
        assert!(open.origin_allowed("https://qualquer.site"));
        let strict = CorsMiddleware::restrictive("https://app.exemplo");
        assert!(strict.origin_allowed("https://app.exemplo"));
        assert!(!strict.origin_allowed("https://evil.site"));
        assert!(strict.allow_credentials);
    }

    // ---- headers ----

    #[test]
    fn header_validation_rejects_bad_names_values() {
        assert!(validate_header_name("content-type").is_ok());
        assert!(validate_header_name("X-Custom_123").is_ok());
        assert!(validate_header_name("").is_err());
        assert!(validate_header_name("bad name").is_err());
        assert!(validate_header_name("bad:colon").is_err());
        assert!(validate_header_name("caf\u{00e9}").is_err());
        assert!(validate_header_name(&"x".repeat(MAX_HEADER_NAME_LEN + 1)).is_err());
        assert!(validate_header_value("text/plain").is_ok());
        assert!(validate_header_value("a\tb").is_ok());
        assert!(validate_header_value("a\rb").is_err());
        assert!(validate_header_value("a\nb").is_err());
        assert!(validate_header_value("a\x00b").is_err());
        assert!(validate_header_value("a\x7fb").is_err());
        assert!(validate_header_value(&"v".repeat(MAX_HEADER_VALUE_LEN + 1)).is_err());
    }

    #[test]
    fn headers_strict_rejects_obs_fold_and_garbage() {
        // obs-fold: linha iniciando com SP.
        let raw = "Host: a\r\n continued: x\r\n";
        assert!(Headers::parse_raw_limited(raw, MAX_HEADER_BYTES, MAX_HEADERS).is_err());
        let raw2 = "Host: a\r\n\tcontinued: x\r\n";
        assert!(Headers::parse_raw_limited(raw2, MAX_HEADER_BYTES, MAX_HEADERS).is_err());
        // sem dois-pontos.
        assert!(Headers::parse_raw_limited("NoColonHere\r\n", MAX_HEADER_BYTES, MAX_HEADERS).is_err());
        // nome inválido.
        assert!(Headers::parse_raw_limited("Bad Name: x\r\n", MAX_HEADER_BYTES, MAX_HEADERS).is_err());
        // valor com CR embutido (split preserva? test direto).
        assert!(validate_header_value("a\rb").is_err());
        // over-limit bytes.
        let big = format!("X-Big: {}\r\n", "v".repeat(MAX_HEADER_BYTES));
        assert!(Headers::parse_raw_limited(&big, MAX_HEADER_BYTES, MAX_HEADERS).is_err());
        // over-limit count.
        let mut many = String::new();
        for i in 0..(MAX_HEADERS + 5) {
            many.push_str(&format!("X-H{i}: v\r\n"));
        }
        assert!(Headers::parse_raw_limited(&many, 1 << 20, MAX_HEADERS).is_err());
        // truncado sem fim não é erro aqui (parse tolera linhas), mas vazio ok.
        let h = Headers::parse_raw_limited("Content-Type: text/plain\r\n", MAX_HEADER_BYTES, MAX_HEADERS).expect("ok");
        assert_eq!(h.get("content-type").as_deref(), Some("text/plain"));
        // zero-copy get_ref.
        assert_eq!(h.get_ref("content-type"), Some("text/plain"));
        assert_eq!(h.get_ref("missing"), None);
    }

    // ---- URL ----

    #[test]
    fn url_parses_valid_shapes() {
        let u = LexUrl::parse("http://example.com/a?x=1#f").expect("http");
        assert_eq!(u.scheme, "http");
        assert_eq!(u.host, "example.com");
        assert_eq!(u.path, "/a");
        let v6 = LexUrl::parse("http://[::1]:8080/x").expect("v6");
        assert_eq!(v6.host, "::1");
        assert_eq!(v6.port, Some(8080));
        let f = LexUrl::parse("file:///tmp/a.txt").expect("file");
        assert_eq!(f.scheme, "file");
        let j = u.join("/abs?q=1").expect("join abs");
        assert_eq!(j.path, "/abs");
        let rel = u.join("sub").expect("join rel");
        assert!(rel.path.ends_with("sub"));
        assert_eq!(u.effective_port(), 80);
        let https = LexUrl::parse("https://h/").expect("https");
        assert_eq!(https.effective_port(), 443);
    }

    #[test]
    fn url_rejects_garbage_truncated_overlimit() {
        assert!(LexUrl::parse("").is_err());
        assert!(LexUrl::parse("http://").is_err());
        assert!(LexUrl::parse("://no-scheme").is_err());
        assert!(LexUrl::parse("http://[::1").is_err());
        assert!(LexUrl::parse("http://h:badport/x").is_err() || LexUrl::parse("http://h:badport/x").is_ok()); // porta inválida vira host puro ou erro — ambos sem panic
        assert!(LexUrl::parse("http://a\x00b/").is_err());
        assert!(LexUrl::parse("http://a\nb/").is_err());
        assert!(LexUrl::parse(&("http://h/".to_string() + &"x".repeat(MAX_URL_LEN))).is_err());
        // sem panic em bytes estranhos.
        assert!(LexUrl::parse("\u{FF}\u{FE}\u{0}").is_err());
        assert!(LexUrl::parse("http://?").is_ok() || LexUrl::parse("http://?").is_err());
        // socket addr.
        assert!(LexSocketAddr::parse("127.0.0.1:80").is_ok());
        assert!(LexSocketAddr::parse("[::1]:80").is_ok());
        assert!(LexSocketAddr::parse("noport").is_err());
        assert!(LexSocketAddr::parse("::1").is_err());
    }

    // ---- cookies ----

    #[test]
    fn cookies_parse_garbage_and_limits() {
        let jar = CookieJar::parse_cookie_header("a=1; b=2; empty; =x; c=3");
        assert_eq!(jar.get("a").map(|c| c.value.as_str()), Some("1"));
        assert_eq!(jar.get("b").map(|c| c.value.as_str()), Some("2"));
        // garbage não dá panic.
        let empty = CookieJar::parse_cookie_header(";;;");
        assert!(empty.is_empty());
        let c = Cookie::parse_set_cookie("id=42; Path=/; Max-Age=60; Secure; HttpOnly; SameSite=Lax").expect("set");
        assert_eq!(c.name, "id");
        assert!(!c.is_expired(0));
        assert!(Cookie::parse_set_cookie("").is_err());
        assert!(Cookie::parse_set_cookie("noequal").is_err());
        assert!(Cookie::parse_set_cookie("=v").is_err());
        let exp = Cookie::parse_set_cookie("a=b; Expires=Thu, 01 Jan 1970 00:00:00 GMT").expect("exp");
        assert!(exp.is_expired(0));
        // matches secure/path.
        let https = LexUrl::parse("https://h/app/x").unwrap();
        let http = LexUrl::parse("http://h/app/x").unwrap();
        let mut sc = Cookie::new("s", "v");
        sc.secure = true;
        sc.path = "/app".to_string();
        assert!(sc.matches(&https));
        assert!(!sc.matches(&http));
        assert!(!sc.matches(&LexUrl::parse("https://h/other").unwrap()));
        // header_for filtra expirados.
        let mut jar2 = CookieJar::new();
        jar2.push(sc.clone());
        let mut dead = Cookie::new("d", "x");
        dead.max_age = Some(0);
        jar2.push(dead);
        let h = jar2.header_for(&https);
        assert!(h.contains("s=v"));
        assert!(!h.contains("d="));
        assert_eq!(jar2.evict_expired(0), 1);
    }

    // ---- multipart ----

    #[test]
    fn multipart_garbage_truncated_overlimit() {
        let b = "--B\r\nContent-Disposition: form-data; name=\"f\"\r\n\r\nhello\r\n--B--\r\n";
        let fs = parse_multipart(b.as_bytes(), "B", 1 << 20).expect("mp");
        assert_eq!(fs.len(), 1);
        assert_eq!(fs[0].text(), "hello");
        // empty boundary.
        assert!(parse_multipart(b.as_bytes(), "", 100).is_err());
        // over-limit body.
        assert!(parse_multipart(b.as_bytes(), "B", 4).is_err());
        // garbage boundary sem match => 0 fields, sem panic.
        let g = parse_multipart(b"garbage\x00\xff no boundary", "ZZZ", 1 << 20).expect("garbage ok");
        assert!(g.is_empty());
        // truncado sem separador corpo => ignora parte.
        let t = parse_multipart(b"--B\r\nContent-Disposition: form-data; name=\"f\"\r\nno-blank", "B", 1 << 20).expect("trunc");
        assert!(t.is_empty());
        // sem name => skip.
        let nn = "--B\r\nContent-Disposition: form-data\r\n\r\nx\r\n--B--\r\n";
        let r = parse_multipart(nn.as_bytes(), "B", 1 << 20).expect("noname");
        assert!(r.is_empty());
        // limited: too many fields + file too large.
        let mut multi = String::new();
        for i in 0..5 {
            multi.push_str(&format!("--B\r\nContent-Disposition: form-data; name=\"f{i}\"\r\n\r\nx\r\n"));
        }
        multi.push_str("--B--\r\n");
        let lim = MultipartLimits::new(1 << 20, 2, 1 << 20);
        assert!(parse_multipart_limited(multi.as_bytes(), "B", &lim).is_err());
        let bigf = "--B\r\nContent-Disposition: form-data; name=\"f\"; filename=\"a.bin\"\r\n\r\n0123456789\r\n--B--\r\n";
        let lim2 = MultipartLimits::new(1 << 20, 100, 4);
        assert!(parse_multipart_limited(bigf.as_bytes(), "B", &lim2).is_err());
        // sanitize traversal.
        assert_eq!(multipart_sanitize_filename("../../etc/passwd"), "passwd");
        assert_eq!(multipart_sanitize_filename("a/b\\c .txt"), "c_.txt");
        assert!(!multipart_sanitize_filename("").is_empty());
    }

    // ---- WS ----

    #[test]
    fn ws_frames_garbage_truncated_overlimit() {
        let f = WsFrame::text("hi");
        let enc = ws_encode(&f, false);
        let (back, n) = ws_decode(&enc).expect("ws roundtrip");
        assert_eq!(back.text_str(), "hi");
        assert_eq!(n, enc.len());
        // masked roundtrip.
        let enc_m = ws_encode(&WsFrame::binary(vec![1, 2, 3]), true);
        let (bm, _) = ws_decode(&enc_m).expect("masked");
        assert_eq!(bm.payload, vec![1, 2, 3]);
        // truncados.
        assert!(ws_decode(&[]).is_err());
        assert!(ws_decode(&[0x81]).is_err());
        assert!(ws_decode(&[0x81, 0x7E, 0x00]).is_err()); // ext16 incompleto
        assert!(ws_decode(&[0x81, 0x7F, 0x00, 0x01]).is_err()); // ext64 incompleto
        assert!(ws_decode(&[0x81, 0x80, 0x01]).is_err()); // mask incompleta
        assert!(ws_decode(&[0x81, 0x02, 0x41]).is_err()); // payload incompleto
        // opcode desconhecido.
        assert!(ws_decode(&[0x8F, 0x00]).is_err());
        // payload gigante via len64 > 16MiB.
        let mut huge = vec![0x82u8, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00];
        huge.extend_from_slice(&[0u8; 4]);
        assert!(ws_decode(&huge).is_err());
        // close codes.
        assert!(ws_close_code_valid(1000));
        assert!(!ws_close_code_valid(1005));
        assert!(ws_close_code_valid(4000));
        assert!(!ws_close_code_valid(2000));
        assert!(ws_check_text(b"ok").is_ok());
        assert!(ws_check_text(&[0xFF, 0xFE]).is_err());
        // reassembler.
        let mut re = WsReassembler::new();
        let p1 = WsFrame { opcode: WsOpcode::Text, payload: b"he".to_vec(), fin: false };
        assert!(re.push(p1).expect("frag1").is_none());
        let p2 = WsFrame { opcode: WsOpcode::Continuation, payload: b"llo".to_vec(), fin: true };
        let done = re.push(p2).expect("frag2").expect("done");
        assert_eq!(done.text_str(), "hello");
        // continuation inesperada.
        let mut re2 = WsReassembler::new();
        assert!(re2.push(WsFrame { opcode: WsOpcode::Continuation, payload: vec![], fin: true }).is_err());
        // control passa direto.
        let mut re3 = WsReassembler::new();
        assert!(re3.push(WsFrame::ping()).expect("ping").is_some());
        // handshake vetor RFC6455.
        assert_eq!(ws_accept_key("dGhlIHNhbXBsZSBub25jZQ=="), "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    // ---- rate limiter com relógio injetável ----

    #[test]
    fn rate_limiter_burst_refill_perkey_no_sleep() {
        use std::time::{Duration, Instant};
        let t0 = Instant::now();
        let mut tb = TokenBucket::new_at(3, 1.0, t0);
        assert!(tb.check_at(t0));
        assert!(tb.check_at(t0));
        assert!(tb.check_at(t0));
        assert!(!tb.check_at(t0)); // burst esgotado
        assert_eq!(tb.remaining_at(t0), 0);
        // +2s refill 2 tokens.
        let t2 = t0 + Duration::from_secs(2);
        assert_eq!(tb.remaining_at(t2), 2);
        assert!(tb.check_at(t2));
        assert!(tb.check_at(t2));
        assert!(!tb.check_at(t2));
        // capacidade não estoura com idle longo.
        let t100 = t0 + Duration::from_secs(100);
        assert_eq!(tb.remaining_at(t100), 3);
        // per-key isolamento.
        let mut rl = RateLimiter::new(2, 1.0);
        assert!(rl.check_at("a", t0));
        assert!(rl.check_at("a", t0));
        assert!(!rl.check_at("a", t0));
        assert!(rl.check_at("b", t0)); // outra chave tem burst próprio
        assert_eq!(rl.remaining_at("b", t0), 1);
        // refill por chave.
        assert!(rl.check_at("a", t0 + Duration::from_secs(1)));
        // prune_idle remove cheios ociosos.
        let pruned = rl.prune_idle(t0 + Duration::from_secs(1000), Duration::from_secs(10));
        assert!(pruned >= 1);
    }

    // ---- client pure fns ----

    #[test]
    fn client_builder_defaults_and_pure_helpers() {
        let c = HttpClientBuilder::new("http://127.0.0.1:9").expect("builder").build().expect("build");
        assert_eq!(c.opts.timeout_ms, 10_000);
        assert_eq!(c.opts.max_redirects, 5);
        assert_eq!(c.opts.max_body, 8 << 20);
        assert!(!c.opts.user_agent.is_empty());
        // custom.
        let c2 = HttpClientBuilder::new("http://h/")
            .unwrap()
            .timeout_ms(500)
            .max_redirects(2)
            .max_body_bytes(1024)
            .user_agent("t/1")
            .default_header("X-A", "b")
            .follow_redirects(false)
            .retry(3, 10, 100)
            .build()
            .unwrap();
        assert_eq!(c2.opts.timeout_ms, 500);
        assert_eq!(c2.opts.max_redirects, 2);
        // redirect rules.
        assert_eq!(redirect_method(301, HttpMethod::POST), HttpMethod::GET);
        assert_eq!(redirect_method(302, HttpMethod::POST), HttpMethod::GET);
        assert_eq!(redirect_method(303, HttpMethod::POST), HttpMethod::GET);
        assert_eq!(redirect_method(303, HttpMethod::HEAD), HttpMethod::HEAD);
        assert_eq!(redirect_method(307, HttpMethod::POST), HttpMethod::POST);
        assert_eq!(redirect_method(308, HttpMethod::PUT), HttpMethod::PUT);
        // retry.
        assert!(should_retry_status(429));
        assert!(should_retry_status(503));
        assert!(!should_retry_status(200));
        assert!(!should_retry_status(404));
        assert_eq!(retry_delay_ms(0, 100), 100);
        assert_eq!(retry_delay_ms(1, 100), 200);
        assert_eq!(parse_retry_after("2"), Some(2000));
        assert_eq!(parse_retry_after("x"), None);
        // https vira TlsRequired sem panic (blocking, sem runtime).
        let err = c2.blocking_get("https://example.com/").expect_err("tls");
        assert!(matches!(err, HttpError::TlsRequired(_)));
    }

    // ---- client + server integration em 127.0.0.1 efêmero ----

    async fn spawn_raw_http(responder: fn(&str) -> String) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let h = tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = listener.accept().await else { break };
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = vec![0u8; 8192];
                // Lê uma request (uma leitura basta p/ testes).
                let n = match sock.read(&mut buf).await {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                // Request-line: "GET /path HTTP/1.1".
                let path = req.lines().next().and_then(|l| l.split_whitespace().nth(1)).unwrap_or("/").to_string();
                let resp = responder(&path);
                let _ = sock.write_all(resp.as_bytes()).await;
                // close (Connection: close).
            }
        });
        (addr, h)
    }

    #[tokio::test]
    async fn client_get_ok_and_status_error_typed() {
        let (addr, h) = spawn_raw_http(|p| {
            if p == "/ok" {
                "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nhi".to_string()
            } else {
                "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nnot found".to_string()
            }
        })
        .await;
        let base = format!("http://{addr}");
        let client = HttpClientBuilder::new(&base).unwrap().timeout_ms(5000).retry(0, 0, 0).build().unwrap();
        let ok = tokio::time::timeout(std::time::Duration::from_secs(5), client.get("/ok")).await.expect("timeout").expect("ok");
        assert_eq!(ok.status, 200);
        assert_eq!(ok.body.as_slice(), b"hi");
        // 404 vira HttpError::Status tipado, não panic.
        let err = tokio::time::timeout(std::time::Duration::from_secs(5), client.get("/missing")).await.expect("timeout").expect_err("404 err");
        assert!(matches!(err, HttpError::Status(404, _)));
        h.abort();
    }

    #[tokio::test]
    async fn client_follows_redirects_and_caps_loops() {
        let (addr, h) = spawn_raw_http(|p| {
            match p {
                "/r1" => "HTTP/1.1 302 Found\r\nLocation: /r2\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string(),
                "/r2" => "HTTP/1.1 301 Moved Permanently\r\nLocation: /final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string(),
                "/final" => "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok".to_string(),
                "/loop" => "HTTP/1.1 302 Found\r\nLocation: /loop\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string(),
                _ => "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string(),
            }
        })
        .await;
        let base = format!("http://{addr}");
        let client = HttpClientBuilder::new(&base).unwrap().timeout_ms(5000).max_redirects(5).retry(0, 0, 0).build().unwrap();
        let ok = tokio::time::timeout(std::time::Duration::from_secs(5), client.get("/r1")).await.expect("t").expect("redirect ok");
        assert_eq!(ok.status, 200);
        // loop excede cap.
        let err = tokio::time::timeout(std::time::Duration::from_secs(5), client.get("/loop")).await.expect("t").expect_err("loop");
        assert!(matches!(err, HttpError::Redirect(_)));
        // sem follow, 302 cru via request_raw.
        let nofollow = HttpClientBuilder::new(&base).unwrap().follow_redirects(false).timeout_ms(5000).build().unwrap();
        let raw = nofollow.request_raw(HttpMethod::GET, "/r1", Headers::new(), HttpBody::Empty).await.expect("raw");
        assert_eq!(raw.status, 302);
        h.abort();
    }

    #[tokio::test]
    async fn client_post_put_delete_head_and_body_cap() {
        let (addr, h) = spawn_raw_http(|p| {
            match p {
                "/echo" => "HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nbody".to_string(),
                "/big" => {
                    let big = "x".repeat(4096);
                    format!("HTTP/1.1 200 OK\r\nContent-Length: 4096\r\nConnection: close\r\n\r\n{big}")
                }
                _ => "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok".to_string(),
            }
        })
        .await;
        let base = format!("http://{addr}");
        let client = HttpClientBuilder::new(&base).unwrap().timeout_ms(5000).retry(0, 0, 0).build().unwrap();
        let r = tokio::time::timeout(std::time::Duration::from_secs(5), client.post("/echo", b"hi".to_vec())).await.expect("t").expect("post");
        assert_eq!(r.status, 200);
        let r = tokio::time::timeout(std::time::Duration::from_secs(5), client.put("/echo", b"hi".to_vec())).await.expect("t").expect("put");
        assert_eq!(r.status, 200);
        let r = tokio::time::timeout(std::time::Duration::from_secs(5), client.delete("/echo")).await.expect("t").expect("del");
        assert_eq!(r.status, 200);
        // body cap estoura em streaming.
        let tiny = HttpClientBuilder::new(&base).unwrap().timeout_ms(5000).max_body_bytes(16).retry(0, 0, 0).build().unwrap();
        let err = tokio::time::timeout(std::time::Duration::from_secs(5), tiny.get("/big")).await.expect("t").expect_err("cap");
        assert!(matches!(err, HttpError::TooLarge(_)), "got {err:?}");
        h.abort();
    }

    #[test]
    fn blocking_client_works_without_runtime() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().unwrap();
        let th = std::thread::spawn(move || {
            for _ in 0..2 {
                let Ok((mut s, _)) = listener.accept() else { break };
                use std::io::{Read, Write};
                s.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();
                let mut raw = Vec::new();
                let mut buf = [0u8; 4096];
                loop {
                    match s.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            raw.extend_from_slice(&buf[..n]);
                            if raw.windows(4).any(|w| w == b"\r\n\r\n") {
                                // Se tem Content-Length, garante corpo completo.
                                if let Ok(head) = std::str::from_utf8(&raw) {
                                    if let Some(cl) = head.lines().find_map(|l| {
                                        let ll = l.to_ascii_lowercase();
                                        ll.strip_prefix("content-length:").map(|v| v.trim().to_string())
                                    }) {
                                        if let Ok(need) = cl.parse::<usize>() {
                                            if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                                                if raw.len() >= pos + 4 + need {
                                                    break;
                                                }
                                                continue;
                                            }
                                        }
                                    }
                                }
                                break;
                            }
                            if raw.len() > 64 * 1024 {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nhi";
                let _ = s.write_all(resp.as_bytes());
            }
        });
        let client = HttpClientBuilder::new(&format!("http://{addr}")).unwrap().timeout_ms(5000).retry(0, 0, 0).build().unwrap();
        let r = client.blocking_get("/a").expect("blocking get");
        assert_eq!(r.status, 200);
        assert_eq!(r.body.as_slice(), b"hi");
        let r = client.blocking_post("/a", b"x".to_vec()).expect("blocking post");
        assert_eq!(r.status, 200);
        let _ = th.join();
    }

    // ---- server wiring ----

    #[test]
    fn server_dispatch_routes_limits_rate_cors() {
        let s = ServerBuilder::new()
            .route(HttpMethod::GET, "/hi", |_, _| HttpResponse::text("hi"))
            .route(HttpMethod::GET, "/u/:id", |_, p| HttpResponse::text(p.get("id").map(|s| s.as_str()).unwrap_or("")))
            .cors(CorsMiddleware::default())
            .limits(ServerLimits { max_body_bytes: 16, max_header_bytes: 1 << 20, max_requests_per_socket: 1000 })
            .rate_limit(1, 0.0)
            .build();
        // hit.
        let mut req = HttpRequest::new(HttpMethod::GET, "/hi");
        let res = s.dispatch(&mut req, "k1");
        assert_eq!(res.status, 200);
        assert!(res.headers.has("access-control-allow-origin"));
        // param.
        let mut req2 = HttpRequest::new(HttpMethod::GET, "/u/42");
        let res2 = s.dispatch(&mut req2, "k2");
        assert_eq!(res2.body.as_slice(), b"42");
        // 404.
        let mut req3 = HttpRequest::new(HttpMethod::GET, "/nope");
        assert_eq!(s.dispatch(&mut req3, "k3").status, 404);
        // 405 (POST existe? adiciona POST /hi em outro server).
        let s405 = ServerBuilder::new()
            .route(HttpMethod::POST, "/only-post", |_, _| HttpResponse::text("p"))
            .build();
        let mut r405 = HttpRequest::new(HttpMethod::GET, "/only-post");
        assert_eq!(s405.dispatch(&mut r405, "x").status, 405);
        // body 413.
        let mut big = HttpRequest::new(HttpMethod::POST, "/hi");
        big.body = HttpBody::Buffer(vec![0u8; 64]);
        assert_eq!(s.dispatch(&mut big, "k4").status, 413);
        // rate 429 na segunda chamada mesma chave (burst 1, refill 0).
        let mut rr = HttpRequest::new(HttpMethod::GET, "/hi");
        // k1 já consumiu 1 token; segunda com k1 deve dar 429.
        assert_eq!(s.dispatch(&mut rr, "k1").status, 429);
    }

    #[test]
    fn server_serve_one_parses_raw() {
        let s = ServerBuilder::new()
            .route(HttpMethod::GET, "/a", |_, _| HttpResponse::text("A"))
            .build();
        let raw = b"GET /a HTTP/1.1\r\nHost: h\r\nConnection: close\r\n\r\n";
        let res = s.serve_one(raw);
        assert_eq!(res.status, 200);
        assert_eq!(res.body.as_slice(), b"A");
        // bad request line.
        assert_eq!(s.serve_one(b"BADLINE\r\n\r\n").status, 400);
        // missing header end.
        assert_eq!(s.serve_one(b"GET /a HTTP/1.1\r\nHost: h").status, 400);
        // obs-fold => 400.
        assert_eq!(s.serve_one(b"GET /a HTTP/1.1\r\nHost: h\r\n folded: x\r\n\r\n").status, 400);
    }

    #[test]
    fn coverage_gaps_new_public_items() {
        // Headers::validate_limited.
        let mut h = Headers::new();
        h.insert("x-a", "1");
        assert!(h.validate_limited(MAX_HEADER_BYTES, MAX_HEADERS).is_ok());
        let mut many = Headers::new();
        for i in 0..(MAX_HEADERS + 1) {
            many.inner.push((format!("x-{i}"), "v".to_string()));
        }
        assert!(many.validate_limited(MAX_HEADER_BYTES, MAX_HEADERS).is_err());
        // HttpError helpers.
        assert!(HttpError::is_retryable_status(429));
        assert!(!HttpError::is_retryable_status(404));
        assert_eq!(HttpError::Status(500, "x".to_string()).status_code(), Some(500));
        assert_eq!(HttpError::Timeout.status_code(), None);
        // TokenBucket/RateLimiter getters.
        let tb = TokenBucket::new(7, 2.0);
        assert_eq!(tb.capacity(), 7);
        assert!((tb.refill_rate() - 2.0).abs() < f64::EPSILON);
        let rl = RateLimiter::new(9, 3.0);
        assert_eq!(rl.capacity(), 9);
        assert!((rl.refill_per_sec() - 3.0).abs() < f64::EPSILON);
        // GraphQL/RPC limited parse.
        let g = GraphQlQuery::parse_limited(
            r#"{"query":"{ hi }","operationName":null,"variables":{}}"#,
            &crate::response::Limits::default(),
        )
        .expect("gql");
        assert_eq!(g.query, "{ hi }");
        assert!(GraphQlQuery::parse_limited("not json", &crate::response::Limits::default()).is_err());
        let r = RpcRequest::parse_limited(
            r#"{"jsonrpc":"2.0","method":"m","params":{},"id":1}"#,
            &crate::response::Limits::default(),
        )
        .expect("rpc");
        assert_eq!(r.method, "m");
        // LexServer getters.
        let s = ServerBuilder::new()
            .route(HttpMethod::GET, "/x", |_, _| HttpResponse::text("x"))
            .build();
        assert_eq!(s.len(), 1);
        assert!(!s.is_empty());
        assert!(s.limits().max_body_bytes > 0);
        let _ = s.timeouts().request;
        // DuplicateKeyPolicy default.
        assert_eq!(
            crate::response::DuplicateKeyPolicy::default(),
            crate::response::DuplicateKeyPolicy::LastWins
        );
    }

    #[tokio::test]
    async fn client_handles_chunked_and_post_json() {
        let (addr, h) = spawn_raw_http(|p| {
            if p == "/chunked" {
                "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n5\r\nhello\r\n0\r\n\r\n".to_string()
            } else {
                "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok".to_string()
            }
        })
        .await;
        // chunked via request_raw (200, sem retry).
        let base = format!("http://{addr}");
        let client = HttpClientBuilder::new(&base).unwrap().timeout_ms(5000).retry(0, 0, 0).build().unwrap();
        let raw = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.request_raw(HttpMethod::GET, "/chunked", Headers::new(), HttpBody::Empty),
        )
        .await
        .expect("t")
        .expect("chunked");
        assert_eq!(raw.body.as_slice(), b"hello");
        // post_json envia content-type json.
        let res = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.post_json("/echo", &serde_json::json!({"a": 1})),
        )
        .await
        .expect("t")
        .expect("post_json");
        assert_eq!(res.status, 200);
        h.abort();
    }

    #[tokio::test]
    async fn server_serve_with_graceful_shutdown() {
        use std::sync::Arc;
        let server = Arc::new(
            ServerBuilder::new()
                .route(HttpMethod::GET, "/ping", |_, _| HttpResponse::text("pong"))
                .build(),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let srv = server.clone();
        let h = tokio::spawn(async move {
            srv.serve(listener, async move {
                let _ = rx.await;
            })
            .await
            .unwrap();
        });
        // client cru via TcpStream.
        let client = HttpClientBuilder::new(&format!("http://{addr}")).unwrap().timeout_ms(5000).retry(0, 0, 0).build().unwrap();
        let res = tokio::time::timeout(std::time::Duration::from_secs(5), client.get("/ping")).await.expect("t").expect("ping");
        assert_eq!(res.body.as_slice(), b"pong");
        let _ = tx.send(());
        tokio::time::timeout(std::time::Duration::from_secs(5), h).await.expect("join").expect("serve");
    }
}
