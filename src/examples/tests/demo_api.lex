// demo-api — API de demonstração usada como teste do servidor HTTP real.
// Rotas via decorators (@Get/@Post) + Http::serve; o runner extrai porta e
// rotas e sobe o axum com CORS, status codes, 404/405 JSON e log.
// Teste: lex run --ci src/examples/tests/demo_api.lex  (CI só reporta)
//        lex run src/examples/tests/demo_api.lex        (sobe de verdade)

@Get("/")
pub fn root() -> String {
    return "Welcome to Lexicon API!";
}

@Get("/hello")
pub fn hello() -> String {
    return "Hello from Lexicon HTTP Server!";
}

@Get("/users")
pub fn users() -> String {
    return "user-list";
}

@Get("/stats")
pub fn stats() -> String {
    return "stats";
}

@Post("/users")
pub fn create_user() -> String {
    return "created";
}

pub fn main() -> void {
    Http::serve("0.0.0.0:18080");
    return;
}
