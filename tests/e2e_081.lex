// e2e_081 - pipe closure single param
pub fn main() -> void {
    let inc = 0 |v| v + 1;
    return;
}
