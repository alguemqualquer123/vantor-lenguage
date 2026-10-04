// e2e_133 - Go-parity stdlib: slog, flag, mime
module e2e_133;
import std::slog;
import std::flag;
import std::mime;

pub fn main() -> void {
    // slog levels + attrs (output goes to stdout; assert the API surface)
    assert(slog::LevelDebug == -4, "LevelDebug");
    assert(slog::LevelError == 8, "LevelError");
    let l = slog::New(slog::LevelInfo);
    slog::Info(l, "hello", [["k", "v"]]);
    slog::Dbg("boot");
    let l2 = slog::With(l, [["a", "1"]]);
    slog::Warn(l2, "w", []);

    // flag definitions + getters (defaults, no CLI parsing in tests)
    flag::String("host", "localhost", "host name");
    flag::Int("port", 8080, "port number");
    flag::Bool("verbose", false, "chatty");
    assert(flag::GetString("host") == "localhost", "GetString default");
    assert(flag::GetInt("port") == 8080, "GetInt default");
    assert(!flag::GetBool("verbose"), "GetBool default");
    assert(flag::GetString("missing") == "", "GetString missing");

    // mime
    assert(mime::TypeByExtension(".json") == "application/json", "json ext");
    assert(mime::TypeByExtension(".png") == "image/png", "png ext");
    assert(mime::TypeByExtension(".zzz") == "", "unknown ext");
    let m = mime::ParseMediaType("text/html; charset=utf-8");
    assert(m.ok && m.typ == "text/html", "media type");
    assert(m.params[0][0] == "charset" && m.params[0][1] == "utf-8", "media param");
    assert(!mime::ParseMediaType("bogus").ok, "media invalid");
    return;
}
