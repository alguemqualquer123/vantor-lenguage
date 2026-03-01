module tests.test_162;

fn test_162() {
    let val: String? = null
    let result = val ?? "default_162"
    assert(result == "default_162")
}


pub fn main() {
    test_162()
}
