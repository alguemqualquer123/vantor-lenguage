// e2e_040 - while with continue
pub fn main() -> void {
    let i = 0;
    while i < 10 {
        let i = i + 1;
        if i < 5 {
            continue;
        } else {
            break;
        }
    }
    return;
}
