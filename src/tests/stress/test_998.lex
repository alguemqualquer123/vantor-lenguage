module tests.test_998;

fn test_998() {
    let val: String? = null
    let result = val ?? "default_998"
    assert(result == "default_998")
}


pub fn main() {
    test_998()
}
