module tests.test_590;

fn test_590() {
    let val: String? = null
    let result = val ?? "default_590"
    assert(result == "default_590")
}


pub fn main() {
    test_590()
}
