module tests.test_844;

fn test_844() {
    let val: String? = null
    let result = val ?? "default_844"
    assert(result == "default_844")
}


pub fn main() {
    test_844()
}
