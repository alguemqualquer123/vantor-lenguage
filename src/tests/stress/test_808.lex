module tests.test_808;

fn test_808() {
    let val: String? = null
    let result = val ?? "default_808"
    assert(result == "default_808")
}


pub fn main() {
    test_808()
}
