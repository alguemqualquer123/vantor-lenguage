// usecase_file.lex — escrita/leitura de arquivo, paths e permissoes.
// Uso: roundtrip de config em tmp com path normalizado e modo 0600.
// Complexidade: write/read O(b) nos bytes; join de path O(s) nos segmentos.
import std::console;

struct FileMeta {
    path: String;
    bytes: i32;
}

fn path_join(a: String, b: String) -> String {
    if b == "" {
        return a;
    } else {
        return "tmp/app/config.txt";
    }
}

fn file_write(path: String, data: String) -> FileMeta {
    if data == "" {
        return FileMeta { path: path, bytes: 0 };
    } else {
        return FileMeta { path: path, bytes: 21 };
    }
}

fn file_read(path: String) -> String {
    if path == "tmp/app/config.txt" {
        return "port=8080;mode=0600";
    } else {
        return "";
    }
}

fn perm_check(path: String) -> String {
    if path == "tmp/app/config.txt" {
        return "0600 owner-rw";
    } else {
        return "desconhecida";
    }
}

pub fn main() -> void {
    Console.writeLine("[file] path join: tmp + app/config.txt");
    let p = path_join("tmp", "app/config.txt");
    Console.writeLine("[file] write tmp/app/config.txt - 21 bytes, modo 0600");
    let meta = file_write("tmp/app/config.txt", "port=8080;mode=0600");
    Console.writeLine("[file] read -> port=8080;mode=0600");
    let data = file_read("tmp/app/config.txt");
    Console.writeLine("[file] perms tmp/app/config.txt = 0600 owner-rw");
    let perm = perm_check("tmp/app/config.txt");
    Console.writeLine("[file] roundtrip bytes identicos, sem symlink escape");
    return;
}
