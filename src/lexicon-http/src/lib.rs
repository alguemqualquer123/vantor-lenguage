pub mod module;
pub mod response;

pub use module::{hello, root, stats, users, HttpModule};
pub use module::{
    authenticate_bearer, authorize_scope, authorize_scope_wild, domain_to_ascii, domain_to_unicode,
    file_url_to_path, hs256_sign, is_ip, is_ipv4, is_ipv6, match_path, multipart_sanitize_filename,
    negotiate_content_encoding, parse_basic_auth, parse_bearer_token, parse_multipart,
    parse_multipart_limited, parse_retry_after, path_to_file_url, proxy_connect_request, qs_escape,
    qs_parse, qs_parse_default, qs_stringify, qs_unescape, redirect_method, retry_delay_ms,
    should_retry_status, status_message, timing_safe_eq, url_to_http_options, validate_api_key,
    validate_bearer, validate_header_name, validate_header_value, verify_hs256, ws_accept_key,
    ws_check_text, ws_close_code_valid, ws_decode, ws_encode, ws_handshake_response,
    ws_negotiate_protocol, AuthContext, AuthMiddleware, BlockList, Cookie, CookieDefaults,
    CookieError, CookieJar, CorsMiddleware, DnsCacheEntry, DnsConfig, DnsError, DnsOrder, DnsResolver,
    Envelope, Framework, GraphQlQuery, GrpcChannel, Headers, HttpAgent, HttpAgentOpts, HttpBody,
    HttpClient, HttpClientBuilder, HttpClientOpts, HttpError, HttpMethod, HttpOptions, HttpRequest,
    HttpResponse, HttpsClientOpts, IpFamily, LexServer, LexSocketAddr, LexUrl, LoggingMiddleware,
    Middleware, MiddlewareChain, MultipartError, MultipartField, MultipartLimits, Page, PageParams,
    ProxyConfig, ProxyError, ProxyKind, QuicConfig, RateLimiter, RateLimitMiddleware, Route, Router,
    RpcError, RpcRequest, RpcResponse, SecureContext, ServerBuilder, ServerLimits, ServerTimeouts,
    Session, SessionBackend, SessionStore, SniRouter, SseClientOpts, SseEvent, SseParser, TcpConfig,
    TlsConfig, TlsError, TlsVerifyMode, TlsVersion, TokenBucket, UdpConfig, UdsConfig, UrlError,
    UrlSearchParams, WsError, WsFrame, WsOpcode, WsOpcodeFull, WsReassembler, WsSession,
    DEFAULT_BODY_LIMIT_BYTES, DEFAULT_TLS_MIN_VERSION, MAX_HEADERS, MAX_HEADER_BYTES,
    MAX_HEADER_NAME_LEN, MAX_HEADER_VALUE_LEN, MAX_URL_LEN,
};
pub use response::JsonResponse as ResponseObj;
pub use response::{
    DuplicateKeyPolicy, JsonResponse, Limits, Response, SerError, decode_json,
    decode_json_reject_duplicates, encode_json, find_duplicate_key, has_duplicate_keys,
    response_err, response_ok,
};

pub fn int_to_string(n: i32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    if n < 0 {
        return "-".to_string() + &int_to_string_pos(-n);
    }
    int_to_string_pos(n)
}

fn int_to_string_pos(n: i32) -> String {
    if n == 0 {
        return "".to_string();
    }
    let digit = n % 10;
    let rest = n / 10;
    int_to_string_pos(rest) + &digit_char(digit)
}

fn digit_char(d: i32) -> String {
    match d {
        0 => "0",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        _ => "0",
    }
    .to_string()
}
