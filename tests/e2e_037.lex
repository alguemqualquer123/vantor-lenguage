// e2e_037 - loop with break and continue
pub fn main() -> void {
    let i = 0;
    loop {
        let i = i + 1;
        if i > 5 {
            break;
        } else {
            continue;
        }
    }
    return;
}
