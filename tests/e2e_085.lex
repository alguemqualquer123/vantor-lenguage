// e2e_085 - pipe closure with interpolated string body
pub fn main() -> void {
    let greet = 0 |name| "hi {name}";
    return;
}
