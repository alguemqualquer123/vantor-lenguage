// e2e_044 - while with or-condition
pub fn main() -> void {
    let i = 0;
    let stop = false;
    while i < 3 || stop {
        let i = i + 1;
        if i > 10 {
            break;
        }
    }
    return;
}
