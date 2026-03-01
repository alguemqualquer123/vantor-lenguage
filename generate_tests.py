import os
import random

def generate_test(id):
    templates = [
        # Teste de Pipe e Aritmética
        f"""fn test_{id}() {{
    let result = {random.randint(1, 100)} ▷ |v| v + {random.randint(1, 10)} ▷ |v| v * 2
    assert(result > 0)
}}
""",
        # Teste de String e Interpolação
        f"""fn test_{id}() {{
    let name = "Lexicon_{id}"
    let msg = "Hello, {{name}}!"
    assert(msg == "Hello, Lexicon_{id}!")
}}
""",
        # Teste de Classes e Getters
        f"""@Getter
class User_{id} {{
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {{
        self.id = id;
        self.name = name;
    }}
}}

fn test_{id}() {{
    let u = User_{id}({id}, "User_{id}")
    assert(u.getId() == {id})
}}
""",
        # Teste de Null Safety
        f"""fn test_{id}() {{
    let val: String? = null
    let result = val ?? "default_{id}"
    assert(result == "default_{id}")
}}
"""
    ]
    
    return f"""module tests.test_{id};

{random.choice(templates)}

pub fn main() {{
    test_{id}()
}}
"""

os.makedirs("tests/stress", exist_ok=True)

for i in range(1, 1001):
    with open(f"tests/stress/test_{i}.lex", "w", encoding="utf-8") as f:
        f.write(generate_test(i))

print(f"Gerados 1000 testes em tests/stress/")
