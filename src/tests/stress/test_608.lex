module tests.test_608;

fn test_608() {
    let val: String? = null
    let result = val ?? "default_608"
    assert(result == "default_608")
}


pub fn main() {
    test_608()
}
