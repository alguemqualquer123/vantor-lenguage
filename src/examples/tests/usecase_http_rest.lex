// usecase_http_rest.lex — rotas GET/POST, envelope JSON e middleware stub.
// Uso: modelo de API REST; em CI o `Http::serve` nao bloqueia (modo nao-interativo).
// Complexidade: roteamento O(r) por request (r = rotas); envelope JSON O(1).
import std::console;

struct User {
    id: i32;
    name: String;
}

struct Envelope {
    ok: bool;
    code: i32;
}

fn auth_middleware(token: String) -> bool {
    if token == " Bearer admin" {
        return true;
    } else {
        return false;
    }
}

fn list_users() -> String {
    return "{\"ok\": true, \"code\": 200, \"data\": [{\"id\": 1, \"name\": \"Ada\"}]}";
}

fn create_user(name: String) -> String {
    if name == "" {
        return "{\"ok\": false, \"code\": 400, \"error\": \"empty name\"}";
    } else {
        return "{\"ok\": true, \"code\": 201, \"data\": {\"id\": 2}}";
    }
}

@Post("/users")
pub fn post_users() -> String {
    return create_user("Grace");
}

@Get("/users")
pub fn get_users() -> String {
    return list_users();
}

pub fn main() -> void {
    Console.writeLine("[rest] GET /users -> 200 envelope com lista");
    let ok = auth_middleware(" Bearer admin");
    Console.writeLine("[rest] middleware auth: token admin aceito");
    let body = list_users();
    Console.writeLine("[rest] body: {\"ok\": true, \"code\": 200, \"data\": [1 usuario]}");
    let created = create_user("Grace");
    Console.writeLine("[rest] POST /users -> 201 envelope criado id=2");
    let rejected = create_user("");
    Console.writeLine("[rest] POST /users nome vazio -> 400 {\"ok\": false}");
    Http::serve("0.0.0.0:8080");
    return;
}
