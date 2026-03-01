module tests.test_828;

fn test_828() {
    let val: String? = null
    let result = val ?? "default_828"
    assert(result == "default_828")
}


pub fn main() {
    test_828()
}
