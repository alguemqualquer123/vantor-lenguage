pub fn main() -> void {
    let mut total: i64 = 0;
    let mut i: i64 = 1;
    while (i <= 1000000) {
        total = total + i;
        i = i + 1;
    }
    println(total);
}
