module tests.test_916;

fn test_916() {
    let val: String? = null
    let result = val ?? "default_916"
    assert(result == "default_916")
}


pub fn main() {
    test_916()
}
