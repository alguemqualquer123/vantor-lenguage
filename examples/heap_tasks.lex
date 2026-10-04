// heap_tasks.lex — priority queue with a lambda comparator.
// Run: lex run examples/heap_tasks.lex
import std::container::heap;

pub fn main() -> void {
    let h = heap::New(|a, b| a < b);
    h = heap::Push(h, 5);
    h = heap::Push(h, 1);
    h = heap::Push(h, 3);
    while heap::Len(h) > 0 {
        let p = heap::Pop(h);
        Console::log(p.value);
        h = p.heap;
    }
}
