// e2e_104 - trait declaration
trait Printable {
    fn print();
    fn len() -> int;
}
struct Doc {
    n: int
}
pub fn main() -> void {
    let d = Doc { n: 3 };
    return;
}
