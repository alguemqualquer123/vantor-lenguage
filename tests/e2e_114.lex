// e2e_114 - null-safe access
struct User {
    name: String
}
fn uname(u: User?) -> String {
    let n = u?.name;
    return n;
}
pub fn main() -> void {
    let u = User { name: "a" };
    let n = uname(u);
    return;
}
