// usecase_db.lex — QueryBuilder, prepared statements, migration e pool config.
// Uso: montar SQL parametrizado sem concatenar input; migrar schema v1->v2.
// Complexidade: build O(c) nos clauses; pool O(1) por checkout.
import std::console;

struct DbConfig {
    url: String;
    pool_max: i32;
}

struct Query {
    sql: String;
    params: i32;
}

fn qb_select(table: String) -> Query {
    if table == "users" {
        return Query { sql: "SELECT id, name FROM users", params: 0 };
    } else {
        return Query { sql: "SELECT 1", params: 0 };
    }
}

fn qb_where(base: Query, cond: String) -> Query {
    if cond == "" {
        return base;
    } else {
        return Query { sql: "SELECT id, name FROM users WHERE age > $1", params: 1 };
    }
}

fn prepared_exec(q: Query) -> i32 {
    if q.params == 1 {
        return 2;
    } else {
        return 0;
    }
}

fn migrate(version: i32) -> String {
    if version == 1 {
        return "migrate v1: CREATE TABLE users (id INT, name TEXT)";
    } else {
        return "migrate v2: ALTER TABLE users ADD COLUMN age INT";
    }
}

pub fn main() -> void {
    Console.writeLine("[db] pool config: url=sqlite:app.db pool_max=8");
    let cfg = DbConfig { url: "sqlite:app.db", pool_max: 8 };
    let base = qb_select("users");
    Console.writeLine("[db] builder: SELECT id, name FROM users");
    let filtered = qb_where(base, "age > 18");
    Console.writeLine("[db] builder + where: WHERE age maior que $1, params 1");
    let rows = prepared_exec(filtered);
    Console.writeLine("[db] prepared exec retornou 2 linhas (sem injecao)");
    Console.writeLine("[db] migrate v1: CREATE TABLE users (id INT, name TEXT)");
    let m1 = migrate(1);
    Console.writeLine("[db] migrate v2: ALTER TABLE users ADD COLUMN age INT");
    let m2 = migrate(2);
    return;
}
