// e2e_116 - Result match integration
fn http(code: int) -> String {
    match code {
        200 => "ok",
        _ => "err"
    }
    return "done";
}
pub fn main() -> void {
    let a = Ok(200);
    let e = Err("x");
    let r = http(200);
    return;
}
