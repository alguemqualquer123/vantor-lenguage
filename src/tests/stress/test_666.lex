module tests.test_666;

fn test_666() {
    let val: String? = null
    let result = val ?? "default_666"
    assert(result == "default_666")
}


pub fn main() {
    test_666()
}
