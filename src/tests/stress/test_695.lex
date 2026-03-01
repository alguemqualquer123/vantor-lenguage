module tests.test_695;

fn test_695() {
    let val: String? = null
    let result = val ?? "default_695"
    assert(result == "default_695")
}


pub fn main() {
    test_695()
}
