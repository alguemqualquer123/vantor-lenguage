// e2e_144 - std mime/quotedprintable + std text/tabwriter
module e2e_144;

import std::mime::quotedprintable;
import std::text::tabwriter;

pub fn main() -> void {
    // RFC 2045 §6.7: printable ASCII passes through, '=' and control bytes
    // become =XX, and "=\r\n" is a soft line break.
    assert(quotedprintable::Encode("a b=c") == "a b=3Dc", "encode equals");
    assert(quotedprintable::Encode("line1\tline2") == "line1\tline2", "encode tab kept");
    let rt = quotedprintable::Decode(quotedprintable::Encode("h=1?x"));
    assert(rt.ok && rt.text == "h=1?x", "round trip");
    let h = quotedprintable::Decode("=48=65=6C=6C=6F");
    assert(h.ok && h.text == "Hello", "decode hello");
    let soft = quotedprintable::Decode("a=\r\nb");
    assert(soft.ok && soft.text == "ab", "soft break");
    let low = quotedprintable::Decode("=41=42");
    assert(low.ok && low.text == "AB", "lowercase hex accepted");
    let bad = quotedprintable::Decode("=zz");
    assert(bad.ok == false, "bad hex");
    let trunc = quotedprintable::Decode("abc=");
    assert(trunc.ok == false, "truncated escape");

    // Columns pad to max(minwidth, widest cell) + padding; the last cell of
    // each row stays free (Go's trailing-tab rule).
    let w = tabwriter::New(2, 1);
    w = tabwriter::Write(w, "a\tbb\tc");
    w = tabwriter::Write(w, "longer\tx\ttail cell");
    assert(tabwriter::Lines(w) == 2, "buffered rows");
    let out = tabwriter::Flush(w);
    assert(out == "a      bb c\nlonger x  tail cell\n", "flush pads");
    let single = tabwriter::Write(tabwriter::New(1, 1), "only one cell");
    assert(tabwriter::Flush(single) == "only one cell\n", "free-standing line");
    let empty = tabwriter::New(1, 1);
    assert(tabwriter::Flush(empty) == "", "empty flush");
}
