module tests.test_296;

fn test_296() {
    let val: String? = null
    let result = val ?? "default_296"
    assert(result == "default_296")
}


pub fn main() {
    test_296()
}
