import core.io.Console;
import core.collections.List;

/// Doc comment for the module
//! inner doc
// region: main logic

#[cfg(feature = "http")]
@Get("/users")
struct User {
    name: String,
    age: i32,
    active: bool,
}

enum Color { Red, Green, Blue }

const MAX: i32 = 100;
static PI: f64 = 3.14159;

pub fn double(n: i32) -> i32 {
    return n * 2;
}

async fn fetch(url: String) -> Result<String> {
    let hex = 0xFF;
    let bin = 0b1010;
    let names = List<String>::new();
    names.add("Alice");
    let processed = names
        |> List::filter(lambda(name: String) -> bool { return name.startsWith("A"); })
        |> List::map(lambda(name: String) -> String { return name.toUpper(); });
    for item in processed {
        if item != null && item.size() > 0 {
            println("Item: {item}");
        }
    }
    match hex {
        255 => Console.writeLine("max"),
        _ => Console.writeLine("other"),
    }
    try {
        Console.writeLine("ok");
    } catch (e) {
        panic("failed");
    }
    return Ok(url);
}
