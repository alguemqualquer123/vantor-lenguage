module tests.test_173;

fn test_173() {
    let val: String? = null
    let result = val ?? "default_173"
    assert(result == "default_173")
}


pub fn main() {
    test_173()
}
