module tests.test_68;

fn test_68() {
    let val: String? = null
    let result = val ?? "default_68"
    assert(result == "default_68")
}


pub fn main() {
    test_68()
}
