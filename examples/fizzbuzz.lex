// fizzbuzz.lex — control flow and string building.
// Run: lex run examples/fizzbuzz.lex
pub fn main() -> void {
    let i = 1;
    while i <= 20 {
        let out = "";
        if i % 3 == 0 {
            out = out + "Fizz";
        }
        if i % 5 == 0 {
            out = out + "Buzz";
        }
        if out == "" {
            out = i.to_string();
        }
        Console::log(out);
        i = i + 1;
    }
}
