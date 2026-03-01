module tests.test_794;

fn test_794() {
    let val: String? = null
    let result = val ?? "default_794"
    assert(result == "default_794")
}


pub fn main() {
    test_794()
}
