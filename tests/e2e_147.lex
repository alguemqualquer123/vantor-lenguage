// e2e_147 - pipes: bare, chained and module-qualified targets
module e2e_147;

import std::strings;
import std::crypto::sha256;

fn dobrar(n: i64) -> i64 {
    return n * 2;
}

fn somar_um(n: i64) -> i64 {
    return n + 1;
}

pub fn main() -> void {
    let chain = 5 |> dobrar |> somar_um;
    assert(chain == 11, "chained pipe");

    let up = "lex" |> strings::ToUpper;
    assert(up == "LEX", "qualified pipe");

    let parts = "a b c" |> strings::Fields;
    assert(parts.len() == 3, "pipe into Fields");

    let joined = parts |> strings::Join("-");
    assert(joined == "a-b-c", "pipe with a second argument");

    let digest = sha256::Sum("lexicon");
    assert(
        digest == "239aec50c94b6ea398dadb908b531bbe124c08fbbe756ce60cd59c37cfd719f2",
        "pipe target and std call agree",
    );
    let also = "lexicon" |> sha256::Sum;
    assert(also == digest, "pipe into sha256::Sum");

    let cut = "lexicon" |> strings::HasPrefix("le");
    assert(cut, "pipe as first argument before the literal");
    return;
}
