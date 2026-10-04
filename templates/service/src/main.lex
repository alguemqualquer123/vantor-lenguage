import core::io::Console;
import core::net::Http;
import core::json::Json;

// Const global: FORMA CORRETA de const (top-level, nao dentro de fn).
// ERRADO: const dentro de pub fn main() -> const nao e statement valido,
// use let / let mut dentro de funcao.
const APP_NAME = "service";

struct User {
    id: int;
    name: String;
}

@Test
fn test_api() {
    let response = Http::get("/status");
    // FORMA CORRETA: assert precisa de parenteses.
    // ERRADO: assert response.code == 200;
    assert(response.code == 200);
}

// FORMA CORRETA de retornar lista de structs.
// ERRADO: [ { "id": 1, "name": "Alice" } ] com chave entre aspas.
// Em Lex objeto anonimo { ... } sozinho nao parseia (Expected expression, found LBrace).
// Use StructName { campo: valor }.
pub fn users() -> User[] {
    let list: User[] = [
        User { id: 1, name: "Alice" },
        User { id: 2, name: "Bob" }
    ];
    return list;
}

@Get("/users")
pub fn get_users() -> User[] {
    return users();
}

// --- casos de uso: lista / Array / Slice ---
// Array em Lex = literal [a, b, c] (Spec Sec 10).
// Slice tipado = User[] ou []User ou String[] (posfixo T[] e prefixo []T valem o mesmo).
// ERRADO: const Owners = new List<User>([...]) -> new nao e expressao valida,
// const dentro de fn nao vale, e { id: 1 } sem nome da struct nao vale.
// ERRADO: List<String>::new() -> ::new nao parseia hoje (new e palavra reservada).
// CERTO: use [ ... ] + : Tipo[] + .len() / .push().
pub fn demo_lista() -> void {
    // Array literal de inteiros
    let nums = [1, 2, 3];
    println(nums);

    // Slice tipado de String
    let nomes: String[] = ["Alice", "Bob", "Charlie"];
    println(nomes);

    // Slice vazio tipado
    let vazio: int[] = [];
    println(vazio);

    // Lista de structs
    let owners: User[] = [
        User { id: 1, name: "John Doe" },
        User { id: 2, name: "Jane Smith" }
    ];
    println(owners);

    // len() e push() sao validos (MethodCall)
    println(owners.len());
    owners.push(User { id: 3, name: "Ada" });
    println(owners.len());

    // Acesso: via for + campo .name (index a[0] ainda nao parseia: Expected Semi but found LBracket)
    for o in owners {
        println(o.name);
    }

    let ownersJson = Json::serialize(owners);
    println(ownersJson);
    return;
}

// --- casos de uso: for ---
// FORMA CORRETA: for x in iteravel { ... }
// ERRADO JS: for (let owner in Owners) / for (const owner in Owners) / for (const o in ...)
// Sem parenteses, sem let/const, com in.
pub fn demo_for() -> void {
    let nums = [10, 20, 30];

    for x in nums {
        println(x);
    }

    let owners: User[] = [
        User { id: 1, name: "Alice" },
        User { id: 2, name: "Bob" }
    ];
    for u in owners {
        if u.id == 1 {
            println(u.name);
        }
    }

    let mut total: int = 0;
    for n in nums {
        total = total + n;
    }
    println(total);
    return;
}

// --- casos de uso: while ---
// FORMA CORRETA: while cond { ... } ou while (cond) { ... }
pub fn demo_while() -> void {
    let mut i: int = 0;
    while i < 3 {
        println(i);
        i = i + 1;
    }

    let mut j: int = 0;
    while (j < 3) {
        j = j + 1;
    }
    println(j);
    return;
}

// --- casos de uso: loop + break / continue ---
// FORMA CORRETA: loop { ... } com break / continue.
pub fn demo_loop() -> void {
    let mut i: int = 0;
    loop {
        i = i + 1;
        if i == 3 {
            break;
        }
    }
    println(i);

    while true {
        break;
    }

    let mut k: int = 0;
    while k < 10 {
        k = k + 1;
        if k < 5 {
            continue;
        } else {
            break;
        }
    }
    println(k);
    return;
}

// --- casos de uso: do-while ---
// FORMA CORRETA: do { ... } while cond;
pub fn demo_do_while() -> void {
    let mut i: int = 0;
    do {
        i = i + 1;
    } while i < 5;
    println(i);
    return;
}

// --- casos de uso: busca em Array sem closure ---
// ERRADO: db.find(u => u.id == id) -> closures x => ... ainda nao suportadas
// (parser diz: use a named fn instead).
// CERTO: for + if + return.
pub fn find_user_by_id(id: int) -> void {
    let owners: User[] = [
        User { id: 1, name: "John Doe" },
        User { id: 2, name: "Jane Smith" }
    ];
    for o in owners {
        if o.id == id {
            println(o.name);
            return;
        }
    }
    println("not found");
    return;
}

// --- casos de uso: etc (if/switch/match/tupla/interpolacao/fn) ---
fn soma(a: int, b: int) -> int {
    return a + b;
}

pub fn demo_etc() -> void {
    println(soma(1, 2));

    let n = "Ada";
    println("ola {n}");

    let t = (1, "a");
    println(t);

    let x = 1;
    if x == 1 {
        println(1);
    } else {
        println(0);
    }

    switch x {
        case 1:
            println(1);
        default:
            println(0);
    }

    match x {
        1 => println(1),
        _ => println(0)
    }
    return;
}

pub fn main() -> void {
    demo_lista();
    demo_for();
    demo_while();
    demo_loop();
    demo_do_while();
    find_user_by_id(2);
    demo_etc();

    let owners: User[] = users();
    for o in owners {
        Console.writeLine(o.name);
    }

    // Http::serve("0.0.0.0:8080");
    return;
}
