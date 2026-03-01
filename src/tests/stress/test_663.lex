module tests.test_663;

fn test_663() {
    let val: String? = null
    let result = val ?? "default_663"
    assert(result == "default_663")
}


pub fn main() {
    test_663()
}
