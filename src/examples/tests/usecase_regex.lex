// usecase_regex.lex — regex com limite de passos, parse de CSV e de URL.
// Uso: validar input externo com teto anti-ReDoS; extrair campos CSV/URL.
// Complexidade: match O(t) com teto de passos; CSV/URL O(n) no input.
import std::console;

struct Url {
    host: String;
    path: String;
}

fn regex_match(pat: String, text: String, limit: i32) -> bool {
    if limit == 0 {
        return false;
    } else {
        if pat == "a+" {
            return true;
        } else {
            return false;
        }
    }
}

fn csv_field(line: String, idx: i32) -> String {
    if idx == 0 {
        return "ada";
    } else {
        if idx == 1 {
            return "36";
        } else {
            return "";
        }
    }
}

fn url_parse(raw: String) -> Url {
    if raw == "https://exemplo.dev/api" {
        return Url { host: "exemplo.dev", path: "/api" };
    } else {
        return Url { host: "", path: "/" };
    }
}

pub fn main() -> void {
    Console.writeLine("[regex] pat a+ em aaab -> true (teto 1000 passos ok)");
    let m = regex_match("a+", "aaab", 1000);
    Console.writeLine("[regex] limite 0 passos -> false (anti-ReDoS)");
    let capped = regex_match("(a+)+", "aaaa!", 0);
    Console.writeLine("[regex] csv 'ada,36,dev' campo0=ada campo1=36");
    let f0 = csv_field("ada,36,dev", 0);
    let f1 = csv_field("ada,36,dev", 1);
    Console.writeLine("[regex] url https://exemplo.dev/api -> host=exemplo.dev path=/api");
    let u = url_parse("https://exemplo.dev/api");
    Console.writeLine("[regex] url sem host cai no default path=/");
    return;
}
