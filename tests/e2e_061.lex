// e2e_061 - switch with guard
pub fn main() -> void {
    let x = 1;
    switch x {
        case 1 if x > 0: break;
        default: break;
    }
    return;
}
