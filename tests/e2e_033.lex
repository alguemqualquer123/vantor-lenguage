// e2e_033 - nested if (else-if via nesting)
pub fn main() -> void {
    let x = 7;
    if x > 10 {
        let y = 3;
    } else {
        if x > 5 {
            let y = 2;
        } else {
            let y = 1;
        }
    }
    return;
}
