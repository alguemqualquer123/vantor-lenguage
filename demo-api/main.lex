// lex-api — API HTTP de demonstração em Lexicon.
//_run: ../run-dev.ps1 (dev) ou ../run-prod.ps1 (prod)_
// main.lex é o arquivo executável (`lex run main.lex`).
// As seções espelham os módulos irmãos (models/store/routes/config.lex),
// validados com `lex vet` / `lint` / `fmt`.

import core::net::Http;
import core::json::Json;
import app::config;
import app::models;
import app::store;
import app::routes;

// --- config (espelha config.lex): lido do ambiente real ---
let app_env = Env::get("APP_ENV");
let db_url = Env::get("DATABASE_URL");
let log_level = Env::get("LOG_LEVEL");

// --- models (espelha models.lex) ---
struct User {
    id: int;
    name: String;
    email: String;
}


@Get("/")
pub fn root() -> String {
    return "lex-api";
}

// class Main {
//     pub Main () {

//     }
//     pub fn hello() -> String {
//         return "Hello from lex-api!";
//     }
// }


// const nn = new Main();

// nn.hello();
@Get("/hello")
pub fn hello() -> String {
    return "Hello from lex-api!";
}

@Get("/users")
pub fn users() -> String {
    return "users";
}

@Get("/stats")
pub fn stats() -> String {
    return "stats";
}

@Get("/health")
pub fn health() -> String {
    return "ok";
}

@Get("/config")
pub fn config() -> String {
    return "config";
}

@Post("/users")
pub fn create_user() -> String {
    return "created";
}

pub fn main() -> void {
    // print("=== lex-api ===");
    // print("env=" + app_env);
    // print("db=" + db_url);
    // print("log=" + log_level);
    // println("routes: ^7 /hello /users /stats /health /config POST /users ^0");
    // Http::serve("0.0.0.0:3001");
    RegisterUser("Alice", "alice@example.com");
}

const db: User[] = [];


pub fn RegisterUser(name: String, email: String) -> User {
    if (name == "" || email == "") {
        throw "Name and email are required";
    }
    return user;
}

pub fn getUserById(id: int) -> User {
    let user = db.find(u => u.id == id);

    return user;
}

pub fn getAllUsers() -> []User {
    return db;
}