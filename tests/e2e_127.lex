// e2e_127 - Go-parity stdlib: os, sync, context, log, filepath
module e2e_127;
import std::os;
import std::sync;
import std::context;
import std::log;
import std::path::filepath;

pub fn main() -> void {
    // os
    assert(os::Getwd() != "", "Getwd");
    os::Setenv("LEX_E2E_127", "42");
    assert(os::Getenv("LEX_E2E_127") == "42", "env roundtrip");
    assert(os::LookupEnv("LEX_E2E_127")[1], "LookupEnv ok");
    let tmp = os::TempDir() + "/lex_e2e_127.txt";
    assert(os::WriteFile(tmp, "hi"), "WriteFile");
    assert(os::ReadFile(tmp) == "hi", "ReadFile");
    assert(os::Exists(tmp), "Exists");
    let fi = os::Stat(tmp);
    assert(fi.size == 2, "Stat size");
    assert(os::Remove(tmp), "Remove");

    // sync (cooperative no-op core, Go API shapes)
    let m = sync::NewMutex();
    m = sync::Lock(m);
    m = sync::Unlock(m);
    let o = sync::NewOnce();
    o = sync::Do(o, | | 0);
    let w = sync::NewWaitGroup();
    w = sync::Add(w, 1);
    w = sync::Done(w);

    // context
    let c = context::Background();
    assert(!context::Done(c), "live");
    let cr = context::WithCancel(c);
    cr.ctx = cr.cancel();
    assert(context::Done(cr.ctx), "cancelled");
    assert(context::Err(cr.ctx) == "Canceled", "Err");
    let v = context::WithValue(c, "k", 7);
    assert(context::Value(v, "k").value == 7, "WithValue");

    // log
    log::SetPrefix("[t] ");
    assert(log::Prefix() == "[t] ", "Prefix");
    log::Print(["hello"]);

    // filepath
    assert(filepath::Base("a/b/c.txt") == "c.txt", "Base");
    assert(filepath::Ext("c.txt") == ".txt", "Ext");
    assert(filepath::Match("*.txt", "c.txt"), "Match");
    assert(!filepath::IsAbs("a/b"), "IsAbs rel");
    return;
}
