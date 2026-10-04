// e2e_128 - Go-parity stdlib: json, base64, hex, csv, html
module e2e_128;
import std::encoding::json;
import std::encoding::base64;
import std::encoding::hex;
import std::encoding::csv;
import std::html;

pub fn main() -> void {
    // json (native bridge)
    assert(json::Marshal([1, 2]) == "[1,2]", "Marshal");
    assert(json::Valid("[1,2]"), "Valid");
    assert(!json::Valid("{bad"), "Valid neg");
    let v = json::Unmarshal("[1,2]");
    assert(v[0] == 1 && v[1] == 2, "Unmarshal");

    // base64 (RFC 4648 vectors)
    assert(base64::Encode("Man") == "TWFu", "Encode");
    let d = base64::Decode("TWFu");
    assert(d.text == "Man" && d.ok, "Decode");
    assert(!base64::Decode("!!!").ok, "Decode invalid");

    // hex
    assert(hex::Encode("AB") == "4142", "hex Encode");
    let h = hex::Decode("4142");
    assert(h.text == "AB" && h.ok, "hex Decode");

    // csv
    let rows = csv::ReadAll("a,b\n\"x,y\",z");
    assert(rows.len() == 2, "csv rows");
    assert(rows[0][1] == "b", "csv field");
    assert(rows[1][0] == "x,y", "csv quoted");
    assert(csv::WriteAll([["a", "b"]]) == "a,b\n", "csv write");

    // html
    assert(html::EscapeString("<a>&") == "&lt;a&gt;&amp;", "Escape");
    assert(html::UnescapeString("&lt;a&gt;") == "<a>", "Unescape");
    return;
}
