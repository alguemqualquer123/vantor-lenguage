// e2e_064 - switch inside while
pub fn main() -> void {
    let x = 0;
    while x < 2 {
        let x = x + 1;
        switch x {
            case 1: continue;
            default: break;
        }
    }
    return;
}
