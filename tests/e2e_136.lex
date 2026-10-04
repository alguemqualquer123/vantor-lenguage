// e2e_136 - Go-parity stdlib: regexp QuoteMeta, time ParseDate
module e2e_136;
import std::regexp;
import std::time;

pub fn main() -> void {
    // QuoteMeta makes literals match literally
    let q = regexp::QuoteMeta("a+b (c)");
    assert(regexp::Match(q, "a+b (c)"), "QuoteMeta match");
    assert(!regexp::Match(q, "aabbc"), "QuoteMeta literal");

    // ParseDate round-trips Format for both layouts
    let p = time::ParseDate("1970-01-02");
    assert(p.ok && time::UnixMilliOf(p.time) == 86400000, "ParseDate epoch+1");
    let p2 = time::ParseDate("2024-02-29 12:30:45");
    assert(p2.ok, "ParseDate leap");
    assert(time::Format(p2.time, "2006-01-02") == "2024-02-29", "Format roundtrip date");
    assert(time::Format(p2.time, "15:04:05") == "12:30:45", "Format roundtrip clock");
    assert(!time::ParseDate("not-a-date").ok, "ParseDate invalid");
    assert(!time::ParseDate("2024-13-01").ok, "ParseDate bad month");
    return;
}
