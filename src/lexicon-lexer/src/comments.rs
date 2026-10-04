// Comment stripping shared by the compiler mock-runner, route/port
// extraction and tooling (Spec §1: `//` line comments and `/* */`
// block comments).
//
// Contract (matches `tokenizer.rs` exactly):
// - `//` runs to (not including) the next `\n`.
// - `/*` runs to the first `*/` (non-nesting, like C); an unterminated
//   block comment discards the rest of the input.
// - `"..."` and `'...'` literals (with `\` escapes) are opaque: comment
//   markers inside them are DATA, e.g. `"http://x"` or `"a /* b"`,
//   and never start a comment.
// - Newlines inside removed comments are PRESERVED so line numbers of
//   the remaining code never shift (diagnostics stay accurate).
// - Output positions are otherwise stable: every kept character keeps its
//   original order; removed bytes become nothing (never spaces), and
//   newlines inside removed comments are preserved, so line numbers of
//   kept code never shift (diagnostics stay accurate).

/// Strip Lexicon comments, preserving newlines for diagnostic stability.
pub fn strip_comments(source: &str) -> String {
    // Char-based (never byte-based): multibyte UTF-8 sequences pass through
    // untouched, so `"é"` stays one character downstream.
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '"' => {
                // String literal: copy verbatim incl. escapes.
                out.push(c);
                i += 1;
                while i < chars.len() {
                    let d = chars[i];
                    out.push(d);
                    i += 1;
                    if d == '\\' {
                        if i < chars.len() {
                            out.push(chars[i]);
                            i += 1;
                        }
                    } else if d == '"' {
                        break;
                    } else if d == '\n' {
                        // Unterminated string: stop being opaque so the
                        // rest of the line is still scanned normally.
                        break;
                    }
                }
            }
            '\'' => {
                // Char literal: same opacity rules as strings.
                out.push(c);
                i += 1;
                while i < chars.len() {
                    let d = chars[i];
                    out.push(d);
                    i += 1;
                    if d == '\\' {
                        if i < chars.len() {
                            out.push(chars[i]);
                            i += 1;
                        }
                    } else if d == '\'' || d == '\n' {
                        break;
                    }
                }
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '/' => {
                // Line comment: skip to (not including) `\n`.
                i += 2;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '*' => {
                // Block comment: skip to first `*/`, keeping newlines.
                i += 2;
                while i < chars.len() {
                    if chars[i] == '\n' {
                        out.push('\n');
                        i += 1;
                    } else if chars[i] == '*'
                        && i + 1 < chars.len()
                        && chars[i + 1] == '/'
                    {
                        i += 2;
                        break;
                    } else {
                        i += 1;
                    }
                }
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// Convenience: does the (comment-stripped) source contain `needle`
/// outside of comments and literals? Used for server/GUI probing so a
/// commented-out `Http::serve` never boots a server.
pub fn code_contains(source: &str, needle: &str) -> bool {
    strip_comments(source).contains(needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_comments_removed_but_code_kept() {
        let out = strip_comments("let x = 1; // trailing\nprint(x);\n");
        assert_eq!(out, "let x = 1; \nprint(x);\n");
    }

    #[test]
    fn full_line_comment_removed() {
        let out = strip_comments("// print(\"ghost\");\nprint(\"live\");\n");
        assert_eq!(out, "\nprint(\"live\");\n");
    }

    #[test]
    fn block_comments_removed_newlines_kept() {
        let out = strip_comments("a();\n/* one\ntwo */\nb();\n");
        assert_eq!(out, "a();\n\n\nb();\n");
    }

    #[test]
    fn markers_inside_strings_are_data() {
        let out = strip_comments("let u = \"http://x\";\nlet c = \"a /* b\";\nlet d = '//';\n");
        assert_eq!(out, "let u = \"http://x\";\nlet c = \"a /* b\";\nlet d = '//';\n");
    }

    #[test]
    fn unterminated_block_discards_rest() {
        let out = strip_comments("live();\n/* never ends\nprint(\"ghost\");\n");
        assert_eq!(out, "live();\n\n\n");
    }

    #[test]
    fn division_is_not_a_comment() {
        let out = strip_comments("let x = a / b;\nlet y = a / b; // ok\n");
        assert_eq!(out, "let x = a / b;\nlet y = a / b; \n");
    }

    #[test]
    fn code_contains_ignores_comments() {
        assert!(code_contains("Http::serve(\"0.0.0.0:3000\");", "Http::serve"));
        assert!(!code_contains("// Http::serve(\"0.0.0.0:3000\");", "Http::serve"));
        assert!(!code_contains("/* Http::serve */", "Http::serve"));
    }

    #[test]
    fn line_numbers_stable() {
        let src = "a();\n// gone\n/* gone\nstill gone */\nb();\n";
        let stripped = strip_comments(src);
        assert_eq!(stripped.lines().count(), src.lines().count());
        assert_eq!(stripped.lines().nth(4).unwrap(), "b();");
    }
}
