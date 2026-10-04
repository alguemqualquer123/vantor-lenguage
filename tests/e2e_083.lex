// e2e_083 - pipe closure with bool body
pub fn main() -> void {
    let is_pos = 0 |v| v > 0;
    return;
}
