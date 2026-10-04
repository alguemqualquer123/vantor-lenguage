// e2e_132 - Go-parity stdlib: net/url
module e2e_132;
import std::net::url;

pub fn main() -> void {
    let r = url::Parse("https://ex.com:8080/p?q=1#f");
    assert(r.ok, "Parse ok");
    assert(r.url.scheme == "https", "scheme");
    assert(r.url.host == "ex.com", "host");
    assert(r.url.port == "8080", "port");
    assert(r.url.path == "/p", "path");
    assert(r.url.query == "q=1", "query");
    assert(r.url.fragment == "f", "fragment");
    assert(url::StringOf(r.url) == "https://ex.com:8080/p?q=1#f", "roundtrip");
    assert(!url::Parse("").ok, "Parse empty fails");
    assert(url::QueryEscape("a b&c") == "a+b%26c", "QueryEscape");
    assert(url::QueryUnescape("a+b%26c") == "a b&c", "QueryUnescape");
    assert(url::QueryEscape("é") == "%C3%A9", "escape utf8");
    assert(url::QueryUnescape("%C3%A9") == "é", "unescape utf8");
    assert(url::EncodeQuery([["a", "1"], ["b", "x y"]]) == "a=1&b=x+y", "EncodeQuery");
    let q = url::ParseQuery("a=1&b=x+y");
    assert(q.len() == 2 && q[1][0] == "b" && q[1][1] == "x y", "ParseQuery");
    return;
}
