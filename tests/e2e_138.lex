// e2e_138 - Go-parity stdlib: mail, textproto, multipart, pem, ascii85, color, tar
module e2e_138;
import std::net::mail;
import std::net::textproto;
import std::mime::multipart;
import std::encoding::pem;
import std::encoding::ascii85;
import std::image::color;
import std::archive::tar;

pub fn main() -> void {
    // mail
    let a = mail::ParseAddress("Ada <a@x.com>");
    assert(a.ok && a.addr.address == "a@x.com" && a.addr.name == "Ada", "ParseAddress");
    assert(!mail::ParseAddress("nope").ok, "ParseAddress invalid");
    assert(mail::ParseAddressList("a@x.com, B <b@y.org>").len() == 2, "ParseAddressList");
    assert(mail::StringOf(a.addr) == "Ada <a@x.com>", "StringOf");

    // textproto
    let h = textproto::ReadMIMEHeader("Host: x\r\nX-A: 1\n\nbody");
    assert(textproto::Get(h.header, "host") == "x", "Get canonical");
    assert(h.body == "body", "body");
    assert(textproto::CanonicalKey("content-type") == "Content-Type", "CanonicalKey");

    // multipart
    let p = multipart::Parse("--B\r\nContent-Type: text/plain\r\n\r\nhi\r\n--B--", "B");
    assert(p.ok && p.parts[0].body == "hi", "multipart");
    assert(multipart::HeaderGet(p.parts[0], "Content-Type") == "text/plain", "HeaderGet");

    // pem roundtrip
    let e = pem::Encode("T", "hi");
    let d = pem::Decode(e);
    assert(d.ok && d.block.typ == "T", "pem");

    // ascii85 (Go vectors)
    assert(ascii85::Encode("hello") == "BOu!rDZ", "a85 encode");
    assert(ascii85::Decode("BOu!rDZ").text == "hello", "a85 decode");

    // color
    let c = color::ParseHex("#ff0000");
    assert(c.r == 255 && c.g == 0 && c.a == 255, "ParseHex");
    assert(color::ToHex(c) == "#ff0000", "ToHex");
    assert(color::ToGray(c).y > 70 && color::ToGray(c).y < 80, "ToGray");

    // tar roundtrip
    let ar = tar::AppendFile("", "a.txt", "hi", 420);
    let n = tar::Next(tar::NewReader(ar));
    assert(n.ok && n.header.name == "a.txt" && n.data == "hi", "tar");
    return;
}
