// e2e_063 - switch case bodies with lets
pub fn main() -> void {
    let x = 1;
    switch x {
        case 1: {
            let y = 10;
            break;
        }
        default: {
            let y = 0;
            break;
        }
    }
    return;
}
