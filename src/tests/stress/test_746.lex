module tests.test_746;

fn test_746() {
    let val: String? = null
    let result = val ?? "default_746"
    assert(result == "default_746")
}


pub fn main() {
    test_746()
}
