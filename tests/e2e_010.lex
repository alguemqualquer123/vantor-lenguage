// e2e_010 - char type in fn signature
fn id_char(c: char) -> char {
    return c;
}
pub fn main() -> void {
    let r = id_char("a");
    return;
}
