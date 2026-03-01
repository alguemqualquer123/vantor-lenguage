module tests.test_428;

fn test_428() {
    let val: String? = null
    let result = val ?? "default_428"
    assert(result == "default_428")
}


pub fn main() {
    test_428()
}
