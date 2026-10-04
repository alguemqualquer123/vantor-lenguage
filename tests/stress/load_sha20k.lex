// load_sha20k — native one-shot over 20KB (Lex reference would take ~45s;
// the native fast path must finish in seconds with identical output).
import std::crypto::sha256;

pub fn main() -> void {
    let m = "";
    let k = 0;
    while k < 20000 {
        m = m + "a";
        k = k + 1;
    }
    let t0 = Time::now_unix_ms();
    let h = sha256::Sum(m);
    let dt = Time::now_unix_ms() - t0;
    assert(h == "cc17faaad36649c4603dda4d8ff97cb149722af0bcac0746305a2134ad2d0b97", "digest");
    Console::log(dt);
    return;
}
