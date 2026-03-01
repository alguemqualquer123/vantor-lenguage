module tests.test_757;

fn test_757() {
    let val: String? = null
    let result = val ?? "default_757"
    assert(result == "default_757")
}


pub fn main() {
    test_757()
}
