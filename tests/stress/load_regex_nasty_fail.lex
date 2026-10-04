// load_regex_nasty_fail — catastrophic backtracking input MUST abort via
// the step budget (clean error, never a hang): `(a+)+$` on 25 a's + "b".
import std::regexp;

pub fn main() -> void {
    Console::log(regexp::Match("(a+)+$", "aaaaaaaaaaaaaaaaaaaaaaaaab"));
    return;
}
