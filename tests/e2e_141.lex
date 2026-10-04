// e2e_141 - std legacy hashes: crypto/md5, crypto/sha1, crypto/rc4
module e2e_141;

import std::crypto::md5;
import std::crypto::sha1;
import std::crypto::rc4;

pub fn main() -> void {
    // RFC 1321 appendix + FIPS 180-1 reference vectors.
    assert(md5::Sum("") == "d41d8cd98f00b204e9800998ecf8427e", "md5 empty");
    assert(md5::Sum("abc") == "900150983cd24fb0d6963f7d28e17f72", "md5 abc");
    assert(
        md5::Sum("The quick brown fox jumps over the lazy dog") == "9e107d9d372bb6826bd81d3542a419d6",
        "md5 fox",
    );
    assert(
        md5::Sum("01234567890123456789012345678901234567890123456789012") == "a3360e2d7e28ed4572c3dc16ef705372",
        "md5 two blocks",
    );
    let mb = md5::SumBytes("abc");
    assert(mb.len() == 16, "md5 bytes");
    assert(md5::HexOf(mb) == "900150983cd24fb0d6963f7d28e17f72", "md5 hexof");

    assert(sha1::Sum("") == "da39a3ee5e6b4b0d3255bfef95601890afd80709", "sha1 empty");
    assert(sha1::Sum("abc") == "a9993e364706816aba3e25717850c26c9cd0d89d", "sha1 abc");
    assert(
        sha1::Sum("The quick brown fox jumps over the lazy dog") == "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12",
        "sha1 fox",
    );
    let sb = sha1::SumBytes("abc");
    assert(sb.len() == 20, "sha1 bytes");
    assert(sha1::HexOf(sb) == "a9993e364706816aba3e25717850c26c9cd0d89d", "sha1 hexof");

    // RC4 vectors (key, plaintext, ciphertext) — XOR is its own inverse.
    assert(rc4::Hex("Key", "Plaintext") == "BBF316E8D940AF0AD3", "rc4 key");
    assert(rc4::Hex("Wiki", "pedia") == "1021BF0420", "rc4 wiki");
    assert(rc4::Hex("Secret", "Attack at dawn") == "45A01F645FC35B383552544B9BF5", "rc4 secret");
    assert(rc4::HexLower("Key", "Plaintext") == "bbf316e8d940af0ad3", "rc4 lower");
    let ks = rc4::Keystream("Key", 10);
    assert(ks.len() == 10, "rc4 keystream");
    let enc = rc4::Cipher("k", "hello lex");
    assert(enc.len() == 9, "rc4 cipher len");
}
