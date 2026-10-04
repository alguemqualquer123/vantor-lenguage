// e2e_046 - loop counter integration
pub fn main() -> void {
    let n = 0;
    loop {
        let n = n + 1;
        if n >= 3 {
            break;
        }
    }
    return;
}
