// e2e_112 - Result type with Ok and Err
fn check(x: int) -> Result<int, String> {
    if x > 0 {
        return Ok(x);
    } else {
        return Err("neg");
    }
    return Ok(0);
}
pub fn main() -> void {
    let r = check(2);
    return;
}
