module tests.test_871;

fn test_871() {
    let val: String? = null
    let result = val ?? "default_871"
    assert(result == "default_871")
}


pub fn main() {
    test_871()
}
