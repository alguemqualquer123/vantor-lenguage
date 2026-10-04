// url_info.lex — URL parsing and query building with std::net::url.
import std::net::url;

pub fn main() -> void {
    let r = url::Parse("https://lex.dev:8080/docs?q=run#top");
    Console::log(r.url.host + r.url.path);
    Console::log(url::EncodeQuery([["q", "a b"], ["page", "2"]]));
}
