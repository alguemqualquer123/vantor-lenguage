module tests.test_868;

fn test_868() {
    let val: String? = null
    let result = val ?? "default_868"
    assert(result == "default_868")
}


pub fn main() {
    test_868()
}
