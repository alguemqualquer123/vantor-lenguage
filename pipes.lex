import core.io.Console;
import core.collections.List;

fn double(n: i32) -> i32 {
    return n * 2;
}

fn add_one(n: i32) -> i32 {
    return n + 1;
}

pub fn main() -> void {
    let result = 5 
        |> double() 
        |> add_one();
        
    Console.writeLine("Result: " + result as String);
    
    let names = List<String>::new();
    names.add("Alice");
    names.add("Bob");
    names.add("Charlie");
    
    let processed = names 
        |> List::filter(lambda(name: String) -> bool { return name.startsWith("A"); })
        |> List::map(lambda(name: String) -> String { return name.toUpper(); });
        
    Console.writeLine("Processed names count: " + processed.size() as String);
}
