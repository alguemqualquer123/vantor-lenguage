module tests.test_940;

fn test_940() {
    let val: String? = null
    let result = val ?? "default_940"
    assert(result == "default_940")
}


pub fn main() {
    test_940()
}
