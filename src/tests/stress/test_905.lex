module tests.test_905;

fn test_905() {
    let val: String? = null
    let result = val ?? "default_905"
    assert(result == "default_905")
}


pub fn main() {
    test_905()
}
