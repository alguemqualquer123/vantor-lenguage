import core.io.Console;

pub fn main() -> void {
    Console.writeLine("=== Testing 6 Bugs ===");
    
    // BUG 1: Pipe operator |>
    let x = 5 |> (fn n => n + 1);
    Console.writeLine("Bug1 - Pipe: " + x as String);
    
    // BUG 2: as cast
    let num = 42;
    let str = num as String;
    Console.writeLine("Bug2 - As cast: " + str);
    
    // BUG 3: Invalid escape (should not corrupt)
    let s = "hello\nworld";
    Console.writeLine("Bug3 - Escape: " + s);
    
    // BUG 4: toString() method
    let arr = [1, 2, 3];
    Console.writeLine("Bug4 - toString: " + arr.toString());
    
    // BUG 5: inspect function
    let config = { port: 8080, host: "localhost" };
    Console.writeLine("Bug5 - inspect: ");
    Console.writeLine(inspect(config));
    
    // BUG 6: Struct shorthand
    let port = 3000;
    let host = "example.com";
    let server = { port, host };
    Console.writeLine("Bug6 - Struct shorthand: " + server.toString());
}
