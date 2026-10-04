// e2e_073 - fn without return annotation
fn log_it(x: int) {
    let y = x;
    return;
}
pub fn main() -> void {
    log_it(1);
    return;
}
