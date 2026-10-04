// Lexicon Standard Library — strconv.
// Go-parity conversions between strings and basic types
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1).
//
// Go's `(T, error)` multi-return becomes a result struct (`ParseIntResult`,
// `ParseFloatResult`) because Lex enums do not yet carry payloads
// (see plan, Fase 8 — runtime Go-parity).

import std::strings;

const DIGITS = "0123456789abcdefghijklmnopqrstuvwxyz";

/// Result of an integer parse: `ok` reports success, `err` the Go-style message.
pub struct ParseIntResult {
    value: i64,
    ok: bool,
    err: String,
}

/// Result of a float parse.
pub struct ParseFloatResult {
    value: f64,
    ok: bool,
    err: String,
}

fn strconv_digitVal(c: String) -> i64 {
    return strings::Index(DIGITS, c.to_lower());
}

/// Returns the string representation of `i` in `base` (2..36, like Go).
pub fn FormatInt(i: i64, base: i64) -> String {
    if base < 2 || base > 36 {
        return "strconv: unsupported base";
    }
    if i == 0 {
        return "0";
    }
    let digits = "";
    let v = i;
    if v < 0 {
        v = 0 - v;
    }
    while v > 0 {
        let d = v % base;
        digits = DIGITS.char_at(d) + digits;
        v = v / base;
    }
    if i < 0 {
        return "-" + digits;
    }
    return digits;
}

/// Returns the decimal string representation of `i`.
pub fn Itoa(i: i64) -> String {
    return FormatInt(i, 10);
}

/// Parses `s` as an integer in the given `base` (2..36), with optional sign.
pub fn ParseInt(s: String, base: i64) -> ParseIntResult {
    let fail = ParseIntResult { value: 0, ok: false, err: "" };
    if base < 2 || base > 36 {
        fail.err = "strconv: unsupported base";
        return fail;
    }
    let n = s.len();
    if n == 0 {
        fail.err = "strconv: parsing empty string";
        return fail;
    }
    let i = 0;
    let neg = false;
    let c0 = s.char_at(0) as String;
    if c0 == "-" || c0 == "+" {
        neg = c0 == "-";
        i = 1;
        if n == 1 {
            fail.err = "strconv: no digits";
            return fail;
        }
    }
    let v = 0;
    while i < n {
        let d = strconv_digitVal(s.char_at(i) as String);
        if d < 0 || d >= base {
            fail.err = "strconv: invalid syntax";
            return fail;
        }
        v = v * base + d;
        i = i + 1;
    }
    if neg {
        v = 0 - v;
    }
    return ParseIntResult { value: v, ok: true, err: "" };
}

/// Parses `s` as a base-10 integer (Go's `strconv.Atoi`).
pub fn Atoi(s: String) -> ParseIntResult {
    return ParseInt(s, 10);
}

/// Returns "true" or "false" (Go's `strconv.FormatBool`).
pub fn FormatBool(b: bool) -> String {
    if b {
        return "true";
    }
    return "false";
}

/// Parses a boolean in Go's accepted spellings.
pub struct ParseBoolResult { value: bool, ok: bool, err: String }

pub fn ParseBool(s: String) -> ParseBoolResult {
    if s == "1" || s == "t" || s == "T" || s == "true" || s == "TRUE" || s == "True" {
        return ParseBoolResult { value: true, ok: true, err: "" };
    }
    if s == "0" || s == "f" || s == "F" || s == "false" || s == "FALSE" || s == "False" {
        return ParseBoolResult { value: false, ok: true, err: "" };
    }
    return ParseBoolResult { value: false, ok: false, err: "strconv: invalid syntax" };
}

fn strconv_pow10(n: i64) -> f64 {
    let p = 1.0;
    let i = 0;
    while i < n {
        p = p * 10.0;
        i = i + 1;
    }
    return p;
}

/// Parses `s` as a floating-point number: `[+-] digits [. digits] [e[+-]digits]`.
pub fn ParseFloat(s: String) -> ParseFloatResult {
    let fail = ParseFloatResult { value: 0.0, ok: false, err: "" };
    let n = s.len();
    if n == 0 {
        fail.err = "strconv: parsing empty string";
        return fail;
    }
    let i = 0;
    let neg = false;
    let c0 = s.char_at(0) as String;
    if c0 == "-" || c0 == "+" {
        neg = c0 == "-";
        i = 1;
    }
    let intPart = 0.0;
    let digitsSeen = false;
    while i < n {
        let d = strconv_digitVal(s.char_at(i) as String);
        if d < 0 || d >= 10 {
            break;
        }
        intPart = intPart * 10.0 + d;
        digitsSeen = true;
        i = i + 1;
    }
    let fracPart = 0.0;
    let fracDigits = 0;
    if i < n && (s.char_at(i) as String) == "." {
        i = i + 1;
        while i < n {
            let d = strconv_digitVal(s.char_at(i) as String);
            if d < 0 || d >= 10 {
                break;
            }
            fracPart = fracPart * 10.0 + d;
            fracDigits = fracDigits + 1;
            digitsSeen = true;
            i = i + 1;
        }
    }
    if !digitsSeen {
        fail.err = "strconv: invalid syntax";
        return fail;
    }
    let v = intPart + fracPart / strconv_pow10(fracDigits);
    if i < n && ((s.char_at(i) as String) == "e" || (s.char_at(i) as String) == "E") {
        i = i + 1;
        let expNeg = false;
        if i < n {
            let ce = s.char_at(i) as String;
            if ce == "-" || ce == "+" {
                expNeg = ce == "-";
                i = i + 1;
            }
        }
        let expDigits = 0;
        let expSeen = false;
        while i < n {
            let d = strconv_digitVal(s.char_at(i) as String);
            if d < 0 || d >= 10 {
                break;
            }
            expDigits = expDigits * 10 + d;
            expSeen = true;
            i = i + 1;
        }
        if !expSeen {
            fail.err = "strconv: invalid syntax";
            return fail;
        }
        if expNeg {
            v = v / strconv_pow10(expDigits);
        } else {
            v = v * strconv_pow10(expDigits);
        }
    }
    if i < n {
        fail.err = "strconv: invalid syntax";
        return fail;
    }
    if neg {
        v = 0.0 - v;
    }
    return ParseFloatResult { value: v, ok: true, err: "" };
}

/// Returns the string form of `f` (Go's `%v` formatting, via the runtime).
pub fn FormatFloat(f: f64) -> String {
    return f.to_string();
}

/// Returns a double-quoted Go-style quoted string.
pub fn Quote(s: String) -> String {
    let out = "\"";
    let i = 0;
    let n = s.len();
    while i < n {
        let c = s.char_at(i);
        let cs = c as String;
        if cs == "\"" {
            out = out + "\\\"";
        } else {
            if cs == "\\" {
                out = out + "\\\\";
            } else {
                if cs == "\n" {
                    out = out + "\\n";
                } else {
                    if cs == "\t" {
                        out = out + "\\t";
                    } else {
                        if cs == "\r" {
                            out = out + "\\r";
                        } else {
                            out = out + cs;
                        }
                    }
                }
            }
        }
        i = i + 1;
    }
    out = out + "\"";
    return out;
}

/// Unquote result: the decoded `text` plus `ok`.
pub struct UnquoteResult {
    text: String,
    ok: bool,
}

/// Unquotes a double-quoted Go string literal (Go's `strconv.Unquote`,
/// common-escape subset: \\ \" \n \t \r).
pub fn Unquote(s: String) -> UnquoteResult {
    let fail = UnquoteResult { text: "", ok: false };
    if s.len() < 2 || Text::slice(s, 0, 1) != "\"" || Text::slice(s, s.len() - 1, s.len()) != "\"" {
        return fail;
    }
    let out = "";
    let i = 1;
    while i < s.len() - 1 {
        let c = Text::slice(s, i, i + 1);
        if c == "\\" {
            if i + 1 >= s.len() - 1 {
                return fail;
            }
            let e = Text::slice(s, i + 1, i + 2);
            if e == "n" {
                out = out + "\n";
            } else {
                if e == "t" {
                    out = out + "\t";
                } else {
                    if e == "r" {
                        out = out + "\r";
                    } else {
                        if e == "\"" || e == "\\" {
                            out = out + e;
                        } else {
                            return fail;
                        }
                    }
                }
            }
            i = i + 2;
        } else {
            out = out + c;
            i = i + 1;
        }
    }
    return UnquoteResult { text: out, ok: true };
}
