module tests.test_309;

fn test_309() {
    let val: String? = null
    let result = val ?? "default_309"
    assert(result == "default_309")
}


pub fn main() {
    test_309()
}
