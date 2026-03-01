module tests.test_219;

fn test_219() {
    let val: String? = null
    let result = val ?? "default_219"
    assert(result == "default_219")
}


pub fn main() {
    test_219()
}
