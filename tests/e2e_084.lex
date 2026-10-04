// e2e_084 - pipe closure over outer variable
pub fn main() -> void {
    let base = 10;
    let addb = base |v| v + base;
    return;
}
