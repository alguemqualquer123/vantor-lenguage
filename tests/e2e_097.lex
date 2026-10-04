// e2e_097 - struct shorthand init
struct Cfg {
    host: String,
    port: int
}
pub fn main() -> void {
    let host = "h";
    let port = 1;
    let c = Cfg { host, port };
    return;
}
