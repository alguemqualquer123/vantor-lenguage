// ctx_timeout.lex — cancellation and deadlines with std::context.
// Run: lex run examples/ctx_timeout.lex
import std::context;

pub fn main() -> void {
    let bg = context::Background();
    let cr = context::WithTimeout(bg, 50);
    Console::log(context::Done(cr.ctx));
    cr.ctx = cr.cancel();
    Console::log(context::Done(cr.ctx));
    Console::log(context::Err(cr.ctx));
    let v = context::WithValue(bg, "user", "ada");
    Console::log(context::Value(v, "user").value);
}
