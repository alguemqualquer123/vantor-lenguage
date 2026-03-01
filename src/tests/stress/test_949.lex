module tests.test_949;

fn test_949() {
    let val: String? = null
    let result = val ?? "default_949"
    assert(result == "default_949")
}


pub fn main() {
    test_949()
}
