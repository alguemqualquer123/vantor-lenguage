@Get("/")
pub fn hello() -> String {
    return "{\"hello\":\"lex api\"}";
}

@Get("/health")
pub fn health() -> String {
    return "ok";
}

pub fn main() -> void {
    Http::serve("0.0.0.0:3000");
}
