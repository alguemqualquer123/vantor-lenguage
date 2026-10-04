// e2e_129 - Go-parity stdlib: fnv, crc32, adler32, rand, subtle
module e2e_129;
import std::hash::fnv;
import std::hash::crc32;
import std::hash::adler32;
import std::rand;
import std::crypto::subtle;

pub fn main() -> void {
    // fnv (IETF vectors for "hello")
    assert(fnv::Sum32a("hello") == 1335831723, "fnv32a hello");
    let h = fnv::New32a();
    h = fnv::Write32(h, "hell");
    h = fnv::Write32(h, "o");
    assert(fnv::Sum32(h) == fnv::Sum32a("hello"), "fnv streaming");

    // crc32 / adler32 (known "hello" digests)
    assert(crc32::Checksum("hello") == 907060870, "crc32 hello");
    let c = crc32::New();
    c = crc32::Write(c, "hello");
    assert(crc32::Sum32(c) == crc32::Checksum("hello"), "crc32 streaming");
    assert(adler32::Checksum("hello") == 103547413, "adler32 hello");

    // rand (range properties; seeded smoke)
    rand::Seed(42);
    let r = rand::Intn(100);
    assert(r >= 0 && r < 100, "Intn range");
    assert(rand::Float64() >= 0.0, "Float64 range");
    assert(rand::Shuffle([1, 2, 3]).len() == 3, "Shuffle len");
    assert(rand::Range(5, 10) >= 5, "Range lo");

    // subtle
    assert(subtle::ConstantTimeCompare("abc", "abc") == 1, "ct eq");
    assert(subtle::ConstantTimeCompare("abc", "abd") == 0, "ct ne");
    return;
}
