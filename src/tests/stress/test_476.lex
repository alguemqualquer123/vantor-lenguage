module tests.test_476;

fn test_476() {
    let val: String? = null
    let result = val ?? "default_476"
    assert(result == "default_476")
}


pub fn main() {
    test_476()
}
