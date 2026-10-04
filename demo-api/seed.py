"""Cria o banco SQLite REAL da demo (stdlib apenas, sem dependências).

Uso:  python3 seed.py            # cria dev.db
      python3 seed.py prod.db    # cria prod.db

Os dados espelham o endpoint GET /users servido pelo `lex run main.lex`.
"""
import sqlite3
import sys

DB = sys.argv[1] if len(sys.argv) > 1 else "dev.db"

USERS = [
    (1, "John", "john@example.com"),
    (2, "Jane", "jane@example.com"),
    (3, "Bob", "bob@example.com"),
    (4, "Alice", "alice@example.com"),
]

con = sqlite3.connect(DB)
cur = con.cursor()
cur.execute("DROP TABLE IF EXISTS users")
cur.execute(
    "CREATE TABLE users ("
    "id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL)"
)
cur.executemany("INSERT INTO users (id, name, email) VALUES (?, ?, ?)", USERS)
con.commit()
rows = cur.execute("SELECT id, name, email FROM users ORDER BY id").fetchall()
con.close()
print(f"{DB}: {len(rows)} users")
for r in rows:
    print(r)
