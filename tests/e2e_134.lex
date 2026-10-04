// e2e_134 - Go-parity stdlib: base32, math/bits, strings/strconv extras
module e2e_134;
import std::encoding::base32;
import std::math::bits;
import std::strings;
import std::strconv;

pub fn main() -> void {
    // base32 (RFC 4648 vectors)
    assert(base32::Encode("foo") == "MZXW6===", "b32 Encode");
    assert(base32::Encode("") == "", "b32 empty");
    let d = base32::Decode("MZXW6===");
    assert(d.ok && d.text == "foo", "b32 Decode");
    assert(!base32::Decode("!!!").ok, "b32 invalid");

    // bits (exact i64, full 64-bit view)
    assert(bits::OnesCount(7) == 3, "OnesCount");
    assert(bits::OnesCount(-1) == 64, "OnesCount neg");
    assert(bits::TrailingZeros(8) == 3, "TrailingZeros");
    assert(bits::TrailingZeros(0) == 64, "TrailingZeros zero");
    assert(bits::LeadingZeros(1) == 63, "LeadingZeros");
    assert(bits::LeadingZeros(-1) == 0, "LeadingZeros neg");
    assert(bits::Len(255) == 8, "Len");
    assert(bits::RotateLeft(1, 1) == 2, "RotateLeft");
    assert(bits::RotateLeft(1, 64) == 1, "RotateLeft wrap");
    assert(bits::ReverseBytes(1) == 72057594037927936, "ReverseBytes");

    // strings extras
    let c = strings::Cut("a=b=c", "=");
    assert(c[0] == "a" && c[1] == "b=c" && c[2], "Cut");
    let c2 = strings::Cut("abc", "=");
    assert(!c2[2], "Cut miss");
    assert(strings::CutPrefix("foobar", "foo")[0] == "bar", "CutPrefix");
    assert(strings::Trim("xxabxx", "x") == "ab", "Trim cutset");

    // strconv extras
    let u = strconv::Unquote("\"a\\n\"");
    assert(u.ok, "Unquote ok");
    assert(!strconv::Unquote("nope").ok, "Unquote invalid");
    return;
}
