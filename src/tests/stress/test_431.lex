module tests.test_431;

fn test_431() {
    let val: String? = null
    let result = val ?? "default_431"
    assert(result == "default_431")
}


pub fn main() {
    test_431()
}
