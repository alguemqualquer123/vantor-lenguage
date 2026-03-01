import core.io.Console;
import core.collections.List;

fn double(n: i32) -> i32 {
    return n * 2;
}

fn add_one(n: i32) -> i32 {
    return n + 1;
}

fn format_result(n: i32) -> String {
    return "The result is: " + n as String;
}

pub fn main() -> void {
    // Exemplo básico de pipes
    let final_value = 10 
        |> double() 
        |> add_one() 
        |> format_result();
        
    Console.writeLine(final_value);
    
    // Exemplo com listas
    let names = List<String>::new();
    names.add("Lexicon");
    names.add("Rust");
    names.add("Java");
    
    let processed = names 
        |> List::filter(lambda(name: String) -> bool { return name.startsWith("L"); })
        |> List::map(lambda(name: String) -> String { return name.toUpper(); });
        
    Console.writeLine("Processed names count: " + processed.size() as String);
}
