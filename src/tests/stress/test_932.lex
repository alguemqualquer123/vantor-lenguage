module tests.test_932;

fn test_932() {
    let val: String? = null
    let result = val ?? "default_932"
    assert(result == "default_932")
}


pub fn main() {
    test_932()
}
