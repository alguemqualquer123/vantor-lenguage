module tests.test_416;

fn test_416() {
    let val: String? = null
    let result = val ?? "default_416"
    assert(result == "default_416")
}


pub fn main() {
    test_416()
}
