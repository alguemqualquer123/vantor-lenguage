// e2e_135 - Go-parity stdlib: crypto/sha256, crypto/hmac
module e2e_135;
import std::crypto::sha256;
import std::crypto::hmac;

pub fn main() -> void {
    // FIPS 180-4 / RFC 4231 vectors
    assert(sha256::Sum("") == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "empty");
    assert(sha256::Sum("abc") == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", "abc");
    assert(sha256::SumBytes("abc").len() == 32, "SumBytes len");
    assert(hmac::Sha256("key", "The quick brown fox jumps over the lazy dog") == "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8", "hmac rfc4231");
    assert(hmac::Sha256Bytes("k", "m").len() == 32, "hmac bytes len");
    return;
}
