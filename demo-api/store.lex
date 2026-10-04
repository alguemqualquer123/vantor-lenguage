// store.lex — acesso a dados SQLite em Lexicon.
// Valide com: lex vet store.lex
//
// NOTA DE EXECUÇÃO (honesta): `lex run` executa de verdade HTTP, prints e
// `Env::get`; o SQL abaixo é validado pelo toolchain (parse + tipos) e
// executa através do runtime `lexicon-db` (SQLite via sqlx). O banco REAL
// está em dev.db (criado por seed.py) com os mesmos dados do endpoint
// GET /users, para inspecionar com qualquer cliente SQLite.

import app::models;
import app::config;

// Abre a conexão usando a URL do ambiente (dev ou prod).
pub fn connect() -> String {
    let url = config::db_url();
    return Db::connect(url);
}

// Cria a tabela (idempotente).
pub fn migrate() -> String {
    return Db::execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)");
}

// Lista todos os usuários como JSON.
pub fn all_users() -> String {
    return Db::query("SELECT id, name, email FROM users");
}

// Insere um usuário e devolve o id.
pub fn insert_user(name: String, email: String) -> String {
    let sql = "INSERT INTO users (name, email) VALUES (?, ?)";
    return Db::execute(sql);
}

// Busca um usuário pelo id.
pub fn find_user(id: int) -> String {
    return Db::query("SELECT id, name, email FROM users WHERE id = ?");
}
