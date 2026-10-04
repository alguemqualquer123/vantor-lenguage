// e2e_110 - struct plus enum plus match integration
struct Req {
    code: int
}
enum Kind {
    Get,
    Post
}
fn route(k: Kind) -> i64 {
    match k {
        Get => 1,
        _ => 2
    }
    return 0;
}
pub fn main() -> void {
    let q = Req { code: 1 };
    let r = route(Get);
    return;
}
