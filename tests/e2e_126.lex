// e2e_126 - Go-parity stdlib: container/list, heap, ring
module e2e_126;
import std::container::list;
import std::container::heap;
import std::container::ring;

pub fn main() -> void {
    // list
    let l = list::New();
    l = list::PushBack(l, 1);
    l = list::PushBack(l, 2);
    l = list::PushFront(l, 0);
    assert(list::Len(l) == 3, "Len");
    assert(list::Front(l) == 0, "Front");
    assert(list::Back(l) == 2, "Back");
    let p = list::PopBack(l);
    assert(p.value == 2 && p.ok, "PopBack");
    l = p.list;
    assert(list::Len(l) == 2, "Len after pop");

    // heap (min-heap via lambda comparator stored in the value)
    let h = heap::New(|a, b| a < b);
    h = heap::Push(h, 3);
    h = heap::Push(h, 1);
    h = heap::Push(h, 2);
    assert(heap::Peek(h) == 1, "Peek");
    let q = heap::Pop(h);
    assert(q.value == 1 && q.ok, "Pop min");
    assert(heap::Peek(q.heap) == 2, "Peek after pop");

    // ring
    let r = ring::New([1, 2, 3]);
    assert(ring::Value(r) == 1, "Value");
    r = ring::Next(r);
    assert(ring::Value(r) == 2, "Next");
    r = ring::Set(r, 99);
    assert(ring::Value(r) == 99, "Set");
    r = ring::Next(r);
    r = ring::Next(r);
    assert(ring::Value(r) == 1, "wrap");
    return;
}
