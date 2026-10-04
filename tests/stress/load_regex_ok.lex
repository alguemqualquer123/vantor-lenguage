// load_regex_ok — nested groups/alternation on small inputs.
import std::regexp;

pub fn main() -> void {
    assert(regexp::Match("(a+)+b", "aaab"), "nested plus");
    assert(regexp::Match("(ab|cd)+", "abcdab"), "alt group");
    assert(regexp::Match("a{2,4}b", "aaab"), "counted");
    assert(regexp::Find("(a+)+b", "xxaaab").text == "aaab", "nested find");
    return;
}
