// tar_pack.lex — pack and unpack ustar entries in memory.
import std::archive::tar;

pub fn main() -> void {
    let ar = tar::AppendFile("", "hello.txt", "hello tar", 420);
    ar = tar::AppendDir(ar, "docs", 493);
    let n = tar::Next(tar::NewReader(ar));
    Console::log(n.header.name + " (" + n.header.size.to_string() + " bytes)");
    Console::log(n.data);
}
