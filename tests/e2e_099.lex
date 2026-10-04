// e2e_099 - enum and match
enum Dir {
    North,
    South,
    East,
    West
}
fn tag(d: Dir) -> i64 {
    match d {
        North => 1,
        _ => 0
    }
    return 0;
}
pub fn main() -> void {
    let r = tag(North);
    return;
}
