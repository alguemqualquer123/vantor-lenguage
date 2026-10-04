// file_copy.lex — whole-file copy with std::os.
// Run: lex run examples/file_copy.lex -- in.txt out.txt
import std::os;

pub fn main() -> void {
    let argv = os::Args();
    if argv.len() < 3 {
        Console::log("usage: lex run file_copy.lex -- <src> <dst>");
        return;
    }
    let data = os::ReadFile(argv[1]);
    if os::WriteFile(argv[2], data) {
        Console::log("copied " + data.len().to_string() + " chars");
    } else {
        Console::log("write failed");
    }
}
