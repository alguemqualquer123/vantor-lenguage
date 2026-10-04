// regexp_grep.lex — grep-like filter with std::regexp.
// Run: lex run examples/regexp_grep.lex -- "[0-9]+"
import std::regexp;

pub fn main() -> void {
    let argv = Process::args();
    let pattern = "[a-z]+@[a-z]+";
    if argv.len() > 1 {
        pattern = argv[1];
    }
    let lines = ["contact ada@mail.com today", "no address here", "bob@x.org wrote"];
    let i = 0;
    while i < lines.len() {
        let m = regexp::Find(pattern, lines[i]);
        if m.ok {
            Console::log(lines[i] + "  =>  " + m.text);
        }
        i = i + 1;
    }
}
