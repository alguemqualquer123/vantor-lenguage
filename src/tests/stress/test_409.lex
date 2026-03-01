module tests.test_409;

fn test_409() {
    let val: String? = null
    let result = val ?? "default_409"
    assert(result == "default_409")
}


pub fn main() {
    test_409()
}
