// e2e_054 - match with break arm inside loop
pub fn main() -> void {
    let x = 1;
    loop {
        match x {
            1 => break,
            _ => 0
        }
        break;
    }
    return;
}
