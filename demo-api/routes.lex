// routes.lex — handlers HTTP da API (importado pelo main).
// Valide com: lex vet routes.lex
// Cada `@Get`/`@Post` vira uma rota real no `lex run main.lex`.

import app::models;
import app::store;

@Get("/")
pub fn root() -> String {
    return "lex-api";
}

@Get("/hello")
pub fn hello() -> String {
    return "Hello from lex-api!";
}

@Get("/users")
pub fn users() -> String {
    return store::all_users();
}

@Post("/users")
pub fn create_user() -> String {
    return "created";
}

@Get("/stats")
pub fn stats() -> String {
    return "stats";
}

@Get("/health")
pub fn health() -> String {
    return "ok";
}
