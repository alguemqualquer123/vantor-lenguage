// e2e_124 - Go-parity stdlib: time, bytes, io, bufio, utf8
module e2e_124;
import std::time;
import std::bytes;
import std::io;
import std::bufio;
import std::unicode::utf8;

pub fn main() -> void {
    // time
    assert(time::UnixMilliOf(time::Unix(0)) == 0, "Unix epoch");
    assert(time::Format(time::Unix(0), "2006-01-02") == "1970-01-01", "Format date");
    assert(time::Format(time::Unix(0), "15:04:05") == "00:00:00", "Format clock");
    assert(time::Before(time::Unix(1), time::Unix(2)), "Before");
    assert(time::After(time::Unix(2), time::Unix(1)), "After");
    assert(time::Equal(time::Unix(5), time::Unix(5)), "Equal");
    assert(time::Sub(time::Unix(3), time::Unix(1)) == 2000, "Sub ms");

    // bytes
    let b = bytes::FromString("hello");
    assert(b.len() == 5, "FromString len");
    assert(bytes::ToString(b) == "hello", "ToString");
    assert(bytes::Equal(b, bytes::FromString("hello")), "Equal");
    assert(bytes::Index(b, bytes::FromString("ll")) == 2, "Index");
    assert(bytes::HasPrefix(b, bytes::FromString("he")), "HasPrefix");
    assert(bytes::HasSuffix(b, bytes::FromString("lo")), "HasSuffix");
    assert(bytes::ToString(bytes::ToUpper(b)) == "HELLO", "ToUpper");

    // io (functional reader: reassign after each Read)
    let r = io::NewReader("hello");
    let a = io::Read(r, 2);
    assert(a.chunk == "he", "Read chunk");
    r = a.reader;
    let rest = io::ReadAll(r);
    assert(rest == "llo", "ReadAll");
    let w = io::NewWriter();
    let c = io::Copy(w, io::NewReader("abc"));
    assert(c.n == 3, "Copy count");
    assert(c.writer.buf == "abc", "Copy buf");

    // bufio
    let sc = bufio::NewScanner("a\nb");
    let n1 = bufio::Next(sc);
    assert(n1.text == "a" && n1.ok, "Scanner first");
    sc = n1.scanner;
    let n2 = bufio::Next(sc);
    assert(n2.text == "b" && n2.ok, "Scanner second");

    // utf8
    assert(utf8::RuneCount("lexicon") == 7, "RuneCount");
    let d = utf8::DecodeRune("A");
    assert(d.rune == 65 && d.size == 1 && d.ok, "DecodeRune");
    assert(utf8::RuneLen(8364) == 3, "RuneLen euro");
    assert(utf8::EncodeRune(65) == "A", "EncodeRune");
    return;
}
