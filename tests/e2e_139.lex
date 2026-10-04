// e2e_139 - Go-parity stdlib: crc64, maphash, signal, user, expvar, utf16 + extras
module e2e_139;
import std::hash::crc64;
import std::hash::maphash;
import std::os::signal;
import std::os::user;
import std::expvar;
import std::unicode::utf16;
import std::errors;
import std::fmt;
import std::slices;
import std::time;

pub fn main() -> void {
    // crc64 ECMA ("123456789" -> 0x995DC9BBDF1939FA, signed view)
    assert(crc64::Checksum("123456789") == -7395533204333446662, "crc64 vector");
    let h = crc64::New();
    h = crc64::Write(h, "123456");
    h = crc64::Write(h, "789");
    assert(crc64::Sum64(h) == crc64::Checksum("123456789"), "crc64 streaming");

    // maphash (seeded; same seed stable, different seeds differ)
    let s = maphash::MakeSeed(7);
    assert(maphash::String(s, "hello") == maphash::String(s, "hello"), "maphash stable");

    // signal/user/expvar/utf16
    assert(signal::SIGINT == 2 && signal::SIGTERM == 15, "signals");
    assert(user::Current().home != "", "user home");
    expvar::NewInt("n");
    expvar::Add("n", 2);
    assert(expvar::Get("n") == 2, "expvar");
    assert(utf16::Decode(utf16::Encode("AZ")) == "AZ", "utf16 ascii");
    assert(utf16::EncodeRune(8364).len() == 1, "bmp single");
    assert(utf16::DecodeRune(55357, 56832) == 128512, "surrogate pair");

    // extras
    let j = errors::Join([errors::New("a"), errors::New("b")]);
    assert(errors::Is(j, errors::New("a")), "Join Is");
    assert(fmt::Errorf("x=%v", [1]).msg == "x=1", "Errorf");
    assert(slices::Repeat([1, 2], 2).len() == 4, "Repeat");
    assert(slices::Chunk([1, 2, 3], 2).len() == 2, "Chunk");
    assert(time::UnixMicroOf(time::Unix(1)) == 1000000, "UnixMicro");
    return;
}
