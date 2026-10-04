// e2e_060 - switch multi-expr case
pub fn main() -> void {
    let x = 2;
    switch x {
        case 1: break;
        case 2, 3: break;
        default: break;
    }
    return;
}
