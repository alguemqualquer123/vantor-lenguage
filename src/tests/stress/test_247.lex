module tests.test_247;

fn test_247() {
    let val: String? = null
    let result = val ?? "default_247"
    assert(result == "default_247")
}


pub fn main() {
    test_247()
}
