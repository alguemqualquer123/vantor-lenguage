// e2e_044 - while with or-condition
// NOTE: the inner counter MUST assign (`i = i + 1`), not redeclare
// (`let i = …` would shadow per iteration like Go/Rust — infinite loop,
// correctly aborted by the step budget, never exit 0).
pub fn main() -> void {
    let i = 0;
    let stop = false;
    while i < 3 || stop {
        i = i + 1;
        if i > 10 {
            break;
        }
    }
    return;
}
