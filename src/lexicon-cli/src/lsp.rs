//! Minimal Language Server (`lex lsp`).
//!
//! Speaks LSP 3.17 over stdio with hand-rolled JSON-RPC (no new deps —
//! `serde_json` only). Capabilities: `initialize`, `textDocument/completion`
//! (triggered by `:`, `@`, `.`), `textDocument/hover`,
//! `textDocument/definition` (Ctrl+Click → SDK source: real `lib/std`
//! files for `std::` packages, `native/*.lex` signature stubs for native
//! builtins), `shutdown`/`exit`.
//! Documents are tracked via `didOpen`/`didChange`; otherwise the file behind
//! a `file://` URI is read from disk. All items come from [`complete`].

use anyhow::Result;
use std::collections::HashMap;
use std::io::{BufRead, Read, Write};
use std::path::PathBuf;

use crate::complete;

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

fn pct_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.as_bytes().iter().peekable();
    while let Some(&b) = it.next() {
        if b == b'%' {
            let hi = it.next().copied().unwrap_or(b'0');
            let lo = it.next().copied().unwrap_or(b'0');
            let hex = |c: u8| (c as char).to_digit(16).unwrap_or(0) as u8;
            out.push((hex(hi) * 16 + hex(lo)) as char);
        } else {
            out.push(b as char);
        }
    }
    out
}

fn uri_to_path(uri: &str) -> Option<String> {
    let p = uri.strip_prefix("file://")?;
    let p = pct_decode(p);
    // Windows: file:///C:/x → C:/x
    if p.starts_with('/') && p.get(1..3).map(|s| s.chars().nth(1) == Some(':')).unwrap_or(false) {
        return Some(p[1..].to_string());
    }
    Some(p)
}

/// Byte offset of a (line, character) position (character = Unicode scalar).
fn line_text<'a>(text: &'a str, line: usize) -> &'a str {
    text.lines().nth(line).unwrap_or("")
}

fn completion_items(prefix: &str) -> Vec<serde_json::Value> {
    complete::complete(prefix)
        .iter()
        .map(|it| {
            serde_json::json!({
                "label": it.label,
                "kind": it.kind.lsp_kind(),
                "detail": it.detail,
                "documentation": { "kind": "markdown", "value": it.doc },
                "insertText": it.insert,
            })
        })
        .collect()
}

fn hover_word(line: &str, character: usize) -> String {    let chars: Vec<char> = line.chars().collect();
    let mut start = character.min(chars.len());
    while start > 0 {
        let c = chars[start - 1];
        if c.is_alphanumeric() || c == '_' || c == ':' || c == '@' {
            start -= 1;
        } else {
            break;
        }
    }
    let mut end = character.min(chars.len());
    while end < chars.len() {
        let c = chars[end];
        if c.is_alphanumeric() || c == '_' || c == ':' {
            end += 1;
        } else {
            break;
        }
    }
    chars[start..end].iter().collect()
}

/// Word under the cursor for go-to-definition: like [`hover_word`] but
/// also crosses `.`, so `Http.get` resolves exactly like `Http::get`.
/// (Field access on plain variables, e.g. `r.ok`, yields a head that is
/// not a known module and resolves to nothing.)
fn definition_word(line: &str, character: usize) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut start = character.min(chars.len());
    while start > 0 {
        let c = chars[start - 1];
        if c.is_alphanumeric() || c == '_' || c == ':' || c == '.' || c == '@' {
            start -= 1;
        } else {
            break;
        }
    }
    let mut end = character.min(chars.len());
    while end < chars.len() {
        let c = chars[end];
        if c.is_alphanumeric() || c == '_' || c == ':' || c == '.' {
            end += 1;
        } else {
            break;
        }
    }
    chars[start..end].iter().collect()
}

/// Split `Head::tail`, `Head.tail`, `std::a::b` or bare `Head` into
/// (module key, member). Returns `None` for non-module shapes.
fn split_module_word(word: &str) -> Option<(String, String)> {
    let w = word.trim().trim_start_matches('@');
    if let Some(idx) = w.find("::") {
        let (head, rest) = (&w[..idx], &w[idx + 2..]);
        if head == "std" {
            // `std::strings`, `std::container::list`: key = last segment.
            let key = rest.rsplit("::").next().unwrap_or("").to_string();
            if key.is_empty() {
                return None;
            }
            return Some((key, String::new()));
        }
        if head.is_empty() || rest.contains("::") {
            return None;
        }
        return Some((head.to_string(), rest.to_string()));
    }
    if let Some(idx) = w.rfind('.') {
        let (head, member) = (&w[..idx], &w[idx + 1..]);
        if head.is_empty() || member.is_empty() {
            return None;
        }
        return Some((head.to_string(), member.to_string()));
    }
    if w.is_empty() {
        return None;
    }
    Some((w.to_string(), String::new()))
}

/// First line (0-based) declaring `member` in `src`: `pub fn NAME`,
/// `pub struct NAME`, `pub const NAME` (member `""` → file top).
fn find_member_line(src: &str, member: &str) -> Option<usize> {
    if member.is_empty() {
        return Some(0);
    }
    for (i, ln) in src.lines().enumerate() {
        let t = ln.trim();
        for prefix in ["pub fn ", "pub struct ", "pub const ", "const ", "fn "] {
            if let Some(rest) = t.strip_prefix(prefix) {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if name == member {
                    return Some(i);
                }
            }
        }
    }
    None
}

/// Resolve a module word to an SDK source location: `(file path,
/// 0-based line)`. Searches `roots` in order (same order the interpreter
/// runs from); unknown modules/members yield `None`.
pub fn definition_in(word: &str, roots: &[PathBuf]) -> Option<(String, usize)> {
    let (key, member) = split_module_word(word)?;
    let segs = complete::module_sdk_path(&key)?;
    for root in roots {
        let mut rel = PathBuf::new();
        for s in &segs {
            rel.push(s);
        }
        rel.set_extension("lex");
        let cand = root.join(&rel);
        if cand.is_file() {
            let src = std::fs::read_to_string(&cand).ok()?;
            let line = find_member_line(&src, &member).unwrap_or(0);
            return Some((cand.to_string_lossy().to_string(), line));
        }
    }
    None
}

/// `definition_in` over the interpreter's live roots (what `lex run`
/// would load — Ctrl+Click lands where the code runs from).
pub fn definition(word: &str) -> Option<(String, usize)> {
    let roots = crate::interp::module_search_roots();
    definition_in(word, &roots)
}

fn path_to_uri(path: &str) -> String {
    // file:///C:/x on Windows, file:///x elsewhere; spaces percent-encoded
    // (mirrors `uri_to_path` in reverse for ASCII paths).
    let mut p = path.replace('\\', "/");
    if !p.starts_with('/') {
        p = format!("/{}", p);
    }
    let enc: String = p
        .chars()
        .map(|c| {
            if c == ' ' {
                "%20".to_string()
            } else {
                c.to_string()
            }
        })
        .collect();
    format!("file://{}", enc)
}

pub fn serve() -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();
    let mut docs: HashMap<String, String> = HashMap::new();

    loop {
        let msg = match read_message(&mut input)? {
            Some(m) => m,
            None => break,
        };
        let id = msg.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(serde_json::Value::Null);
        let has_id = !id.is_null();

        let mut respond = |result: serde_json::Value| -> Result<()> {
            write_message(
                &mut output,
                &serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            )
        };

        match method {
            "initialize" => {
                respond(serde_json::json!({
                    "capabilities": {
                        "completionProvider": { "triggerCharacters": [":", "@", "."] },
                        "hoverProvider": true,
                        "definitionProvider": true,
                        "textDocumentSync": 1,
                    },
                    "serverInfo": { "name": "lex-lsp", "version": env!("CARGO_PKG_VERSION") },
                }))?;
            }
            "initialized" => {}
            "shutdown" => {
                if has_id {
                    respond(serde_json::Value::Null)?;
                }
            }
            "exit" => break,
            "textDocument/didOpen" => {
                let uri = params.pointer("/textDocument/uri").and_then(|u| u.as_str()).unwrap_or("");
                let text = params.pointer("/textDocument/text").and_then(|t| t.as_str()).unwrap_or("");
                docs.insert(uri.to_string(), text.to_string());
            }
            "textDocument/didChange" => {
                let uri = params.pointer("/textDocument/uri").and_then(|u| u.as_str()).unwrap_or("");
                if let Some(changes) = params.pointer("/contentChanges").and_then(|c| c.as_array()) {
                    if let Some(last) = changes.last().and_then(|c| c.pointer("/text")).and_then(|t| t.as_str()) {
                        docs.insert(uri.to_string(), last.to_string());
                    }
                }
            }
            "textDocument/didClose" => {
                let uri = params.pointer("/textDocument/uri").and_then(|u| u.as_str()).unwrap_or("");
                docs.remove(uri);
            }
            "$/cancelRequest" => {}
            "textDocument/completion" => {
                let uri = params.pointer("/textDocument/uri").and_then(|u| u.as_str()).unwrap_or("");
                let line = params.pointer("/position/line").and_then(|l| l.as_u64()).unwrap_or(0) as usize;
                let character = params.pointer("/position/character").and_then(|c| c.as_u64()).unwrap_or(0) as usize;
                let text: String = match docs.get(uri) {
                    Some(t) => t.clone(),
                    None => uri_to_path(uri).and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default(),
                };
                let l = line_text(&text, line);
                let prefix = complete::prefix_at(l, character);
                let items = completion_items(&prefix);
                if has_id {
                    respond(serde_json::json!({ "isIncomplete": false, "items": items }))?;
                }
            }
            "textDocument/hover" => {
                let uri = params.pointer("/textDocument/uri").and_then(|u| u.as_str()).unwrap_or("");
                let line = params.pointer("/position/line").and_then(|l| l.as_u64()).unwrap_or(0) as usize;
                let character = params.pointer("/position/character").and_then(|c| c.as_u64()).unwrap_or(0) as usize;
                let text: String = match docs.get(uri) {
                    Some(t) => t.clone(),
                    None => uri_to_path(uri).and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default(),
                };
                let l = line_text(&text, line);
                let word = hover_word(l, character);
                let result = complete::hover(&word).map(|(sig, doc)| {
                    serde_json::json!({ "contents": { "kind": "markdown", "value": format!("```lex\n{}\n```\n{}", sig, doc) } })
                }).unwrap_or(serde_json::Value::Null);
                if has_id {
                    respond(result)?;
                }
            }
            "textDocument/definition" => {
                let uri = params.pointer("/textDocument/uri").and_then(|u| u.as_str()).unwrap_or("");
                let line = params.pointer("/position/line").and_then(|l| l.as_u64()).unwrap_or(0) as usize;
                let character = params.pointer("/position/character").and_then(|c| c.as_u64()).unwrap_or(0) as usize;
                let text: String = match docs.get(uri) {
                    Some(t) => t.clone(),
                    None => uri_to_path(uri).and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default(),
                };
                let l = line_text(&text, line);
                let word = definition_word(l, character);
                let result = definition(&word).map(|(path, target)| {
                    let target_text: String = std::fs::read_to_string(&path).unwrap_or_default();
                    let width = line_text(&target_text, target).trim().chars().count().max(1);
                    serde_json::json!({
                        "uri": path_to_uri(&path),
                        "range": {
                            "start": { "line": target, "character": 0 },
                            "end": { "line": target, "character": width },
                        },
                    })
                }).unwrap_or(serde_json::Value::Null);
                if has_id {
                    respond(result)?;
                }
            }
            _ => {
                if has_id {
                    write_message(
                        &mut output,
                        &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": format!("method not found: {}", method) } }),
                    )?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_root(name: &str) -> PathBuf {
        // Unique per test: `cargo test` runs threads in parallel sharing
        // `temp_dir()`, and a shared fixture races remove-vs-read.
        let root = std::env::temp_dir().join(format!("lex-lsp-def-test-{}", name));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("std")).unwrap();
        std::fs::write(
            root.join("std").join("strings.lex"),
            "pub fn ToUpper(s: String) -> String {\n    return s;\n}\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("std").join("native")).unwrap();
        std::fs::write(
            root.join("std").join("native").join("math.lex"),
            "// stubs\n/// Native square root.\npub fn sqrt(x: f64) -> f64 {\n    panic(\"native stub\");\n}\n",
        )
        .unwrap();
        root
    }

    #[test]
    fn split_module_word_shapes() {
        assert_eq!(
            split_module_word("strings::ToUpper"),
            Some(("strings".to_string(), "ToUpper".to_string()))
        );
        assert_eq!(
            split_module_word("Math::"),
            Some(("Math".to_string(), String::new()))
        );
        assert_eq!(
            split_module_word("Http.get"),
            Some(("Http".to_string(), "get".to_string()))
        );
        assert_eq!(
            split_module_word("std::container::list"),
            Some(("list".to_string(), String::new()))
        );
        assert_eq!(
            split_module_word("r.ok"),
            Some(("r".to_string(), "ok".to_string()))
        );
    }

    #[test]
    fn definition_resolves_std_member_line() {
        let root = fixture_root("std");
        let (path, line) = definition_in("strings::ToUpper", &[root.clone()]).expect("found");
        assert!(path.ends_with("strings.lex"), "{}", path);
        assert_eq!(line, 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn definition_resolves_native_stub() {
        let root = fixture_root("native");
        let (path, line) = definition_in("Math::sqrt", &[root.clone()]).expect("found");
        assert!(path.ends_with("math.lex"), "{}", path);
        assert_eq!(line, 2);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn definition_unknown_is_none() {
        let root = fixture_root("unknown");
        assert!(definition_in("Nope::x", &[root.clone()]).is_none());
        // Unknown member falls back to file top (still navigable).
        let (path, line) = definition_in("strings::Missing", &[root.clone()]).expect("file");
        assert!(path.ends_with("strings.lex"), "{}", path);
        assert_eq!(line, 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn definition_word_crosses_dot() {
        assert_eq!(definition_word("    Http.get(url)", 9), "Http.get");
        assert_eq!(definition_word("strings::ToUpper(s)", 5), "strings::ToUpper");
    }

    #[test]
    fn path_uri_round_trip_ascii() {
        let uri = path_to_uri("C:/x/y.lex");
        assert!(uri.starts_with("file:///"), "{}", uri);
        assert_eq!(uri_to_path(&uri).as_deref(), Some("C:/x/y.lex"));
    }
}
