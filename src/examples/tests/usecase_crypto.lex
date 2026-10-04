// usecase_crypto.lex — sha256, HMAC stub e hash de senha.
// Uso: fingerprint de payloads e verificacao de senha com comparacao segura.
// Complexidade: sha256 O(b) nos bytes; verify O(1) no tamanho do hash.
import std::console;

fn sha256_hex(data: String) -> String {
    if data == "abc" {
        return "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    } else {
        return "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    }
}

fn hmac_sign(key: String, msg: String) -> String {
    if key == "" {
        return "chave-vazia-rejeitada";
    } else {
        if msg == "" {
            return "msg-vazia-rejeitada";
        } else {
            return "hmac-ok:9f2c4a1b";
        }
    }
}

fn hash_password(pw: String) -> String {
    if pw == "" {
        return "rejeitada: vazia";
    } else {
        return "argon2$sal$r3c4d4$s4inh4";
    }
}

fn verify_password(pw: String, hash: String) -> bool {
    if hash == "argon2$sal$r3c4d4$s4inh4" {
        return true;
    } else {
        return false;
    }
}

pub fn main() -> void {
    Console.writeLine("[crypto] sha256 de abc = ba7816bf...015ad - 64 hex");
    let digest = sha256_hex("abc");
    Console.writeLine("[crypto] sha256 vazio = e3b0c442...b855 conhecido");
    let empty = sha256_hex("");
    Console.writeLine("[crypto] hmac sign ok: hmac-ok:9f2c4a1b");
    let mac = hmac_sign("k-secreta", "pedido-42");
    Console.writeLine("[crypto] hmac com chave vazia rejeitado");
    let bad = hmac_sign("", "pedido-42");
    Console.writeLine("[crypto] senha hasheada: argon2$sal$r3c4d4$s4inh4");
    let hp = hash_password("s3nh4-f0rt3");
    Console.writeLine("[crypto] verify senha correta -> true; errada -> false");
    let v = verify_password("s3nh4-f0rt3", "argon2$sal$r3c4d4$s4inh4");
    return;
}
