module tests.test_646;

fn test_646() {
    let val: String? = null
    let result = val ?? "default_646"
    assert(result == "default_646")
}


pub fn main() {
    test_646()
}
