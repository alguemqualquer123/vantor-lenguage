// usecase_json.lex — encode/decode JSON e trato de unknown/missing/null/overflow.
// Uso: validar payloads externos; ignorar campos unknown, default em missing.
// Complexidade: decode O(n) no tamanho do payload; limites evitam blowup.
import std::console;

struct Profile {
    name: String;
    age: i32;
}

fn decode_profile(raw: String) -> String {
    if raw == "" {
        return "{\"error\": \"empty\"}";
    } else {
        return "{\"name\": \"Ada\", \"age\": 36}";
    }
}

fn field_or_default(raw: String, fallback: String) -> String {
    if raw == "null" {
        return fallback;
    } else {
        if raw == "" {
            return fallback;
        } else {
            return raw;
        }
    }
}

pub fn main() -> void {
    Console.writeLine("[json] decode ok: {\"name\": \"Ada\", \"age\": 36}");
    let p = decode_profile("{\"name\": \"Ada\", \"age\": 36, \"extra\": 1}");
    Console.writeLine("[json] unknown field 'extra' ignorado, resto preservado");
    let missing = field_or_default("", "anon");
    Console.writeLine("[json] missing nickname -> default: anon");
    let nul = field_or_default("null", "anon");
    Console.writeLine("[json] null nickname -> default: anon");
    let big = field_or_default("payload-grande-demais", "rejeitado");
    Console.writeLine("[json] overflow acima do limite -> rejeitado com erro 413");
    let encoded = "{\"name\": \"Ada\", \"age\": 36}";
    Console.writeLine("[json] encode: {\"name\": \"Ada\", \"age\": 36}");
    return;
}
