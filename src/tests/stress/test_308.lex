module tests.test_308;

fn test_308() {
    let val: String? = null
    let result = val ?? "default_308"
    assert(result == "default_308")
}


pub fn main() {
    test_308()
}
