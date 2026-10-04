// e2e_146 - package-scoped name resolution: std packages that declare the
// same simple names (Sum/Encode/ParseResult/POW2…) must not shadow each
// other from inside their own bodies.
module e2e_146;

import std::crypto::md5;
import std::crypto::sha1;
import std::crypto::sha256;
import std::encoding::hex;
import std::encoding::base64;
import std::net::url;
import std::net::mail;
import std::mime::multipart;

pub fn main() -> void {
    // md5/sha1/sha256 all define Sum, SumBytes, HexOf, words, rotr and POW2.
    assert(md5::Sum("abc") == "900150983cd24fb0d6963f7d28e17f72", "md5 own rounds");
    assert(sha1::Sum("abc") == "a9993e364706816aba3e25717850c26c9cd0d89d", "sha1 own rounds");
    assert(
        sha256::Sum("abc") == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "sha256 own rounds",
    );
    assert(md5::SumBytes("abc").len() == 16, "md5 byte width");
    assert(sha1::SumBytes("abc").len() == 20, "sha1 byte width");
    assert(sha256::SumBytes("abc").len() == 32, "sha256 byte width");

    // hex/base64 both export Encode + Decode over the same digits table name.
    assert(hex::Encode("abc") == "616263", "hex encode");
    assert(base64::Encode("abc") == "YWJj", "base64 encode");
    let hx = hex::Decode("616263");
    assert(hx.ok && hx.text == "abc", "hex decode");
    let b64 = base64::Decode("YWJj");
    assert(b64.ok && b64.text == "abc", "base64 decode");

    // url, mail and multipart each declare a `ParseResult` struct.
    let u = url::Parse("http://example.com:8080/a?b=1");
    assert(u.ok && u.url.host == "example.com" && u.url.port == "8080", "url result");
    let m = mail::ParseAddress("Ada <ada@example.com>");
    assert(m.ok && m.addr.address == "ada@example.com", "mail result");
    let mp = multipart::Parse("--X\r\nA: 1\r\n\r\nbody\r\n--X--\r\n", "X");
    assert(mp.ok && mp.parts.len() == 1, "multipart result");
    assert(multipart::HeaderGet(mp.parts[0], "A") == "1", "multipart part");
}
