pub fn main() -> void {
    let items = [1, 2, 3, 4, 5, 10];
    let total = 0;
    for x in items {
        let total = total + x;
        print("Total: ", total, "\n");
    }
    print(total);
    return;
}
