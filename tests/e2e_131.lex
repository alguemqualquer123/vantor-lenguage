// e2e_131 - Go-parity stdlib: os/exec, sync/atomic
module e2e_131;
import std::os::exec;
import std::sync::atomic;

pub fn main() -> void {
    // atomic cells (native registry)
    let a = atomic::NewInt64(5);
    assert(atomic::Load(a) == 5, "Load");
    assert(atomic::Add(a, 3) == 8, "Add");
    assert(atomic::CompareAndSwap(a, 8, 9), "CAS ok");
    assert(!atomic::CompareAndSwap(a, 8, 10), "CAS mismatch");
    assert(atomic::Load(a) == 9, "Load after CAS");
    assert(atomic::Swap(a, 1) == 9, "Swap returns old");
    atomic::Store(a, 42);
    assert(atomic::Load(a) == 42, "Store");

    // exec (only where a shell resolves; portable skip elsewhere)
    let shell = exec::LookPath("cmd");
    if shell != "" {
        let o = exec::Output(exec::Command("cmd", ["/c", "echo ok"]));
        assert(o.ok && o.code == 0, "echo ok");
        assert(o.out.trim() == "ok", "echo out");
        let bad = exec::Output(exec::Command("lex-no-such-prog-xyz", []));
        assert(!bad.ok && bad.code == -1, "missing prog");
    }
    return;
}
