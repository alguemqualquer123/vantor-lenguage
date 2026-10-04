// e2e_130 - Go-parity stdlib: regexp subset engine
module e2e_130;
import std::regexp;

pub fn main() -> void {
    assert(regexp::Match("a*b", "aaab"), "star");
    assert(regexp::Match("a+b", "aaab"), "plus");
    assert(regexp::Match("colou?r", "color"), "question");
    assert(regexp::Match("^a+$", "aaa"), "anchors");
    assert(!regexp::Match("^a+$", "aaab"), "anchors neg");
    assert(regexp::Match("cat|dog", "dog"), "alternation");
    assert(regexp::Match("(ab)+", "abab"), "group plus");
    assert(regexp::Match("[0-9]+", "abc123"), "class");
    let f = regexp::Find("[0-9]+", "abc123");
    assert(f.ok && f.text == "123" && f.start == 3, "Find");
    assert(regexp::ReplaceAll("a1b2", "[0-9]", "#") == "a#b#", "ReplaceAll");
    assert(regexp::Match("\\d+", "abc123"), "digit class");
    return;
}
