// e2e_051 - match string arms
fn code(s: String) -> i64 {
    match s {
        "a" => 1,
        "b" => 2,
        _ => 0
    }
    return 0;
}
pub fn main() -> void {
    let r = code("a");
    return;
}
