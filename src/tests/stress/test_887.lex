module tests.test_887;

fn test_887() {
    let val: String? = null
    let result = val ?? "default_887"
    assert(result == "default_887")
}


pub fn main() {
    test_887()
}
