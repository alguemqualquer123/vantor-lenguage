// e2e_106 - class implements interface
interface Greeter {
    fn greet(name: String) -> String;
}
class Hi implements Greeter {
    fn greet(name: String) -> String {
        return name;
    }
    fn helper(x: int) -> int {
        return x;
    }
}
pub fn main() -> void {
    let h = Hi { n: 0 };
    return;
}
