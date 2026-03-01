module tests.test_614;

fn test_614() {
    let val: String? = null
    let result = val ?? "default_614"
    assert(result == "default_614")
}


pub fn main() {
    test_614()
}
