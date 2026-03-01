module tests.test_745;

fn test_745() {
    let val: String? = null
    let result = val ?? "default_745"
    assert(result == "default_745")
}


pub fn main() {
    test_745()
}
