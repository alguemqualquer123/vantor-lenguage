// e2e_118 - Test with args plus abi inline cfg
@Test("arith")
fn test_mul() -> void {
    let r = 2 * 3;
    return;
}
#[abi("C")]
#[inline]
fn fast(x: int) -> int {
    return x;
}
#[cfg(test)]
fn helper() -> int {
    return 1;
}
pub fn main() -> void {
    test_mul();
    let r = fast(2);
    return;
}
