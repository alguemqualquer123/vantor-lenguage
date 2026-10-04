// http_hello.lex — minimal JSON API (real axum server on :3000).
// Run: lex run examples/http_hello.lex   (Ctrl+C to stop)
@Get("/")
pub fn hello() -> String {
    return "{\"hello\":\"lex\"}";
}

@Get("/health")
pub fn health() -> String {
    return "ok";
}

pub fn main() -> void {
    Http::serve("0.0.0.0:3000");
}
