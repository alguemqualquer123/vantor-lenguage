// e2e_096 - pub struct with semicolon separators
pub struct User {
    pub name: String;
    pub age: int;
}
pub fn main() -> void {
    let u = User { name: "a", age: 3 };
    return;
}
