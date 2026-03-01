module tests.test_921;

fn test_921() {
    let result = 64 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_921()
}
