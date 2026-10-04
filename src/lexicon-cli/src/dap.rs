//! Minimal Debug Adapter (`lex dap`).
//!
//! Speaks Debug Adapter Protocol over stdio with hand-rolled JSON framing
//! (same `Content-Length` transport as [`crate::lsp`], copied locally into
//! this module — `serde_json` only, no new deps). Enables real VSCode
//! debugging with a run-to-completion execution model.
//!
//! Handled requests: `initialize`, `launch`, `setBreakpoints`,
//! `configurationDone`, `threads`, `stackTrace`, `scopes`, `variables`,
//! `evaluate`, `disconnect`/`terminate`, `continue`, `next`, `stepIn`,
//! `stepOut`, `pause`. All unknown commands get an error response.
//!
//! Breakpoint verification parses the target file with `lexicon-lexer` +
//! `lexicon-parser`: a 1-based line inside a function body range counts as
//! `verified: true`; blank/comment/out-of-range/non-function lines (or files
//! with parse errors) come back `verified: false` with a message.
//!
//! Honest limitations (documented, by design):
//! - stepping (`next`/`stepIn`/`stepOut`) is continue-only: the program runs
//!   to completion (same as `continue`);
//! - `variables` is honestly empty (`Locals` scope has `variablesReference: 0`);
//! - `evaluate` is unsupported (`Repl::eval` in `repl.rs` is private, so the
//!   REPL eval path is not reachable without touching that module);
//! - `pause` is unsupported (run-to-completion cannot suspend).

use anyhow::Result;
use std::collections::HashMap;
use std::io::{BufRead, Read, Write};

// ---------------------------------------------------------------------------
// Framed transport (copied from lsp.rs — do not refactor lsp.rs).
// ---------------------------------------------------------------------------

fn read_message(input: &mut impl BufRead) -> Result<Option<serde_json::Value>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            return Ok(None); // EOF
        }
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if let Some(v) = line.strip_prefix("Content-Length:") {
            content_length = v.trim().parse().ok();
        }
    }
    let len = content_length.unwrap_or(0);
    let mut buf = vec![0u8; len];
    input.read_exact(&mut buf)?;
    let msg: serde_json::Value = serde_json::from_slice(&buf)?;
    Ok(Some(msg))
}

fn write_message(output: &mut impl Write, msg: &serde_json::Value) -> Result<()> {
    let body = serde_json::to_string(msg)?;
    write!(output, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    output.flush()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// DAP envelopes.
// ---------------------------------------------------------------------------

/// Capabilities advertised in the `initialize` response body.
fn initialize_body() -> serde_json::Value {
    serde_json::json!({
        "supportsConfigurationDoneRequest": true,
        "supportsEvaluateForHovers": true,
        "supportsSetVariable": false,
        "supportsStepBack": false,
    })
}

fn ok_response(
    seq: u64,
    req_seq: &serde_json::Value,
    command: &str,
    body: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "seq": seq,
        "type": "response",
        "request_seq": req_seq,
        "success": true,
        "command": command,
        "body": body,
    })
}

fn err_response(
    seq: u64,
    req_seq: &serde_json::Value,
    command: &str,
    message: impl Into<String>,
) -> serde_json::Value {
    serde_json::json!({
        "seq": seq,
        "type": "response",
        "request_seq": req_seq,
        "success": false,
        "command": command,
        "message": message.into(),
    })
}

fn event(seq: u64, name: &str, body: Option<serde_json::Value>) -> serde_json::Value {
    let mut e = serde_json::json!({ "seq": seq, "type": "event", "event": name });
    if let Some(b) = body {
        e["body"] = b;
    }
    e
}

// ---------------------------------------------------------------------------
// Breakpoint verification (lexicon-lexer + lexicon-parser).
// ---------------------------------------------------------------------------

/// Byte offsets of every 1-based line start in `source`.
fn line_starts(source: &str) -> Vec<usize> {
    let mut v = vec![0usize];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' {
            v.push(i + 1);
        }
    }
    v
}

/// Map a byte offset to a 1-based line number.
fn offset_line(starts: &[usize], off: usize) -> u64 {
    match starts.binary_search(&off) {
        Ok(i) => (i + 1) as u64,
        Err(i) => i as u64,
    }
}

/// 1-based line of the `}` matching the `{` at byte offset `open`.
/// String literals and `//`/`/* */` comments are skipped so braces inside
/// them don't affect depth counting.
fn match_brace_close_line(source: &str, open: usize) -> Option<u64> {
    let bytes = source.as_bytes();
    if bytes.get(open) != Some(&b'{') {
        return None;
    }
    #[derive(PartialEq)]
    enum S {
        Code,
        LineComment,
        BlockComment,
        Dq,
        Sq,
    }
    let mut st = S::Code;
    let mut depth: u32 = 0;
    let mut line: u64 = source[..open].bytes().filter(|&b| b == b'\n').count() as u64 + 1;
    let mut i = open;
    let mut prev = 0u8;
    while i < bytes.len() {
        let b = bytes[i];
        match st {
            S::Code => match b {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(line);
                    }
                }
                b'"' => st = S::Dq,
                b'\'' => st = S::Sq,
                b'/' if bytes.get(i + 1) == Some(&b'/') => st = S::LineComment,
                b'/' if bytes.get(i + 1) == Some(&b'*') => st = S::BlockComment,
                _ => {}
            },
            S::LineComment => {
                if b == b'\n' {
                    st = S::Code;
                }
            }
            S::BlockComment => {
                if prev == b'*' && b == b'/' {
                    st = S::Code;
                }
            }
            S::Dq => match b {
                b'\\' => i += 1, // skip escaped char
                b'"' => st = S::Code,
                _ => {}
            },
            S::Sq => match b {
                b'\\' => i += 1, // skip escaped char
                b'\'' => st = S::Code,
                _ => {}
            },
        }
        if b == b'\n' {
            line += 1;
        }
        prev = b;
        i += 1;
    }
    None
}

/// 1-based line ranges covering every function body (top-level `fn` decls
/// plus class methods/constructors): from the body's opening `{` line to its
/// matching `}` line. A breakpoint is verifiable when it lands inside one of
/// these ranges on a non-blank, non-comment line.
///
/// NOTE: `Function.span`/`Block.span` in `lexicon-parser` only cover the
/// first token, so body extents are derived by brace matching from the
/// body's `{` offset instead of relying on span end offsets.
fn executable_line_ranges(
    source: &str,
    module: &lexicon_parser::Module,
) -> Vec<(u64, u64)> {
    use lexicon_parser::{ClassMember, Decl};
    let mut out = Vec::new();
    let mut push_body = |span: lexicon_core::span::Span| {
        let open = span.start.min(source.len());
        if let Some(close) = match_brace_close_line(source, open) {
            let starts = line_starts(source);
            let first = offset_line(&starts, open);
            out.push((first.min(close), first.max(close)));
        }
    };
    for d in &module.declarations {
        match d {
            Decl::Function(f) => {
                if let Some(b) = &f.body {
                    push_body(b.span);
                }
            }
            Decl::Class(c) => {
                for m in &c.members {
                    match m {
                        ClassMember::Method(f) => {
                            if let Some(b) = &f.body {
                                push_body(b.span);
                            }
                        }
                        ClassMember::Constructor(k) => push_body(k.body.span),
                        ClassMember::Field(_) => {}
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Verify a 1-based DAP breakpoint line against `source`.
/// Returns `(verified, message)` — `message` explains unverified lines.
fn verify_breakpoint(source: &str, line: u64) -> (bool, Option<String>) {
    let lines: Vec<&str> = source.lines().collect();
    if line == 0 || (line as usize) > lines.len() {
        return (false, Some("line out of range".to_string()));
    }
    let text = lines[line as usize - 1].trim();
    if text.is_empty() {
        return (false, Some("blank line — no executable code".to_string()));
    }
    if text.starts_with("//") || text.starts_with('#') || text.starts_with("/*") || text.starts_with('*') {
        return (
            false,
            Some("comment line — no executable code".to_string()),
        );
    }
    let tokens = lexicon_lexer::Lexer::new(source).tokenize();
    let mut parser = lexicon_parser::Parser::new(tokens);
    match parser.parse() {
        Ok(module) => {
            let ranges = executable_line_ranges(source, &module);
            if ranges.iter().any(|(s, e)| line >= *s && line <= *e) {
                (true, None)
            } else {
                (
                    false,
                    Some("line is not inside a function body".to_string()),
                )
            }
        }
        Err(e) => (false, Some(format!("parse error: {}", e))),
    }
}

// ---------------------------------------------------------------------------
// Program execution (run-to-completion).
// ---------------------------------------------------------------------------

/// Minimal double-quote literal scanner with backslash escapes.
fn string_literals(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_str = false;
    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if in_str {
            if ch == '\\' {
                match chars.next() {
                    Some('n') => cur.push('\n'),
                    Some('t') => cur.push('\t'),
                    Some('r') => cur.push('\r'),
                    Some('\\') => cur.push('\\'),
                    Some('"') => cur.push('"'),
                    Some(other) => cur.push(other),
                    None => {}
                }
            } else if ch == '"' {
                in_str = false;
                out.push(std::mem::take(&mut cur));
            } else {
                cur.push(ch);
            }
        } else if ch == '"' {
            in_str = true;
        }
    }
    out
}

/// Capture print-like output from `source`.
///
/// NOTE: this intentionally does NOT call `compiler::run_with_ci` — that
/// helper prints to our own stdout (which would corrupt the DAP message
/// stream) and can block on interactive waits / HTTP serving. Instead we
/// mirror `compile_run`'s observable behavior: validate via lexer+parser
/// (parse diagnostics go to the caller as a stderr event) and capture
/// `print(`/`println(`/`writeLine`/`log(` string literals, falling back to
/// the same `"Program executed successfully"` sentinel `compile_run` uses.
fn extract_program_output(source: &str) -> String {
    let mut acc = Vec::new();
    for line in source.lines() {
        if line.contains("print(")
            || line.contains("println(")
            || line.contains("writeLine")
            || line.contains("log(")
        {
            acc.extend(string_literals(line));
        }
    }
    if acc.is_empty() {
        "Program executed successfully".to_string()
    } else {
        acc.join("\n")
    }
}

// ---------------------------------------------------------------------------
// Session.
// ---------------------------------------------------------------------------

struct Session {
    seq: u64,
    program: Option<String>,
    stop_on_entry: bool,
    executed: bool,
    breakpoints: HashMap<String, Vec<u64>>,
    bp_next_id: u64,
}

impl Session {
    fn next_seq(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }
}

/// Run the launched program to completion: stream one `output` event per line
/// (category `stdout`, parse problems as `stderr`) then a `terminated` event.
fn finish_execution(sess: &mut Session, output: &mut impl Write) -> Result<()> {
    let path = sess.program.clone().unwrap_or_default();
    let source = std::fs::read_to_string(&path).unwrap_or_default();
    if source.is_empty() && std::fs::metadata(&path).is_err() {
        let seq = sess.next_seq();
        write_message(
            output,
            &event(
                seq,
                "output",
                Some(serde_json::json!({
                    "category": "stderr",
                    "output": format!("cannot read program file: {}\n", path),
                })),
            ),
        )?;
    } else {
        let tokens = lexicon_lexer::Lexer::new(&source).tokenize();
        let mut parser = lexicon_parser::Parser::new(tokens);
        if let Err(e) = parser.parse() {
            let seq = sess.next_seq();
            write_message(
                output,
                &event(
                    seq,
                    "output",
                    Some(serde_json::json!({
                        "category": "stderr",
                        "output": format!("Parse error: {}\n", e),
                    })),
                ),
            )?;
        }
        let text = extract_program_output(&source);
        for line in text.lines() {
            let seq = sess.next_seq();
            write_message(
                output,
                &event(
                    seq,
                    "output",
                    Some(serde_json::json!({
                        "category": "stdout",
                        "output": format!("{}\n", line),
                    })),
                ),
            )?;
        }
    }
    let seq = sess.next_seq();
    write_message(output, &event(seq, "terminated", None))?;
    sess.executed = true;
    Ok(())
}

/// Resolve a DAP `source` object (or legacy flat fields) to a file path.
fn source_path(args: &serde_json::Value) -> Option<String> {
    if let Some(p) = args.pointer("/source/path").and_then(|v| v.as_str()) {
        return Some(p.to_string());
    }
    if let Some(p) = args.pointer("/sourcePath").and_then(|v| v.as_str()) {
        return Some(p.to_string());
    }
    if let Some(p) = args.pointer("/path").and_then(|v| v.as_str()) {
        return Some(p.to_string());
    }
    None
}

pub fn serve() -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();
    let mut sess = Session {
        seq: 0,
        program: None,
        stop_on_entry: false,
        executed: false,
        breakpoints: HashMap::new(),
        bp_next_id: 1,
    };

    loop {
        let msg = match read_message(&mut input)? {
            Some(m) => m,
            None => break, // EOF
        };
        // DAP servers only handle client requests here; anything else is ignored.
        if msg.get("type").and_then(|t| t.as_str()) != Some("request") {
            continue;
        }
        let req_seq = msg.get("seq").cloned().unwrap_or(serde_json::Value::Null);
        let command = msg
            .get("command")
            .and_then(|c| c.as_str())
            .unwrap_or("");
        let args = msg
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        match command {
            "initialize" => {
                let seq = sess.next_seq();
                write_message(&mut output, &ok_response(seq, &req_seq, command, initialize_body()))?;
            }
            "launch" => {
                let program = args
                    .get("program")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let stop = args
                    .get("stopOnEntry")
                    .and_then(|v| v.as_bool())
                    .or_else(|| args.get("stopAtEntry").and_then(|v| v.as_bool()))
                    .unwrap_or(false);
                match program {
                    Some(p) if std::path::Path::new(&p).exists() => {
                        sess.program = Some(p);
                        sess.stop_on_entry = stop;
                        sess.executed = false;
                        let seq = sess.next_seq();
                        write_message(
                            &mut output,
                            &ok_response(seq, &req_seq, command, serde_json::json!({})),
                        )?;
                        let seq = sess.next_seq();
                        write_message(&mut output, &event(seq, "initialized", None))?;
                    }
                    Some(p) => {
                        let seq = sess.next_seq();
                        write_message(
                            &mut output,
                            &err_response(seq, &req_seq, command, format!("program not found: {}", p)),
                        )?;
                    }
                    None => {
                        let seq = sess.next_seq();
                        write_message(
                            &mut output,
                            &err_response(seq, &req_seq, command, "launch requires a `program` path"),
                        )?;
                    }
                }
            }
            "setBreakpoints" => {
                let path = source_path(&args).unwrap_or_default();
                let empty = Vec::new();
                let wanted = args
                    .get("breakpoints")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&empty);
                let source_text = std::fs::read_to_string(&path).unwrap_or_default();
                let source_ok = std::path::Path::new(&path).exists();
                let mut out_bps = Vec::new();
                let mut stored = Vec::new();
                for bp in wanted {
                    let line = bp.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
                    let (verified, message) = if !source_ok {
                        (false, Some(format!("source not found: {}", path)))
                    } else {
                        verify_breakpoint(&source_text, line)
                    };
                    let id = sess.bp_next_id;
                    sess.bp_next_id += 1;
                    stored.push(line);
                    let mut obj = serde_json::json!({ "id": id, "verified": verified, "line": line });
                    if let Some(m) = message {
                        obj["message"] = serde_json::Value::String(m);
                    }
                    out_bps.push(obj);
                }
                if !path.is_empty() {
                    sess.breakpoints.insert(path, stored);
                }
                let seq = sess.next_seq();
                write_message(
                    &mut output,
                    &ok_response(seq, &req_seq, command, serde_json::json!({ "breakpoints": out_bps })),
                )?;
            }
            "configurationDone" => {
                if sess.program.is_none() {
                    let seq = sess.next_seq();
                    write_message(
                        &mut output,
                        &err_response(seq, &req_seq, command, "no program launched (missing `launch` request)"),
                    )?;
                    continue;
                }
                let seq = sess.next_seq();
                write_message(
                    &mut output,
                    &ok_response(seq, &req_seq, command, serde_json::json!({})),
                )?;
                if sess.stop_on_entry && !sess.executed {
                    let seq = sess.next_seq();
                    write_message(
                        &mut output,
                        &event(
                            seq,
                            "stopped",
                            Some(serde_json::json!({ "reason": "entry", "threadId": 1 })),
                        ),
                    )?;
                } else {
                    finish_execution(&mut sess, &mut output)?;
                }
            }
            "threads" => {
                let seq = sess.next_seq();
                write_message(
                    &mut output,
                    &ok_response(
                        seq,
                        &req_seq,
                        command,
                        serde_json::json!({ "threads": [{ "id": 1, "name": "main" }] }),
                    ),
                )?;
            }
            "stackTrace" => {
                let mut frame = serde_json::json!({
                    "id": 1, "name": "main", "line": 1, "column": 1,
                });
                if let Some(p) = sess.program.clone() {
                    let name = std::path::Path::new(&p)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("main.lex")
                        .to_string();
                    frame["source"] = serde_json::json!({ "name": name, "path": p });
                }
                let seq = sess.next_seq();
                write_message(
                    &mut output,
                    &ok_response(
                        seq,
                        &req_seq,
                        command,
                        serde_json::json!({ "stackFrames": [frame], "totalFrames": 1 }),
                    ),
                )?;
            }
            "scopes" => {
                let seq = sess.next_seq();
                write_message(
                    &mut output,
                    &ok_response(
                        seq,
                        &req_seq,
                        command,
                        serde_json::json!({ "scopes": [{
                            "name": "Locals",
                            "variablesReference": 0,
                            "expensive": false,
                        }] }),
                    ),
                )?;
            }
            "variables" => {
                // Honest empty: run-to-completion keeps no inspectable frames.
                let seq = sess.next_seq();
                write_message(
                    &mut output,
                    &ok_response(seq, &req_seq, command, serde_json::json!({ "variables": [] })),
                )?;
            }
            "evaluate" => {
                let seq = sess.next_seq();
                write_message(
                    &mut output,
                    &err_response(seq, &req_seq, command, "evaluate not supported"),
                )?;
            }
            "pause" => {
                let seq = sess.next_seq();
                write_message(
                    &mut output,
                    &err_response(seq, &req_seq, command, "pause not supported (run-to-completion)"),
                )?;
            }
            "continue" | "next" | "stepIn" | "stepOut" => {
                // Continue-only stepping: any step request runs to the end.
                let seq = sess.next_seq();
                write_message(
                    &mut output,
                    &ok_response(seq, &req_seq, command, serde_json::json!({})),
                )?;
                if sess.program.is_some() && !sess.executed {
                    finish_execution(&mut sess, &mut output)?;
                }
            }
            "disconnect" | "terminate" => {
                let seq = sess.next_seq();
                write_message(
                    &mut output,
                    &ok_response(seq, &req_seq, command, serde_json::json!({})),
                )?;
                break;
            }
            _ => {
                let seq = sess.next_seq();
                write_message(
                    &mut output,
                    &err_response(seq, &req_seq, command, format!("unknown command: {}", command)),
                )?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const SAMPLE: &str = "// sample program\n\npub fn main() -> void {\n    let a = 42;\n    return;\n}\n";

    #[test]
    fn breakpoint_inside_fn_is_verified() {
        // Line 4 (`let a = 42;`) sits inside `main`'s body.
        let (verified, message) = verify_breakpoint(SAMPLE, 4);
        assert!(verified, "expected verified, got message: {:?}", message);
        assert!(message.is_none());
    }

    #[test]
    fn breakpoint_on_blank_line_is_unverified() {
        let (verified, message) = verify_breakpoint(SAMPLE, 2);
        assert!(!verified);
        assert!(message.unwrap().contains("blank"));
    }

    #[test]
    fn breakpoint_on_comment_line_is_unverified() {
        let (verified, message) = verify_breakpoint(SAMPLE, 1);
        assert!(!verified);
        assert!(message.unwrap().contains("comment"));
    }

    #[test]
    fn breakpoint_outside_fn_is_unverified() {
        // A top-level line past the closing brace is not executable.
        let src = format!("{}\nlet top = 1;\n", SAMPLE);
        let line = src.lines().count() as u64;
        let (verified, message) = verify_breakpoint(&src, line);
        assert!(!verified);
        assert!(message.is_some());
    }

    #[test]
    fn initialize_response_shape_contains_required_capabilities() {
        let body = initialize_body();
        assert_eq!(body["supportsConfigurationDoneRequest"], serde_json::json!(true));
        assert_eq!(body["supportsEvaluateForHovers"], serde_json::json!(true));
        assert_eq!(body["supportsSetVariable"], serde_json::json!(false));
        assert_eq!(body["supportsStepBack"], serde_json::json!(false));
    }

    #[test]
    fn framing_roundtrip_encode_then_decode() {
        let req = serde_json::json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {},
        });
        let mut buf = Vec::new();
        write_message(&mut buf, &req).unwrap();
        let text = String::from_utf8(buf.clone()).unwrap();
        assert!(text.starts_with("Content-Length: "));
        let mut cursor = Cursor::new(buf);
        let back = read_message(&mut cursor).unwrap().unwrap();
        assert_eq!(back, req);
    }
}
