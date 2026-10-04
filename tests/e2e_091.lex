// e2e_091 - where clause before return type
fn get<T>(x: T) where T: Display -> T {
    return x;
}
pub fn main() -> void {
    let r = get(7);
    return;
}
