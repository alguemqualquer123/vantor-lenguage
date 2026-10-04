// run_cmd.lex — subprocesses with std::os::exec.
// Run: lex run examples/run_cmd.lex
import std::os::exec;

pub fn main() -> void {
    let where_cmd = "cmd";
    let argv = ["/c", "echo from-lex"];
    if exec::LookPath("sh") != "" {
        where_cmd = "sh";
        argv = ["-c", "echo from-lex"];
    }
    let o = exec::Output(exec::Command(where_cmd, argv));
    Console::log("ok=" + o.ok.to_string() + " out=" + o.out.trim());
}
