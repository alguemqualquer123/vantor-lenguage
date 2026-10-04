pub fn main() -> void {
    let list = new List<i32>();

    list.add(1);
    list.add(2);
    list.add(3);
    let items = [1, 2, 3, 4, 5, 10, 252, "Hello, World!", 3.14, true];
    let total = 0;
    for x in items {
        Console::writeLine(x, typeOf(x));
        if (typeOf(x) == typeOf(total)) {
            total += x;
        }
    
    }
    Console::writeLine("Total: ", total, typeOf(items));

    Console::writeLine("List", list, "Size: ", list.size(), "Capacity: ", list.capacity());
}

