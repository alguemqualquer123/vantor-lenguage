// Lexicon VS Code providers — vscode-only, NO node imports (no child_process/fs).
// Shared by the desktop entry (`extension.ts`) and the web entry (`extension.web.ts`).
//
// Contents:
//   - tables: KEYWORDS / TYPES / HOVER_DOCS / DECORATOR_DOCS / BUILTIN_FNS
//   - import helpers: hasImport / importEditFor
//   - existing provider factories: hover, completion, definition, document
//     symbols (upgraded), codelens, pipe/arrow decorations helper
//   - new provider factories: signature help, inlay hints, folding ranges,
//     document colors, references, rename, semantic tokens, quick fixes,
//     test discovery, lex task provider, formatting provider
//
// Every factory returns a plain provider object (methods only). Registration
// against `vscode.languages` / `vscode.tasks` happens in the entries so the
// providers stay directly unit-testable (call the methods with a fake doc).

import * as vscode from 'vscode';
import { builtinModules, findModule } from './modules';

export const LANG_ID = 'lexicon';

// ---------------------------------------------------------------------------
// tables (moved verbatim from extension.ts — keep in sync with the toolchain)
// ---------------------------------------------------------------------------

export const KEYWORDS = [
    'fn', 'let', 'var', 'mut', 'const', 'static', 'if', 'else', 'match', 'switch',
    'case', 'default', 'for', 'while', 'loop', 'do', 'in', 'break', 'continue',
    'return', 'defer', 'struct', 'enum', 'class', 'trait', 'interface', 'type',
    'effect', 'where', 'as', 'is', 'new', 'self', 'super', 'import', 'module',
    'pub', 'private', 'protected', 'async', 'await', 'task', 'channel', 'actor',
    'spawn', 'select', 'macro', 'unsafe', 'extern', 'native', 'inline', 'try',
    'catch', 'throw', 'finally', 'panic', 'recover', 'extends', 'implements',
    'with', 'derive', 'yield', 'ref', 'lambda', 'dynamic', 'function',
];

export const TYPES = [
    'bool', 'int', 'uint', 'byte', 'rune', 'char',
    'i8', 'i16', 'i32', 'i64', 'i128',
    'u8', 'u16', 'u32', 'u64', 'u128',
    'f32', 'f64', 'float32', 'float64', 'decimal',
    'string', 'String', 'void',
    'Option', 'Result', 'List', 'Map', 'Set', 'Vec', 'Array',
];

export const HOVER_DOCS: Record<string, { title: string; desc: string; from?: string }> = {
    import: { title: 'Keyword: import', desc: 'Brings modules into scope. Paths use `::`: `import core::net::Http;`, alias with `as`.' },
    module: { title: 'Keyword: module', desc: 'Declares the module path of this file: `module app::routes;`.' },
    pub: { title: 'Modifier: pub', desc: 'Exports the symbol to other modules. Without it the item is file-private.' },
    fn: { title: 'Keyword: fn', desc: 'Declares a function. `pub fn name(params) -> Type { }`. Async with `async fn`.' },
    let: { title: 'Keyword: let', desc: 'Immutable binding. `let x = 1;` or typed `let x: int = 1;`.' },
    mut: { title: 'Modifier: mut', desc: '`let mut x = ...` allows rebinding. Loans follow borrow rules (E0305/E0382).' },
    const: { title: 'Keyword: const', desc: 'Compile-time constant evaluated by `eval_const`.' },
    if: { title: 'Keyword: if', desc: 'Conditional. Also usable as an expression.' },
    else: { title: 'Keyword: else', desc: 'Alternative branch of `if`.' },
    match: { title: 'Keyword: match', desc: 'Pattern match with optional guards: `n if n > 0 => ...`. Bool/enum matches are exhaustiveness-checked (E0204).' },
    switch: { title: 'Keyword: switch', desc: '`switch x { case 1: ... default: ... }`. Cases accept comma lists.' },
    case: { title: 'Keyword: case', desc: 'A `switch` case arm.' },
    default: { title: 'Keyword: default', desc: 'Fallback arm of `switch`.' },
    for: { title: 'Keyword: for', desc: '`for item in iterable { }`. Ranges: `for i in 0..10`.' },
    while: { title: 'Keyword: while', desc: 'Condition loop. `do { } while cond;` runs the body at least once.' },
    loop: { title: 'Keyword: loop', desc: 'Infinite loop; exit with `break`, skip with `continue` (labels allowed).' },
    return: { title: 'Keyword: return', desc: 'Returns a value. Narrowing conversions need `as` (E0306).' },
    defer: { title: 'Keyword: defer', desc: 'Runs at scope unwind, including during `panic`.' },
    struct: { title: 'Keyword: struct', desc: 'Value aggregate. Duplicate fields are rejected (E0304).' },
    enum: { title: 'Keyword: enum', desc: 'Sum type; matched arms are exhaustiveness-checked (E0204).' },
    class: { title: 'Keyword: class', desc: '`class Dog extends Animal implements Loud { }` — `implements` is conformance-checked (E0304).' },
    trait: { title: 'Keyword: trait', desc: 'Behavior contract with method signatures.' },
    interface: { title: 'Keyword: interface', desc: 'Behavior contract (alias family of `trait`).' },
    type: { title: 'Keyword: type', desc: 'Type alias preserving identity: `type UserId = int;`.' },
    async: { title: 'Keyword: async', desc: 'Async function; `.await` completion with `await`.' },
    await: { title: 'Keyword: await', desc: 'Awaits an async value.' },
    spawn: { title: 'Keyword: spawn', desc: 'Spawns a lightweight task on the scheduler.' },
    channel: { title: 'Keyword: channel', desc: 'Channel type; see `Channel::new` with close semantics.' },
    select: { title: 'Keyword: select', desc: 'Multiplexes channel receives (`select_poll` runtime).' },
    unsafe: { title: 'Keyword: unsafe', desc: '`unsafe { }` confines raw operations; diagnostics stay active inside.' },
    extern: { title: 'Keyword: extern', desc: 'Foreign declarations, e.g. `extern "C" { fn f() -> void; }`.' },
    panic: { title: 'Function: panic', desc: 'Fatal error of type `Never`. Unwinds through `defer`; catch at boundaries with recover semantics.' },
    recover: { title: 'Function: recover', desc: 'Converts a panic back into a value at a boundary.' },
    print: { title: 'Function: print', desc: 'Prints to stdout. Concatenate with `+`: `print("env=" + v);`.' },
    println: { title: 'Function: println', desc: 'Prints a line to stdout.' },
    self: { title: 'Keyword: self', desc: 'Current instance.' },
    extends: { title: 'Keyword: extends', desc: 'Single inheritance for classes.' },
    implements: { title: 'Keyword: implements', desc: 'Interface conformance, statically validated.' },
    Some: { title: 'Variant: Some', desc: 'Present value of `Option<T>`. Field access on bare `Option` is rejected (E0305) — unwrap first.' },
    None: { title: 'Variant: None', desc: 'Absent value of `Option<T>`.' },
    Ok: { title: 'Variant: Ok', desc: 'Success of `Result<T, E>`.' },
    Err: { title: 'Variant: Err', desc: 'Failure of `Result<T, E>`.' },
    true: { title: 'Literal: true', desc: 'Boolean true.' },
    false: { title: 'Literal: false', desc: 'Boolean false.' },
    null: { title: 'Literal: null', desc: 'Absence of value.' },
    Http: { title: 'Module: Http', desc: 'Real HTTP server + client (`serve`, `get`, `post`).', from: 'core::net' },
    Json: { title: 'Module: Json', desc: 'JSON `parse` / `stringify` (serde_json).', from: 'core::json' },
    Env: { title: 'Module: Env', desc: '`Env::get` reads the REAL process env. Assign first: `let v = Env::get("K");`.', from: 'core::env' },
    Db: { title: 'Module: Db', desc: '`connect` / `query` / `execute` over lexicon-db (SQLite via sqlx).', from: 'core::db' },
    List: { title: 'Type: List', desc: 'Growable sequence.', from: 'core::collections' },
    Map: { title: 'Type: Map', desc: 'Associative container.', from: 'core::collections' },
};

export const DECORATOR_DOCS: Record<string, string> = {
    '@Get': 'HTTP GET route. The next `pub fn` becomes the handler; served for real by `Http::serve`.',
    '@Post': 'HTTP POST route handler.',
    '@Put': 'HTTP PUT route handler.',
    '@Delete': 'HTTP DELETE route handler.',
    '@Test': 'Marks a test picked up by `lex test`.',
};

export const BUILTIN_FNS: Array<{ name: string; detail: string; doc: string }> = [
    { name: 'print', detail: 'fn print(message: String)', doc: 'Prints to stdout. Concatenate with `+`.' },
    { name: 'println', detail: 'fn println(message: String)', doc: 'Prints a line to stdout.' },
    { name: 'panic', detail: 'fn panic(message: String) -> Never', doc: 'Fatal error. Unwinds through `defer`.' },
    { name: 'recover', detail: 'fn recover() -> any', doc: 'Converts a panic back into a value at a boundary.' },
    { name: 'assert', detail: 'fn assert(cond: bool)', doc: 'Assertion picked up by `lex test` failures.' },
    { name: 'inspect', detail: 'fn inspect(value: any) -> String', doc: 'Debug rendering of any value.' },
    { name: 'spawn', detail: 'spawn task()', doc: 'Spawns a lightweight task: `spawn worker();`.' },
];

// ---------------------------------------------------------------------------
// shared helpers
// ---------------------------------------------------------------------------

export function escapeRegExp(s: string): string {
    return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

export function hasImport(text: string, fullPath: string): boolean {
    return new RegExp(`^\\s*import\\s+${escapeRegExp(fullPath)}\\s*;`, 'm').test(text);
}

export function importEditFor(document: any, fullPath: string): any[] {
    const lines: string[] = document.getText().split('\n');
    let insertLine = 0;
    for (let i = 0; i < lines.length; i++) {
        if (/^\s*import\s+.*;/.test(lines[i])) {
            insertLine = i + 1;
        }
    }
    return [vscode.TextEdit.insert(new vscode.Position(insertLine, 0), `import ${fullPath};\n`)];
}

/** Infer a type name for a *literal* expression only. Returns undefined for anything else (never wrong). */
export function inferLiteralType(expr: string): string | undefined {
    const t = expr.trim().replace(/;$/, '').trim();
    if (/^(true|false)$/.test(t)) {
        return 'bool';
    }
    if (/^[0-9][0-9_]*$/.test(t)) {
        return 'int';
    }
    if (/^[0-9][0-9_]*\.[0-9][0-9_]*$/.test(t)) {
        return 'f64';
    }
    if (/^"(?:[^"\\\n]|\\.)*"$/.test(t) || /^'(?:[^'\\\n]|\\.)*'$/.test(t)) {
        return 'String';
    }
    return undefined;
}

/** Offsets of whole-word occurrences of `word` in CODE (comments and
 * string/char literals are masked first, so `// fn ghost` or `"let x"`
 * never match — offsets are stable because masking preserves length). */
export function findWholeWordOffsets(text: string, word: string): Array<{ start: number; end: number }> {
    const out: Array<{ start: number; end: number }> = [];
    if (!word) {
        return out;
    }
    const re = new RegExp(`\\b${escapeRegExp(word)}\\b`, 'g');
    const masked = maskNonCode(text);
    let m: RegExpExecArray | null;
    while ((m = re.exec(masked))) {
        out.push({ start: m.index, end: m.index + m[0].length });
    }
    return out;
}

// ---------------------------------------------------------------------------
// comment/string masking + pre-run syntax precheck (clear errors w/ line:col)
// ---------------------------------------------------------------------------

/**
 * Mask comments (`//`, `/* *\/`), string literals and char literals with
 * spaces, preserving `\n` and total length. Searching the masked text
 * finds CODE only; every offset still maps 1:1 onto the original
 * document. Mirrors `lexicon-lexer/src/comments.rs` (`strip_comments`).
 */
export function maskNonCode(src: string): string {
    let out = '';
    let i = 0;
    const n = src.length;
    while (i < n) {
        const c = src[i];
        if (c === '"') {
            out += '"';
            i++;
            let closed = false;
            while (i < n) {
                const d = src[i];
                if (d === '\\' && i + 1 < n) {
                    out += '  ';
                    i += 2;
                    continue;
                }
                if (d === '"') {
                    out += '"';
                    i++;
                    closed = true;
                    break;
                }
                if (d === '\n') {
                    out += '\n';
                    i++;
                    break; // unterminated: resume normal scanning next line
                }
                out += ' ';
                i++;
            }
            void closed;
        } else if (c === "'") {
            out += "'";
            i++;
            while (i < n) {
                const d = src[i];
                if (d === '\\' && i + 1 < n) {
                    out += '  ';
                    i += 2;
                    continue;
                }
                if (d === "'" || d === '\n') {
                    out += (d === '\n' ? '\n' : "'");
                    i++;
                    break;
                }
                out += ' ';
                i++;
            }
        } else if (c === '/' && src[i + 1] === '/') {
            while (i < n && src[i] !== '\n') {
                out += ' ';
                i++;
            }
        } else if (c === '/' && src[i + 1] === '*') {
            out += '  ';
            i += 2;
            while (i < n) {
                if (src[i] === '\n') {
                    out += '\n';
                    i++;
                } else if (src[i] === '*' && src[i + 1] === '/') {
                    out += '  ';
                    i += 2;
                    break;
                } else {
                    out += ' ';
                    i++;
                }
            }
        } else {
            out += c;
            i++;
        }
    }
    return out;
}

export interface PrecheckFinding {
    /** 0-based line */
    line: number;
    /** 0-based column */
    col: number;
    /** stable machine code, e.g. E0101 */
    code: string;
    message: string;
    severity: 'error' | 'warning';
}

function offsetToLineCol(text: string, offset: number): { line: number; col: number } {
    let line = 0;
    let col = 0;
    for (let i = 0; i < offset && i < text.length; i++) {
        if (text[i] === '\n') {
            line++;
            col = 0;
        } else {
            col++;
        }
    }
    return { line, col };
}

/**
 * Client-side syntax precheck: clear errors with exact line:col BEFORE
 * anything runs. Catches, in order:
 * - E0101 unterminated string / char literal / block comment (with the
 *   opening line:col),
 * - E0201 unbalanced `()[]{}` (opener line:col on unclosed; closer
 *   line:col on mismatch) and dotted imports (`import a.b;` → use `::`),
 * - Warnings: `Http::serve` with a non-literal address (port detection
 *   fails), dangling `@Get/@Post/...` with no following `fn`, and JS-style
 *   `for (let x in xs)` (canonical is `for x in xs`, W0003 with quick-fix).
 * Never wrong: every rule is syntactic and position-exact.
 */
export function precheckLexicon(text: string): PrecheckFinding[] {
    const out: PrecheckFinding[] = [];
    const masked = maskNonCode(text);

    // 1. unterminated literals/comments (scan raw text, track state).
    {
        let i = 0;
        const n = text.length;
        let strStart = -1;
        let charStart = -1;
        let blockStart = -1;
        while (i < n) {
            const c = text[i];
            if (strStart >= 0) {
                if (c === '\\') {
                    i += 2;
                    continue;
                }
                if (c === '"') {
                    strStart = -1;
                } else if (c === '\n') {
                    const p = offsetToLineCol(text, strStart);
                    out.push({ line: p.line, col: p.col, code: 'E0101', message: 'Unterminated string literal (missing closing `"`).', severity: 'error' });
                    strStart = -1;
                }
                i++;
                continue;
            }
            if (charStart >= 0) {
                if (c === '\\') {
                    i += 2;
                    continue;
                }
                if (c === "'" || c === '\n') {
                    if (c === '\n') {
                        const p = offsetToLineCol(text, charStart);
                        out.push({ line: p.line, col: p.col, code: 'E0101', message: "Unterminated char literal (missing closing ').", severity: 'error' });
                    }
                    charStart = -1;
                }
                i++;
                continue;
            }
            if (blockStart >= 0) {
                if (c === '*' && text[i + 1] === '/') {
                    blockStart = -1;
                    i += 2;
                    continue;
                }
                i++;
                continue;
            }
            if (c === '"') {
                strStart = i;
            } else if (c === "'") {
                charStart = i;
            } else if (c === '/' && text[i + 1] === '/') {
                while (i < n && text[i] !== '\n') {
                    i++;
                }
                continue;
            } else if (c === '/' && text[i + 1] === '*') {
                blockStart = i;
                i += 2;
                continue;
            }
            i++;
        }
        if (strStart >= 0) {
            const p = offsetToLineCol(text, strStart);
            out.push({ line: p.line, col: p.col, code: 'E0101', message: 'Unterminated string literal (missing closing `"`).', severity: 'error' });
        }
        if (charStart >= 0) {
            const p = offsetToLineCol(text, charStart);
            out.push({ line: p.line, col: p.col, code: 'E0101', message: 'Unterminated char literal.', severity: 'error' });
        }
        if (blockStart >= 0) {
            const p = offsetToLineCol(text, blockStart);
            out.push({ line: p.line, col: p.col, code: 'E0101', message: 'Unterminated block comment (missing `*/`).', severity: 'error' });
        }
    }

    // 2. bracket balance on masked code (comments/strings invisible).
    {
        const pairs: Record<string, string> = { '(': ')', '[': ']', '{': '}' };
        const closers: Record<string, string> = { ')': '(', ']': '[', '}': '{' };
        const stack: Array<{ ch: string; offset: number }> = [];
        for (let i = 0; i < masked.length; i++) {
            const c = masked[i];
            if (pairs[c]) {
                stack.push({ ch: c, offset: i });
            } else if (closers[c]) {
                const top = stack.pop();
                if (!top) {
                    const p = offsetToLineCol(text, i);
                    out.push({ line: p.line, col: p.col, code: 'E0201', message: `Unmatched closing \`${c}\` (no opener).`, severity: 'error' });
                } else if (top.ch !== closers[c]) {
                    const p = offsetToLineCol(text, i);
                    const q = offsetToLineCol(text, top.offset);
                    out.push({ line: p.line, col: p.col, code: 'E0201', message: `Mismatched \`${top.ch}\` opened at line ${q.line + 1}, closed with \`${c}\`.`, severity: 'error' });
                }
            }
        }
        for (const top of stack) {
            const p = offsetToLineCol(text, top.offset);
            out.push({ line: p.line, col: p.col, code: 'E0201', message: `Unclosed \`${top.ch}\` opened here.`, severity: 'error' });
        }
    }

    // 3. dotted imports + serve/decorator warnings (line-based, masked).
    {
        const lines = masked.split('\n');
        lines.forEach((ln, idx) => {
            const imp = /^\s*import\s+([^;]*);/.exec(ln);
            if (imp && imp[1].includes('.')) {
                const dot = ln.indexOf('.', imp[0].indexOf(imp[1]));
                out.push({ line: idx, col: Math.max(0, dot), code: 'E0201', message: 'Import paths use `::`, not `.` — write `import a::b::C;`.', severity: 'error' });
            }
            if (/Http::serve\s*\(/.test(ln) && !/Http::serve\s*\(\s*"/.test(ln)) {
                out.push({ line: idx, col: ln.indexOf('Http::serve'), code: 'W0001', message: '`Http::serve` needs a literal address (e.g. "0.0.0.0:3000") for port detection.', severity: 'warning' });
            }
            const dec = /^\s*@(Get|Post|Put|Delete|Test)\b/.exec(ln);
            if (dec) {
                let j = idx + 1;
                while (j < lines.length && lines[j].trim() === '') {
                    j++;
                }
                const next = j < lines.length ? lines[j].trim() : '';
                if (!/^(pub\s+)?(async\s+)?fn\b/.test(next)) {
                    out.push({ line: idx, col: ln.indexOf('@'), code: 'W0002', message: `@${dec[1]} has no following \`fn\` — the route will not register.`, severity: 'warning' });
                }
            }
            // JS-like for-in: the parser wants `for x in xs`, no `let`/parens.
            const jsFor = /\bfor\s*\(\s*(?:let\s+|var\s+|const\s+|mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s+in\b/.exec(ln);
            if (jsFor) {
                out.push({ line: idx, col: Math.max(0, ln.indexOf('for')), code: 'W0003', message: `JS-style for-in: write \`for ${jsFor[1]} in ...\` (no \`let\`, no parens). Quick-fix available.`, severity: 'warning' });
            }
        });
    }

    out.sort((a, b) => a.line - b.line || a.col - b.col);
    return out;
}

/** Split a comma-separated argument prefix at top level (ignores nested parens/strings). */
export function countTopLevelCommas(argsSoFar: string): number {
    let depth = 0;
    let inStr: string | undefined;
    let count = 0;
    for (let i = 0; i < argsSoFar.length; i++) {
        const c = argsSoFar[i];
        if (inStr) {
            if (c === '\\') {
                i++;
            } else if (c === inStr) {
                inStr = undefined;
            }
            continue;
        }
        if (c === '"' || c === '\'') {
            inStr = c;
        } else if (c === '(' || c === '[' || c === '{') {
            depth++;
        } else if (c === ')' || c === ']' || c === '}') {
            depth = Math.max(0, depth - 1);
        } else if (c === ',' && depth === 0) {
            count++;
        }
    }
    return count;
}

/** Extract parameter labels from a `fn name(a: T, b: U)` style detail string. */
export function paramsFromDetail(detail: string): string[] {
    const m = /\(([^)]*)\)/.exec(detail);
    if (!m) {
        return [];
    }
    const inner = m[1].trim();
    if (!inner) {
        return [];
    }
    return inner.split(',').map(s => s.trim()).filter(s => s.length > 0);
}

// ---------------------------------------------------------------------------
// scope/type analysis (pure, no vscode) — hover / completion / inlay /
// definition share one conservative model of the file. Understands:
// struct/class fields, let/var/const with `: Type` and/or `= init`,
// fn params (`name: Type`), and for-in loop vars in BOTH the canonical
// form (`for owner in Owners { }`) and the JS-like form some users type
// (`for (let owner in Owners)`). Inference returns undefined when unsure
// (never wrong); callers mark heuristic results as "(inferred)".
// ---------------------------------------------------------------------------

export interface StructField { name: string; type: string; line: number; }
export interface TypeDecl { kind: string; name: string; fields: StructField[]; line: number; }
export interface VarBinding { name: string; kind: string; type?: string; init?: string; line: number; }

function splitTopLevel(s: string): string[] {
    const out: string[] = [];
    let depth = 0;
    let inStr: string | undefined;
    let cur = '';
    for (let i = 0; i < s.length; i++) {
        const c = s[i];
        if (inStr) {
            cur += c;
            if (c === '\\' && i + 1 < s.length) {
                cur += s[i + 1];
                i++;
            } else if (c === inStr) {
                inStr = undefined;
            }
            continue;
        }
        if (c === '"' || c === '\'') {
            inStr = c;
            cur += c;
        } else if (c === '(' || c === '[' || c === '{' || c === '<') {
            depth++;
            cur += c;
        } else if (c === ')' || c === ']' || c === '}' || c === '>') {
            depth = Math.max(0, depth - 1);
            cur += c;
        } else if (c === ',' && depth === 0) {
            out.push(cur);
            cur = '';
        } else {
            cur += c;
        }
    }
    out.push(cur);
    return out;
}

function lineOf(text: string, offset: number): number {
    return offsetToLineCol(text, offset).line;
}

/** All `struct Name { f: T, ... }` / `class Name { ... }` declarations. */
export function parseTypeDecls(text: string): TypeDecl[] {
    const out: TypeDecl[] = [];
    const masked = maskNonCode(text);
    const headRe = /\b(struct|class)\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>]*>)?[^{;]*\{/g;
    let m: RegExpExecArray | null;
    while ((m = headRe.exec(masked))) {
        const kind = m[1];
        const name = m[2];
        const declLine = lineOf(text, m.index);
        // brace-balanced body scan (methods contain braces: skip them).
        let depth = 0;
        let bodyStart = -1;
        let bodyEnd = -1;
        for (let i = m.index + m[0].length - 1; i < masked.length; i++) {
            const c = masked[i];
            if (c === '{') {
                if (bodyStart < 0) {
                    bodyStart = i + 1;
                }
                depth++;
            } else if (c === '}') {
                depth--;
                if (depth <= 0) {
                    bodyEnd = i;
                    break;
                }
            }
        }
        if (bodyStart < 0 || bodyEnd < 0) {
            continue;
        }
        const body = text.slice(bodyStart, bodyEnd);
        const fields: StructField[] = [];
        for (const rawLn of body.split('\n')) {
            const ln = rawLn.trim();
            if (!ln || /^(pub\s+)?(fn|constructor|get|set)\b/.test(ln) || ln.startsWith('@') || ln.startsWith('//')) {
                continue;
            }
            const fm = /^(?:pub\s+|private\s+|protected\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([A-Za-z_][A-Za-z0-9_?]+(?:\s*<[^;{}]*>)?(?:\[\])?)/.exec(ln);
            if (fm) {
                fields.push({ name: fm[1], type: fm[2].trim(), line: declLine });
            }
        }
        out.push({ kind, name, fields, line: declLine });
    }
    return out;
}

/** let/var/const bindings + fn params + for-in loop vars (canonical and JS-like). */
export function parseBindings(text: string): VarBinding[] {
    const out: VarBinding[] = [];
    const masked = maskNonCode(text);
    let m: RegExpExecArray | null;
    const letRe = /\b(let|var|const)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*(?::\s*([A-Za-z_][A-Za-z0-9_?]+(?:\s*<[^;={]*>)?(?:\[\])?))?\s*(?:=\s*([^;]+))?;/g;
    while ((m = letRe.exec(masked))) {
        const rawInit = m[4];
        out.push({
            name: m[2],
            kind: m[1],
            type: m[3] ? m[3].trim() : undefined,
            init: rawInit ? rawInit.trim().replace(/\s+/g, ' ') : undefined,
            line: lineOf(text, m.index),
        });
    }
    const fnRe = /\bfn\s+[A-Za-z_][A-Za-z0-9_]*\s*\(([^)]*)\)/g;
    while ((m = fnRe.exec(masked))) {
        const inner = m[1].trim();
        if (!inner) {
            continue;
        }
        const fnLine = lineOf(text, m.index);
        for (const part of splitTopLevel(inner)) {
            const pm = /^\s*(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(.+?)\s*$/.exec(part);
            if (pm && !/^(mut|self|super)$/.test(pm[1])) {
                out.push({ name: pm[1], kind: 'param', type: pm[2].trim(), init: undefined, line: fnLine });
            }
        }
    }
    // Canonical `for owner in Owners {` plus tolerated `for (let owner in Owners)`.
    // NOTE: the iterable group must be GREEDY (lazy + optional tail would
    // match a single char, e.g. `in:O`).
    const forRe = /\bfor\s*\(?\s*(?:let\s+|var\s+|const\s+|mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s+in\s+([^\)\{\n;]+)\)?\s*\{?/g;
    while ((m = forRe.exec(masked))) {
        const name = m[1];
        if (name === 'in') {
            continue;
        }
        let iter = m[2].trim().replace(/\s+$/, '');
        iter = iter.replace(/\)\s*$/, '').replace(/\{\s*$/, '').trim();
        if (!iter) {
            continue;
        }
        out.push({ name, kind: 'loop', type: undefined, init: `in:${iter}`, line: lineOf(text, m.index) });
    }
    out.sort((a, b) => a.line - b.line);
    return out;
}

/** Conservative type inference for an initializer/right-hand expression. */
export function inferExprType(expr: string, bindings: VarBinding[], decls: TypeDecl[], depth = 0): string | undefined {
    if (depth > 6) {
        return undefined;
    }
    let t = expr.trim().replace(/;$/, '').trim();
    if (!t) {
        return undefined;
    }
    const lit = inferLiteralType(t);
    if (lit) {
        return lit;
    }
    let m: RegExpExecArray | null;
    // `x as Type` casts.
    m = /^(.+?)\s+as\s+([A-Za-z_][A-Za-z0-9_]*(?:\s*<[^>]*>)?(?:\[\])?)$/.exec(t);
    if (m) {
        return m[2].trim();
    }
    // `new Owner(...)` constructor calls.
    m = /^new\s+([A-Za-z_][A-Za-z0-9_]*(?:\s*<[^>]*>)?)/.exec(t);
    if (m) {
        return m[1].trim();
    }
    // `Owner { ... }` / `Box<i32> { ... }` struct literals.
    m = /^([A-Za-z_][A-Za-z0-9_]*)(?:\s*<([^>]*)>)?\s*\{/.exec(t);
    if (m && /^[A-Z]/.test(m[1])) {
        return m[2] ? `${m[1]}<${m[2].trim()}>` : m[1];
    }
    // Range `0..10` / `0..=10` iterates ints.
    if (/^[0-9][0-9_]*\s*\.\.=?.*$/.test(t) || /^\([^)]*\.\.[^)]*\)$/.test(t)) {
        return 'int';
    }
    // List literal `[a, b]` — element from first inferable item.
    if (/^\[/.test(t) && /\]$/.test(t)) {
        const inner = t.slice(1, -1).trim();
        if (!inner) {
            return 'List';
        }
        for (const part of splitTopLevel(inner)) {
            const et = inferExprType(part, bindings, decls, depth + 1);
            if (et) {
                return `List<${et}>`;
            }
        }
        return 'List';
    }
    // Known method results.
    m = /^.+?\.to_string\s*\(\s*\)$/.exec(t);
    if (m) {
        return 'String';
    }
    m = /^.+?\.(len|length|count|size)\s*\(\s*\)$/.exec(t);
    if (m) {
        return 'int';
    }
    // Bare variable reference — follow the binding (explicit type wins).
    m = /^([A-Za-z_][A-Za-z0-9_]*)$/.exec(t);
    if (m) {
        const name = m[1];
        for (let i = bindings.length - 1; i >= 0; i--) {
            const b = bindings[i];
            if (b.name !== name) {
                continue;
            }
            if (b.type) {
                return b.type;
            }
            if (b.init && !b.init.startsWith('in:')) {
                return inferExprType(b.init, bindings, decls, depth + 1);
            }
        }
        return undefined;
    }
    return undefined;
}

/** Element type when iterating `iterType` / `iterExpr` with for-in. */
export function elementTypeOf(iterType: string | undefined, iterExpr: string, bindings: VarBinding[], decls: TypeDecl[]): { type?: string; heuristic: boolean } {
    const expr = iterExpr.trim();
    if (iterType) {
        const lt = /^List\s*<\s*([^<>]+?)\s*>$/.exec(iterType);
        if (lt) {
            return { type: lt[1].trim(), heuristic: false };
        }
        if (/^List$/.test(iterType)) {
            return { type: undefined, heuristic: false };
        }
        const map = /^Map\s*<\s*[^,<>]+?\s*,\s*([^<>]+?)\s*>$/.exec(iterType);
        if (map) {
            return { type: map[1].trim(), heuristic: true };
        }
    }
    // List literal directly in the loop head.
    if (/^\[/.test(expr)) {
        const t = inferExprType(expr, bindings, decls);
        if (t) {
            const lt = /^List\s*<\s*([^<>]+?)\s*>$/.exec(t);
            if (lt) {
                return { type: lt[1].trim(), heuristic: false };
            }
        }
        return { type: undefined, heuristic: false };
    }
    // Range iterates ints.
    if (/^[0-9][0-9_]*\s*\.\.=?.*$/.test(expr)) {
        return { type: 'int', heuristic: false };
    }
    // Plural heuristic: `Owners` -> struct `Owner` (common convention,
    // also the exact case from the report). Marked heuristic by the caller.
    const cap = /^([A-Za-z_][A-Za-z0-9_]*)$/.exec(expr);
    if (cap) {
        const plural = cap[1];
        const singular = plural.endsWith('s') && plural.length > 1 ? plural.slice(0, -1) : undefined;
        const candidates = singular ? [singular, singular[0].toUpperCase() + singular.slice(1)] : [];
        for (const c of candidates) {
            if (decls.some(d => d.name === c)) {
                return { type: c, heuristic: true };
            }
        }
    }
    return { type: undefined, heuristic: false };
}

export interface ResolvedVar {
    binding: VarBinding;
    type?: string;
    heuristic: boolean;
    decl?: TypeDecl;
    fields: StructField[];
}

/** Resolve a variable name to its binding + type at (or before) a line. */
export function resolveVariable(text: string, name: string, beforeLine?: number): ResolvedVar | undefined {
    const decls = parseTypeDecls(text);
    const bindings = parseBindings(text);
    const usable = typeof beforeLine === 'number'
        ? bindings.filter(b => b.line <= beforeLine)
        : bindings;
    let binding: VarBinding | undefined;
    for (let i = usable.length - 1; i >= 0; i--) {
        if (usable[i].name === name) {
            binding = usable[i];
            break;
        }
    }
    if (!binding) {
        return undefined;
    }
    let type = binding.type;
    let heuristic = false;
    if (!type) {
        if (binding.kind === 'loop' && binding.init && binding.init.startsWith('in:')) {
            const iterExpr = binding.init.slice(3).trim();
            const iterType = inferExprType(iterExpr, bindings, decls);
            const el = elementTypeOf(iterType, iterExpr, bindings, decls);
            type = el.type || iterType;
            heuristic = el.heuristic || (!el.type && !!iterType);
            if (type && !el.type && iterType) {
                heuristic = true;
            }
        } else if (binding.init) {
            type = inferExprType(binding.init, bindings, decls);
        }
    }
    const base = type ? type.replace(/\s*<.*$/, '').replace(/\[\]$/, '').trim() : undefined;
    const decl = base ? decls.find(d => d.name === base) : undefined;
    return { binding, type, heuristic, decl, fields: decl ? decl.fields : [] };
}

/** If the cursor sits on `receiver.field`, return both sides + which side. */
export function findDotAccess(line: string, character: number): { receiver: string; field: string; onField: boolean } | undefined {
    const re = /([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)?/g;
    let m: RegExpExecArray | null;
    while ((m = re.exec(line))) {
        const recvStart = m.index;
        const recvEnd = recvStart + m[1].length;
        const field = m[2] || '';
        const fieldStart = line.indexOf(field, recvEnd);
        const fieldEnd = field ? fieldStart + field.length : recvEnd;
        const dotPos = line.indexOf('.', recvEnd);
        if (character >= recvStart && character <= recvEnd) {
            return { receiver: m[1], field, onField: false };
        }
        if (field && character >= fieldStart && character <= fieldEnd) {
            return { receiver: m[1], field, onField: true };
        }
        if (character > recvEnd && character <= dotPos + 1 && !field) {
            return { receiver: m[1], field: '', onField: true };
        }
    }
    return undefined;
}

/** `///` / `//` comment lines directly above `line` (0-based). */
export function docCommentsFor(lines: string[], line: number): string[] {
    const comments: string[] = [];
    let ln = line - 1;
    while (ln >= 0) {
        const t = lines[ln].trim();
        if (t.startsWith('///')) {
            comments.unshift(t.substring(3).trim());
        } else if (t.startsWith('//')) {
            comments.unshift(t.substring(2).trim());
        } else {
            break;
        }
        ln--;
    }
    return comments;
}

// ---------------------------------------------------------------------------
// hover
// ---------------------------------------------------------------------------

export function createHoverProvider(): any {
    return {
        provideHover(document: any, position: any): any {
            const wordRange = document.getWordRangeAtPosition(position);
            const word = wordRange ? document.getText(wordRange) : '';
            const known = HOVER_DOCS[word];
            if (known) {
                const md = new vscode.MarkdownString();
                md.appendMarkdown(`### ${known.title}\n`);
                if (known.from) {
                    md.appendMarkdown(`*From: \`${known.from}\`*\n\n`);
                }
                md.appendMarkdown(known.desc);
                return new vscode.Hover(md);
            }
            const decRange = document.getWordRangeAtPosition(position, /@[a-zA-Z0-9_]+/);
            if (decRange) {
                const dec = document.getText(decRange);
                if (DECORATOR_DOCS[dec]) {
                    const md = new vscode.MarkdownString();
                    md.appendMarkdown(`### Decorator: ${dec}\n---\n${DECORATOR_DOCS[dec]}`);
                    return new vscode.Hover(md);
                }
            }
            // stdlib member hover: `Module::member`
            const line = document.lineAt(position.line).text;
            const modMatch = /([A-Za-z_][A-Za-z0-9_]*)::([A-Za-z_][A-Za-z0-9_]*)/.exec(line);
            if (modMatch) {
                const mod = findModule(modMatch[1]);
                const member = mod?.members.find(m => m.name === modMatch[2]);
                if (mod && member) {
                    const md = new vscode.MarkdownString();
                    md.appendCodeblock(`${mod.shortName}::${member.name} — ${member.detail}`, 'lexicon');
                    md.appendMarkdown(member.documentation);
                    return new vscode.Hover(md);
                }
            }
            if (!word) {
                return null;
            }
            const fullText: string = document.getText();
            const allLines: string[] = fullText.split('\n');

            // NEW: `receiver.field` hover — struct/class field types.
            // Covers the reported `owner.name` / `owner.email` case.
            try {
                const dot = findDotAccess(line, position.character);
                if (dot && dot.field) {
                    const recv = resolveVariable(fullText, dot.receiver, position.line);
                    if (recv && recv.type && recv.fields.length > 0) {
                        const f = recv.fields.find(fl => fl.name === dot.field);
                        if (f) {
                            const md = new vscode.MarkdownString();
                            md.appendCodeblock(`${recv.type}.${f.name}: ${f.type}`, 'lexicon');
                            md.appendMarkdown(`Field of \`${recv.type}\` (via \`${dot.receiver}\`).`);
                            const sibs = recv.fields.filter(fl => fl.name !== f.name).slice(0, 8);
                            if (sibs.length > 0) {
                                md.appendMarkdown('\n\nSibling fields: ' + sibs.map(s => `\`${s.name}: ${s.type}\``).join(', '));
                            }
                            return new vscode.Hover(md);
                        }
                        // Unknown field: still show the receiver type + known fields.
                        if (dot.onField) {
                            const md = new vscode.MarkdownString();
                            md.appendCodeblock(`${dot.receiver}: ${recv.type}${recv.heuristic ? ' (inferred)' : ''}`, 'lexicon');
                            md.appendMarkdown(`Known fields of \`${recv.type}\`: ` + recv.fields.map(fl => `\`${fl.name}: ${fl.type}\``).join(', '));
                            return new vscode.Hover(md);
                        }
                    }
                }
            } catch {
                // field hover is best-effort; fall through
            }

            // NEW: variable hover — let/var/const, fn params AND for-in loop
            // vars, with inferred types (`let owner`, `for owner in Owners`).
            try {
                const resolved = resolveVariable(fullText, word, position.line);
                if (resolved) {
                    const md = new vscode.MarkdownString();
                    const kindLabel = resolved.binding.kind === 'loop' ? 'loop var' : resolved.binding.kind;
                    const typeLabel = resolved.type
                        ? `: ${resolved.type}${resolved.heuristic ? ' (inferred)' : ''}`
                        : ': unknown';
                    md.appendCodeblock(`${kindLabel} ${word}${typeLabel}`, 'lexicon');
                    const bits: string[] = [];
                    if (resolved.binding.kind === 'loop' && resolved.binding.init) {
                        bits.push(`Iterates \`${resolved.binding.init.slice(3).trim()}\`.`);
                    } else if (resolved.binding.init && resolved.binding.init.length <= 120) {
                        bits.push(`Initialized with \`${resolved.binding.init}\`.`);
                    }
                    if (resolved.decl && resolved.fields.length > 0) {
                        bits.push(`Fields of \`${resolved.decl.name}\`: ` + resolved.fields.map(f => `\`${f.name}: ${f.type}\``).join(', '));
                    } else if (resolved.type && !resolved.decl && resolved.heuristic) {
                        bits.push('Type the binding explicitly (`let ' + word + ': Type`) or declare the struct for full field info.');
                    }
                    const comments = docCommentsFor(allLines, resolved.binding.line);
                    if (comments.length > 0) {
                        bits.push('---\n' + comments.join('\n\n'));
                    }
                    if (bits.length > 0) {
                        md.appendMarkdown(bits.join('\n\n'));
                    }
                    return new vscode.Hover(md);
                }
            } catch {
                // variable hover is best-effort; fall through
            }

            // NEW: type-name hover — `Owner` / `Owners` element shows its fields.
            try {
                const decls = parseTypeDecls(fullText);
                const decl = decls.find(d => d.name === word);
                if (decl) {
                    const md = new vscode.MarkdownString();
                    md.appendCodeblock(`${decl.kind} ${decl.name}`, 'lexicon');
                    if (decl.fields.length > 0) {
                        md.appendMarkdown('Fields: ' + decl.fields.map(f => `\`${f.name}: ${f.type}\``).join(', '));
                    } else {
                        md.appendMarkdown('No typed fields found in the declaration.');
                    }
                    const comments = docCommentsFor(allLines, decl.line);
                    if (comments.length > 0) {
                        md.appendMarkdown('\n\n---\n' + comments.join('\n\n'));
                    }
                    return new vscode.Hover(md);
                }
            } catch {
                // ignore
            }

            // local definition search with doc comments (comments/strings
            // masked: `// fn ghost` or `"fn ghost"` never match; offsets
            // are stable because masking preserves length)
            const text = maskNonCode(fullText);
            const patterns: Array<{ re: RegExp; fmt: (m: RegExpExecArray) => string }> = [
                { re: new RegExp(`fn\\s+${escapeRegExp(word)}\\s*\\(([^)]*)\\)\\s*(->\\s*([a-zA-Z0-9_?<>]+))?`, 'g'), fmt: (m) => `**fn** ${word}(${m[1] || ''}) ${m[3] ? '-> ' + m[3] : ''}` },
                { re: new RegExp(`\\b(class|struct|enum|trait|interface)\\s+${escapeRegExp(word)}\\b`, 'g'), fmt: (m) => `**${m[1]}** ${word}` },
                { re: new RegExp(`(let|var|const)\\s+(mut\\s+)?${escapeRegExp(word)}\\s*:\\s*([a-zA-Z0-9_?<>\\[\\]]+)`, 'g'), fmt: (m) => `**${m[1]}** ${word}: ${m[3]}` },
                { re: new RegExp(`(let|var|const)\\s+(mut\\s+)?${escapeRegExp(word)}\\s*=`, 'g'), fmt: (m) => `**${m[1]}** ${word} (type inferred — hover the use for details)` },
                { re: new RegExp(`for\\s*\\(?\\s*(?:let\\s+|var\\s+|const\\s+|mut\\s+)?${escapeRegExp(word)}\\s+in\\s+([^\\n;\\{]+)`, 'g'), fmt: (m) => `**loop var** ${word} in ${(m[1] || '').trim()}` },
            ];
            for (const p of patterns) {
                let m: RegExpExecArray | null;
                while ((m = p.re.exec(text))) {
                    const pos = document.positionAt(m.index);
                    const comments: string[] = [];
                    let ln = pos.line - 1;
                    while (ln >= 0) {
                        const t = document.lineAt(ln).text.trim();
                        if (t.startsWith('///')) {
                            comments.unshift(t.substring(3).trim());
                        } else if (t.startsWith('//')) {
                            comments.unshift(t.substring(2).trim());
                        } else {
                            break;
                        }
                        ln--;
                    }
                    const md = new vscode.MarkdownString();
                    md.appendCodeblock(p.fmt(m), 'lexicon');
                    if (comments.length > 0) {
                        md.appendMarkdown('---\n' + comments.join('\n\n'));
                    }
                    return new vscode.Hover(md);
                }
            }
            return null;
        }
    };
}

// ---------------------------------------------------------------------------
// completion (context-aware + auto-import)
// ---------------------------------------------------------------------------

export function createCompletionProvider(): any {
    return {
        provideCompletionItems(document: any, position: any): any[] {
            const items: any[] = [];
            const line = document.lineAt(position.line).text;
            const prefix = line.substring(0, position.character);
            const text = document.getText();

            // Context 1: inside an import statement.
            const impMatch = /^\s*import\s+([A-Za-z0-9_:]*)$/.exec(prefix);
            if (impMatch) {
                const typed = impMatch[1];
                const wordStart = position.character - typed.length;
                const range = new vscode.Range(new vscode.Position(position.line, wordStart), position);
                for (const mod of builtinModules) {
                    if (mod.fullPath.startsWith(typed) || mod.shortName.startsWith(typed)) {
                        const item = new vscode.CompletionItem(mod.fullPath, vscode.CompletionItemKind.Module);
                        item.range = range;
                        item.detail = `Module ${mod.shortName}`;
                        item.documentation = new vscode.MarkdownString(
                            `**${mod.fullPath}**\n\n${mod.members.length} members: ` +
                            mod.members.map((m: any) => `\`${m.name}\``).join(', ')
                        );
                        item.sortText = '0' + mod.shortName;
                        items.push(item);
                    }
                }
                return items;
            }

            // Context 2: Module::member access.
            const modMatch = /([A-Za-z_][A-Za-z0-9_]*)::([A-Za-z_][A-Za-z0-9_]*)?$/.exec(prefix);
            if (modMatch) {
                const mod = findModule(modMatch[1]);
                if (!mod) {
                    return [];
                }
                const typed = modMatch[2] || '';
                const wordStart = position.character - typed.length;
                const range = new vscode.Range(new vscode.Position(position.line, wordStart), position);
                const needImport = !hasImport(text, mod.fullPath);
                mod.members
                    .filter((m: any) => m.name.startsWith(typed))
                    .forEach((m: any, i: number) => {
                        const item = new vscode.CompletionItem(m.name, vscode.CompletionItemKind.Function);
                        item.range = range;
                        item.detail = `${mod.shortName}::${m.name}: ${m.detail}`;
                        item.documentation = new vscode.MarkdownString(
                            `**\`${mod.shortName}::${m.name}\`** — ${m.detail}\n\n${m.documentation}` +
                            (needImport ? `\n\n*Auto-imports \`import ${mod.fullPath};\` on accept.*` : '')
                        );
                        item.insertText = new vscode.SnippetString(`${m.name}($1)$0`);
                        item.sortText = '00' + String(i).padStart(3, '0');
                        item.filterText = `${mod.shortName}::${m.name}`;
                        if (needImport) {
                            item.additionalTextEdits = importEditFor(document, mod.fullPath);
                        }
                        items.push(item);
                    });
                return items;
            }

            // Context 2b: dot member access (`Http.get` or `ident.`).
            // Module heads reuse the member table; unknown idents get the
            // generic instance-method list marked "(inferred)".
            const dotMatch = /([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)?$/.exec(prefix);
            if (dotMatch) {
                const head = dotMatch[1];
                const typed = dotMatch[2] || '';
                const wordStart = position.character - typed.length;
                const range = new vscode.Range(new vscode.Position(position.line, wordStart), position);
                const mod = findModule(head);
                if (mod) {
                    const needImport = !hasImport(text, mod.fullPath);
                    mod.members
                        .filter((m: any) => m.name.startsWith(typed))
                        .forEach((m: any, i: number) => {
                            const item = new vscode.CompletionItem(m.name, vscode.CompletionItemKind.Function);
                            item.range = range;
                            item.detail = `${mod.shortName}.${m.name}: ${m.detail}`;
                            item.documentation = new vscode.MarkdownString(
                                `**\`${mod.shortName}.${m.name}\`** — ${m.detail}\n\n${m.documentation}` +
                                (needImport ? `\n\n*Auto-imports \`import ${mod.fullPath};\` on accept.*` : '')
                            );
                            item.insertText = new vscode.SnippetString(`${m.name}($1)$0`);
                            item.sortText = '00' + String(i).padStart(3, '0');
                            if (needImport) {
                                item.additionalTextEdits = importEditFor(document, mod.fullPath);
                            }
                            items.push(item);
                        });
                    return items;
                }
                // Known local variable with a struct/class type: complete
                // its REAL fields first (`owner.` -> `name`, `email`).
                try {
                    const resolved = resolveVariable(text, head, position.line);
                    if (resolved && resolved.type && resolved.fields.length > 0) {
                        resolved.fields
                            .filter(f => f.name.startsWith(typed))
                            .forEach((f, i) => {
                                const item = new vscode.CompletionItem(f.name, vscode.CompletionItemKind.Field);
                                item.range = range;
                                item.detail = `${resolved.type}.${f.name}: ${f.type}`;
                                item.documentation = new vscode.MarkdownString(
                                    `Field \`${f.name}\` of \`${resolved.type}\`${resolved.heuristic ? ' (receiver type inferred)' : ''}.`
                                );
                                item.sortText = '00' + String(i).padStart(3, '0');
                                items.push(item);
                            });
                        if (items.length > 0) {
                            return items;
                        }
                    }
                } catch {
                    // fall through to generic methods
                }
                // Unknown receiver: generic instance methods (item 2).
                const INSTANCE_METHODS = [
                    ['push', 'fn push(value) — append (inferred)'],
                    ['pop', 'fn pop() — remove last (inferred)'],
                    ['get', 'fn get(index|key) (inferred)'],
                    ['set', 'fn set(index|key, value) (inferred)'],
                    ['len', 'fn len() -> int (inferred)'],
                    ['isEmpty', 'fn isEmpty() -> bool (inferred)'],
                    ['first', 'fn first() (inferred)'],
                    ['last', 'fn last() (inferred)'],
                    ['contains', 'fn contains(value) -> bool (inferred)'],
                    ['map', 'fn map(fn) (inferred)'],
                    ['filter', 'fn filter(fn) (inferred)'],
                    ['keys', 'fn keys() (inferred)'],
                    ['values', 'fn values() (inferred)'],
                ];
                for (const [name, detail] of INSTANCE_METHODS) {
                    if ((name as string).startsWith(typed)) {
                        const item = new vscode.CompletionItem(name as string, vscode.CompletionItemKind.Method);
                        item.range = range;
                        item.detail = detail as string;
                        item.sortText = '03' + (name as string);
                        items.push(item);
                    }
                }
                return items;
            }

            // Context 3: general code.
            // Locals first (bindings declared at or above the cursor): loop
            // vars, params and let/var/const with inferred types.
            try {
                const decls = parseTypeDecls(text);
                const bindings = parseBindings(text).filter(b => b.line <= position.line);
                const seen = new Set<string>();
                for (let i = bindings.length - 1; i >= 0; i--) {
                    const b = bindings[i];
                    if (seen.has(b.name)) {
                        continue;
                    }
                    seen.add(b.name);
                    let bt: string | undefined = b.type;
                    if (!bt) {
                        if (b.kind === 'loop' && b.init && b.init.startsWith('in:')) {
                            const iterExpr = b.init.slice(3).trim();
                            const iterType = inferExprType(iterExpr, bindings, decls);
                            const el = elementTypeOf(iterType, iterExpr, bindings, decls);
                            bt = el.type || iterType;
                        } else if (b.init) {
                            bt = inferExprType(b.init, bindings, decls);
                        }
                    }
                    const item = new vscode.CompletionItem(b.name, vscode.CompletionItemKind.Variable);
                    item.detail = `${b.kind} ${b.name}${bt ? ': ' + bt : ''}`;
                    item.sortText = '005' + b.name;
                    items.push(item);
                }
                for (const d of decls) {
                    const item = new vscode.CompletionItem(d.name, vscode.CompletionItemKind.Struct);
                    item.detail = `${d.kind} ${d.name}` + (d.fields.length > 0 ? ` (${d.fields.length} fields)` : '');
                    item.sortText = '006' + d.name;
                    items.push(item);
                }
            } catch {
                // locals are best-effort
            }
            for (const k of KEYWORDS) {
                items.push(new vscode.CompletionItem(k, vscode.CompletionItemKind.Keyword));
            }
            for (const t of TYPES) {
                const item = new vscode.CompletionItem(t, vscode.CompletionItemKind.Class);
                item.detail = 'Lexicon builtin type';
                items.push(item);
            }
            for (const f of BUILTIN_FNS) {
                const item = new vscode.CompletionItem(f.name, vscode.CompletionItemKind.Function);
                item.detail = f.detail;
                item.documentation = new vscode.MarkdownString(f.doc);
                item.insertText = new vscode.SnippetString(`${f.name}($1)$0`);
                item.sortText = '01' + f.name;
                items.push(item);
            }
            for (const mod of builtinModules) {
                const item = new vscode.CompletionItem(mod.shortName, vscode.CompletionItemKind.Module);
                item.detail = mod.fullPath;
                item.documentation = new vscode.MarkdownString(
                    `Module \`${mod.fullPath}\`\n\nImport with: \`import ${mod.fullPath};\`\n\n` +
                    mod.members.map((m: any) => `- \`${m.name}\`: ${m.documentation}`).join('\n')
                );
                item.insertText = mod.shortName;
                item.sortText = '02' + mod.shortName;
                if (!hasImport(text, mod.fullPath)) {
                    item.additionalTextEdits = importEditFor(document, mod.fullPath);
                }
                items.push(item);
            }
            const fnSnippet = new vscode.CompletionItem('fn', vscode.CompletionItemKind.Snippet);
            fnSnippet.insertText = new vscode.SnippetString('fn ${1:name}(${2:params}) -> ${3:void} {\n\t$0\n}');
            fnSnippet.detail = 'Function snippet (see Snippets catalog for 85 more)';
            fnSnippet.sortText = '0fn';
            items.push(fnSnippet);
            return items;
        }
    };
}

// ---------------------------------------------------------------------------
// definition
// ---------------------------------------------------------------------------

export function createDefinitionProvider(): any {
    return {
        provideDefinition(document: any, position: any): any {
            const range = document.getWordRangeAtPosition(position);
            if (!range) {
                return null;
            }
            const word = document.getText(range);
            const raw = document.getText();
            const text = maskNonCode(raw);
            const patterns: RegExp[] = [
                new RegExp(`\\b(fn|class|struct|enum|trait|interface|let|var|const|type)\\s+(?:mut\\s+)?${escapeRegExp(word)}\\b`, 'g'),
                // for-in loop var, canonical + parenthesized/let forms.
                new RegExp(`\\bfor\\s*\\(?\\s*(?:let\\s+|var\\s+|const\\s+|mut\\s+)?${escapeRegExp(word)}\\s+in\\b`, 'g'),
                // fn param `name: Type`.
                new RegExp(`\\bfn\\s+[A-Za-z_][A-Za-z0-9_]*\\s*\\([^)]*\\b${escapeRegExp(word)}\\s*:`, 'g'),
            ];
            for (const re of patterns) {
                let m: RegExpExecArray | null;
                while ((m = re.exec(text))) {
                    const wordIdx = m[0].lastIndexOf(word);
                    const start = document.positionAt(m.index + (wordIdx >= 0 ? wordIdx : 0));
                    return new vscode.Location(document.uri, new vscode.Range(start, start));
                }
            }
            // struct/class field definition: `owner.name` -> field line.
            try {
                const line = document.lineAt(position.line).text;
                const dot = findDotAccess(line, position.character);
                if (dot && dot.field) {
                    const resolved = resolveVariable(raw, dot.receiver, position.line);
                    if (resolved && resolved.decl) {
                        const f = resolved.decl.fields.find(fl => fl.name === dot.field);
                        if (f) {
                            const start = new vscode.Position(resolved.decl.line, 0);
                            return new vscode.Location(document.uri, new vscode.Range(start, start));
                        }
                    }
                }
            } catch {
                // ignore
            }
            return null;
        }
    };
}

// ---------------------------------------------------------------------------
// document symbols (upgraded: proper SymbolKinds incl. let/const/type + @Test)
// ---------------------------------------------------------------------------

export function createDocumentSymbolProvider(): any {
    return {
        provideDocumentSymbols(document: any): any[] {
            const out: any[] = [];
            // Masked: commented-out declarations (`// fn ghost`) and
            // declarations inside strings never become symbols.
            const text = maskNonCode(document.getText());
            const pushAll = (re: RegExp, kind: any, nameIdx: number, detail?: string): void => {
                let m: RegExpExecArray | null;
                re.lastIndex = 0;
                while ((m = re.exec(text))) {
                    const start = document.positionAt(m.index);
                    const line = document.lineAt(start.line);
                    out.push(new vscode.DocumentSymbol(
                        m[nameIdx], detail || '', kind, line.range, line.range
                    ));
                }
            };
            pushAll(/\bpub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Function, 1);
            pushAll(/(?<!pub\s)fn\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Function, 1);
            pushAll(/\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Struct, 1);
            pushAll(/\bclass\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Class, 1);
            pushAll(/\benum\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Enum, 1);
            pushAll(/\btrait\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Interface, 1);
            pushAll(/\binterface\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Interface, 1);
            pushAll(/^\s*(?:pub\s+)?(?:let|var|const)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)/gm, vscode.SymbolKind.Variable, 1);
            pushAll(/^\s*(?:pub\s+)?type\s+([A-Za-z_][A-Za-z0-9_]*)/gm, vscode.SymbolKind.Class, 1, 'type alias');
            // @Test methods: `@Test` on its own line (or trailing) directly above a fn.
            const lines = text.split('\n');
            for (let i = 0; i < lines.length; i++) {
                const fnm = /(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/.exec(lines[i]);
                if (!fnm) {
                    continue;
                }
                const above = i > 0 ? lines[i - 1] : '';
                if (/@Test/.test(lines[i]) || /@Test/.test(above)) {
                    const range = document.lineAt(i).range;
                    out.push(new vscode.DocumentSymbol(fnm[1], '@Test', vscode.SymbolKind.Function, range, range));
                }
            }
            return out;
        }
    };
}

// ---------------------------------------------------------------------------
// codelens
// ---------------------------------------------------------------------------

export function createCodeLensProvider(): any {
    return {
        provideCodeLenses(document: any): any[] {
            const lenses: any[] = [];
            const text = document.getText();
            const mainRe = /\b(pub\s+)?fn\s+main\s*\(/g;
            let m: RegExpExecArray | null;
            while ((m = mainRe.exec(text))) {
                const range = document.lineAt(document.positionAt(m.index).line).range;
                lenses.push(new vscode.CodeLens(range, {
                    title: '$(play) Run', tooltip: "Run with 'lex run'",
                    command: 'lexicon.run', arguments: [document.uri],
                }));
                lenses.push(new vscode.CodeLens(range, {
                    title: '$(beaker) Test', tooltip: "Run 'lex test'",
                    command: 'lexicon.test', arguments: [document.uri],
                }));
            }
            return lenses;
        }
    };
}

// ---------------------------------------------------------------------------
// pipe/arrow decorations helper
// ---------------------------------------------------------------------------

export const PIPE_PATTERN = /\|>/g;
export const ARROW_PATTERN = /->/g;

/** Pure helper: ranges of every match of `pattern` in CODE only.
 * Comments and string/char literals are masked first (length-preserving),
 * so `|>` / `->` inside `// comments`, `/* blocks *\/` or `"strings"`
 * never produce decorations; offsets still map 1:1 onto the document. */
export function collectDecorationRanges(document: any, pattern: RegExp): any[] {
    const raw = document.getText();
    const text = maskNonCode(raw);
    const out: any[] = [];
    const re = new RegExp(pattern.source, pattern.flags.includes('g') ? pattern.flags : pattern.flags + 'g');
    let m: RegExpExecArray | null;
    while ((m = re.exec(text))) {
        out.push(new vscode.Range(
            document.positionAt(m.index),
            document.positionAt(m.index + m[0].length)
        ));
    }
    return out;
}

/** Applies pipe (`|>`) and arrow (`->`) after-glyph decorations to an editor. */
export function updatePipeArrowDecorations(editor: any, pipeDecoration: any, arrowDecoration: any): void {
    if (!editor || editor.document.languageId !== LANG_ID) {
        return;
    }
    editor.setDecorations(pipeDecoration, collectDecorationRanges(editor.document, PIPE_PATTERN));
    editor.setDecorations(arrowDecoration, collectDecorationRanges(editor.document, ARROW_PATTERN));
}

export function createDecorationTypes(): { pipe: any; arrow: any } {
    return {
        pipe: vscode.window.createTextEditorDecorationType({
            after: { contentText: '▷', color: '#58A6FF', fontWeight: 'bold', margin: '0 0 0 -1ch' },
            textDecoration: 'none; display: none;',
        }),
        arrow: vscode.window.createTextEditorDecorationType({
            after: { contentText: '→', color: '#FF7B72', margin: '0 0 0 -1ch' },
            textDecoration: 'none; display: none;',
        }),
    };
}

// ---------------------------------------------------------------------------
// 1. signature help: `Mod::fn(` or known builtin + `(` shows signature
// ---------------------------------------------------------------------------

export function createSignatureHelpProvider(): any {
    return {
        provideSignatureHelp(document: any, position: any): any {
            const linePrefix = document.lineAt(position.line).text.substring(0, position.character);
            // `Mod::member(` — capture args typed so far (after the last unclosed paren).
            let detail: string | undefined;
            let doc = '';
            let argsSoFar = '';
            const modCall = /([A-Za-z_][A-Za-z0-9_]*)::([A-Za-z_][A-Za-z0-9_]*)\(([^()]*)$/.exec(linePrefix);
            if (modCall) {
                const mod = findModule(modCall[1]);
                const member = mod?.members.find(m => m.name === modCall[2]);
                if (!member) {
                    return null;
                }
                detail = member.detail;
                doc = member.documentation;
                argsSoFar = modCall[3];
            } else {
                const fnCall = /([A-Za-z_][A-Za-z0-9_]*)\(([^()]*)$/.exec(linePrefix);
                if (!fnCall) {
                    return null;
                }
                const builtin = BUILTIN_FNS.find(f => f.name === fnCall[1]);
                if (builtin) {
                    detail = builtin.detail;
                    doc = builtin.doc;
                    argsSoFar = fnCall[2];
                } else {
                    // User-defined `fn name(params) -> Ret` from the same file.
                    try {
                        const src = maskNonCode(document.getText());
                        const ure = new RegExp(`\\b(?:pub\\s+)?fn\\s+${escapeRegExp(fnCall[1])}\\s*\\(([^)]*)\\)\\s*(->\\s*[A-Za-z0-9_?<>\\[\\]]+)?`, 'g');
                        const um = ure.exec(src);
                        if (!um) {
                            return null;
                        }
                        const ret = um[2] ? ` ${um[2].trim()}` : '';
                        detail = `fn ${fnCall[1]}(${um[1].trim()})${ret}`;
                        doc = 'User-defined function (this file).';
                        argsSoFar = fnCall[2];
                    } catch {
                        return null;
                    }
                }
            }
            const params = paramsFromDetail(detail);
            const activeParameter = params.length === 0
                ? 0
                : Math.min(countTopLevelCommas(argsSoFar), Math.max(0, params.length - 1));
            const sig = new vscode.SignatureInformation(detail, new vscode.MarkdownString(doc));
            sig.parameters = params.map((p: string) => new vscode.ParameterInformation(p));
            const help = new vscode.SignatureHelp();
            help.signatures = [sig];
            help.activeSignature = 0;
            help.activeParameter = activeParameter;
            return help;
        }
    };
}

// ---------------------------------------------------------------------------
// 2. inlay hints — literals only, never wrong
// ---------------------------------------------------------------------------

export function createInlayHintsProvider(): any {
    return {
        provideInlayHints(document: any, range: any): any[] {
            const hints: any[] = [];
            const text = document.getText();
            const lines = text.split('\n');
            const inRange = (ln: number): boolean => {
                if (!range) {
                    return true;
                }
                return ln >= range.start.line && ln <= range.end.line;
            };
            // `let x = <expr>;` (no explicit type) -> `: <inferred>` after the name.
            // Now covers literals, struct literals, aliases and List element
            // types — still conservative (inference returns undefined unsure).
            // Also `for x in Owners` loop vars get `: Element`.
            const declsForHints = (() => { try { return parseTypeDecls(text); } catch { return []; } })();
            const bindingsForHints = (() => { try { return parseBindings(text); } catch { return []; } })();
            const pushTypeHint = (ln: number, name: string, label: string): void => {
                const nameIdx = lines[ln].indexOf(name);
                if (nameIdx < 0) {
                    return;
                }
                hints.push(new vscode.InlayHint(
                    new vscode.Position(ln, nameIdx + name.length),
                    label,
                    vscode.InlayHintKind.Type
                ));
            };
            for (let i = 0; i < lines.length; i++) {
                if (!inRange(i)) {
                    continue;
                }
                // for-in loop var without annotation.
                const loop = /\bfor\s*\(?\s*(?:let\s+|var\s+|const\s+|mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s+in\s+([^\)\{\n;]+)\)?/.exec(lines[i]);
                if (loop && !/:\s*[A-Za-z_][A-Za-z0-9_?]/.test(lines[i])) {
                    const iterExpr = loop[2].trim().replace(/\)\s*$/, '').trim();
                    const iterType = inferExprType(iterExpr, bindingsForHints, declsForHints);
                    const el = elementTypeOf(iterType, iterExpr, bindingsForHints, declsForHints);
                    if (el.type) {
                        pushTypeHint(i, loop[1], `: ${el.type}`);
                        continue;
                    }
                    if (iterType) {
                        pushTypeHint(i, loop[1], `: ${iterType} (iter)`);
                        continue;
                    }
                }
                const m = /^\s*(?:let|const|var)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+?);\s*(?:\/\/.*)?$/.exec(lines[i]);
                if (!m) {
                    continue;
                }
                // Skip bindings that already carry an explicit `: Type`.
                if (/^\s*(?:let|const|var)\s+(?:mut\s+)?[A-Za-z_][A-Za-z0-9_]*\s*:/.test(lines[i])) {
                    continue;
                }
                const inferred = inferExprType(m[2], bindingsForHints, declsForHints) || inferLiteralType(m[2]);
                if (!inferred) {
                    continue;
                }
                pushTypeHint(i, m[1], `: ${inferred}`);
            }
            // `fn name(...)` without `->` -> `-> <type>` only when a
            // `return <literal>;` exists in the body and all agree.
            const fnRe = /^\s*(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*\)\s*(\{[ \t]*|\{[ \t]*(?:\/\/.*)?$|$)/;
            for (let i = 0; i < lines.length; i++) {
                if (!inRange(i)) {
                    continue;
                }
                const decl = /^\s*(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*\)\s*(->\s*[A-Za-z0-9_?<>]+)?\s*(\{|$)/.exec(lines[i]);
                if (!decl || decl[2]) {
                    continue; // has an explicit return type (or not a decl line)
                }
                void fnRe;
                // scan the brace-balanced body for `return <literal>;`
                let depth = 0;
                let started = false;
                let foundType: string | undefined;
                let agree = true;
                for (let j = i; j < lines.length; j++) {
                    const ln = lines[j].replace(/\/\/.*$/, '');
                    for (const c of ln) {
                        if (c === '{') {
                            depth++;
                            started = true;
                        } else if (c === '}') {
                            depth--;
                        }
                    }
                    const ret = /^\s*return\s+(.+?);\s*$/.exec(ln);
                    if (ret && started) {
                        const t = inferLiteralType(ret[1]);
                        if (!t) {
                            agree = false; // non-literal return: cannot infer honestly
                        } else if (foundType === undefined) {
                            foundType = t;
                        } else if (foundType !== t) {
                            agree = false;
                        }
                    }
                    if (started && depth <= 0 && j > i) {
                        break;
                    }
                    if (j - i > 500) {
                        break; // safety cap
                    }
                }
                if (foundType && agree) {
                    hints.push(new vscode.InlayHint(
                        new vscode.Position(i, lines[i].length),
                        `-> ${foundType}`,
                        vscode.InlayHintKind.Type
                    ));
                }
            }
            return hints;
        }
    };
}

// ---------------------------------------------------------------------------
// 3. folding ranges: fn/struct/class/enum/trait/interface/match/switch + {}
// ---------------------------------------------------------------------------

function stripLineForBraces(ln: string): string {
    // drop line comments and double-quoted strings so braces inside them don't count
    return ln.replace(/\/\/.*$/, '').replace(/"(?:[^"\\]|\\.)*"/g, '""');
}

export function computeFoldingRanges(lines: string[]): Array<{ start: number; end: number }> {
    // Keep the original lines for comment folding; brace folding works on
    // masked code so braces inside comments/strings never open folds.
    // (Masking preserves length, so line numbers are unaffected.)
    const original = lines.slice();
    const masked = maskNonCode(lines.join('\n')).split('\n');
    lines = masked;
    const found: Array<{ start: number; end: number }> = [];
    const seen = new Set<string>();
    const add = (start: number, end: number): void => {
        if (end - start < 1) {
            return;
        }
        const key = `${start}:${end}`;
        if (!seen.has(key)) {
            seen.add(key);
            found.push({ start, end });
        }
    };
    const closeFrom = (fromLine: number): number => {
        let depth = 0;
        let started = false;
        for (let j = fromLine; j < lines.length; j++) {
            const ln = stripLineForBraces(lines[j]);
            for (const c of ln) {
                if (c === '{') {
                    depth++;
                    started = true;
                } else if (c === '}') {
                    depth--;
                }
            }
            if (started && depth <= 0) {
                return j;
            }
        }
        return -1;
    };
    // keyword blocks first
    for (let i = 0; i < lines.length; i++) {
        const t = lines[i].trim();
        if (/^(pub\s+)?(fn|struct|class|enum|trait|interface)\b/.test(t) || /^(match|switch)\b/.test(t) || /\b(match|switch)\b/.test(t)) {
            const end = closeFrom(i);
            if (end > i) {
                add(i, end);
            }
        }
    }
    // generic multi-line brace ranges
    for (let i = 0; i < lines.length; i++) {
        if (stripLineForBraces(lines[i]).includes('{')) {
            const end = closeFrom(i);
            if (end > i) {
                add(i, end);
            }
        }
    }
    // block comments `/* ... */` spanning multiple lines (from original:
    // masked text erases the delimiters, so scan the raw lines; a `/*`
    // trailing after `//` on the same line is a line comment, not a block).
    {
        let blockStart = -1;
        for (let i = 0; i < original.length; i++) {
            const ln = original[i];
            if (blockStart < 0) {
                const open = ln.indexOf('/*');
                if (open < 0) {
                    continue;
                }
                const lineComment = ln.indexOf('//');
                if (lineComment >= 0 && lineComment < open) {
                    continue;
                }
                const close = ln.indexOf('*/', open + 2);
                if (close >= 0) {
                    continue; // single-line block comment: no fold
                }
                blockStart = i;
            } else {
                if (ln.indexOf('*/') >= 0) {
                    add(blockStart, i);
                    blockStart = -1;
                }
            }
        }
    }
    // runs of `//` line comments with >= 5 lines (from original lines).
    {
        let runStart = -1;
        let runLen = 0;
        const flush = (): void => {
            if (runStart >= 0 && runLen >= 5) {
                add(runStart, runStart + runLen - 1);
            }
            runStart = -1;
            runLen = 0;
        };
        for (let i = 0; i <= original.length; i++) {
            const ln = i < original.length ? original[i] : '';
            const isLineComment = i < original.length && /^\s*\/\//.test(ln);
            if (isLineComment) {
                if (runStart < 0) {
                    runStart = i;
                    runLen = 1;
                } else {
                    runLen++;
                }
            } else {
                flush();
            }
        }
    }
    found.sort((a, b) => a.start - b.start || a.end - b.end);
    return found;
}

export function createFoldingRangeProvider(): any {
    return {
        provideFoldingRanges(document: any): any[] {
            const lines = document.getText().split('\n');
            return computeFoldingRanges(lines).map(r => new vscode.FoldingRange(r.start, r.end));
        }
    };
}

// ---------------------------------------------------------------------------
// 4. document colors: #RGB / #RRGGBB inside strings
// ---------------------------------------------------------------------------

function parseHexColor(hex: string): { r: number; g: number; b: number } | undefined {
    const h = hex.startsWith('#') ? hex.slice(1) : hex;
    const expand = h.length === 3 ? h.split('').map(c => c + c).join('') : h;
    if (!/^[0-9a-fA-F]{6}$/.test(expand)) {
        return undefined;
    }
    return {
        r: parseInt(expand.slice(0, 2), 16) / 255,
        g: parseInt(expand.slice(2, 4), 16) / 255,
        b: parseInt(expand.slice(4, 6), 16) / 255,
    };
}

export function createDocumentColorProvider(): any {
    return {
        provideDocumentColors(document: any): any[] {
            const out: any[] = [];
            const lines = document.getText().split('\n');
            for (let i = 0; i < lines.length; i++) {
                const ln = lines[i];
                const strRe = /"(?:[^"\\\n]|\\.)*"/g;
                let sm: RegExpExecArray | null;
                while ((sm = strRe.exec(ln))) {
                    const inner = sm[0];
                    const base = sm.index;
                    const hexRe = /#([0-9a-fA-F]{6}|[0-9a-fA-F]{3})\b/g;
                    let hm: RegExpExecArray | null;
                    while ((hm = hexRe.exec(inner))) {
                        const rgb = parseHexColor(hm[0]);
                        if (!rgb) {
                            continue;
                        }
                        out.push(new vscode.DocumentColor(
                            new vscode.Color(rgb.r, rgb.g, rgb.b, 1),
                            new vscode.Range(
                                new vscode.Position(i, base + hm.index),
                                new vscode.Position(i, base + hm.index + hm[0].length)
                            )
                        ));
                    }
                }
            }
            return out;
        }
    };
}

// ---------------------------------------------------------------------------
// 5. references (workspace-wide) + 6. rename (workspace-wide with preview)
// ---------------------------------------------------------------------------

async function collectLexDocuments(exclude?: any): Promise<Array<{ uri: any; text: string }>> {
    const out: Array<{ uri: any; text: string }> = [];
    try {
        const uris = await vscode.workspace.findFiles('**/*.lex', '**/target/**', 200);
        for (const uri of uris) {
            if (exclude && uri.toString() === exclude.toString()) {
                continue;
            }
            try {
                const doc = await vscode.workspace.openTextDocument(uri);
                out.push({ uri, text: doc.getText() });
            } catch {
                // unreadable file: skip
            }
        }
    } catch {
        // no workspace: caller falls back to current file
    }
    return out;
}

export function createReferenceProvider(): any {
    return {
        async provideReferences(document: any, position: any, _context: any): Promise<any[]> {
            const wordRange = document.getWordRangeAtPosition(position);
            if (!wordRange) {
                return [];
            }
            const word = document.getText(wordRange);
            const docs = await collectLexDocuments();
            if (docs.length === 0) {
                // Fallback: current file only.
                const text = document.getText();
                return findWholeWordOffsets(text, word).map(o => new vscode.Location(
                    document.uri,
                    new vscode.Range(document.positionAt(o.start), document.positionAt(o.end))
                ));
            }
            const locations: any[] = [];
            for (const d of docs) {
                try {
                    const doc = await vscode.workspace.openTextDocument(d.uri);
                    for (const o of findWholeWordOffsets(d.text, word)) {
                        locations.push(new vscode.Location(
                            d.uri,
                            new vscode.Range(doc.positionAt(o.start), doc.positionAt(o.end))
                        ));
                    }
                } catch {
                    // skip
                }
            }
            // Always include the current file even if findFiles missed it.
            if (!docs.some(d => d.uri.toString() === document.uri.toString())) {
                const text = document.getText();
                for (const o of findWholeWordOffsets(text, word)) {
                    locations.push(new vscode.Location(
                        document.uri,
                        new vscode.Range(document.positionAt(o.start), document.positionAt(o.end))
                    ));
                }
            }
            return locations;
        }
    };
}

export function createRenameProvider(): any {
    return {
        prepareRename(document: any, position: any): any {
            const wordRange = document.getWordRangeAtPosition(position);
            if (!wordRange) {
                throw new Error('Lexicon: nothing to rename here.');
            }
            const word = document.getText(wordRange);
            if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(word)) {
                throw new Error('Lexicon: cannot rename this token.');
            }
            return wordRange;
        },
        async provideRenameEdits(document: any, position: any, newName: string): Promise<any> {
            const wordRange = document.getWordRangeAtPosition(position);
            if (!wordRange) {
                return null;
            }
            const word = document.getText(wordRange);
            const edit = new vscode.WorkspaceEdit();
            const docs = await collectLexDocuments();
            if (docs.length === 0) {
                const text = document.getText();
                for (const o of findWholeWordOffsets(text, word)) {
                    edit.replace(
                        document.uri,
                        new vscode.Range(document.positionAt(o.start), document.positionAt(o.end)),
                        newName
                    );
                }
                return edit;
            }
            for (const d of docs) {
                try {
                    const doc = await vscode.workspace.openTextDocument(d.uri);
                    for (const o of findWholeWordOffsets(d.text, word)) {
                        edit.replace(
                            d.uri,
                            new vscode.Range(doc.positionAt(o.start), doc.positionAt(o.end)),
                            newName
                        );
                    }
                } catch {
                    // skip
                }
            }
            if (!docs.some(d => d.uri.toString() === document.uri.toString())) {
                const text = document.getText();
                for (const o of findWholeWordOffsets(text, word)) {
                    edit.replace(
                        document.uri,
                        new vscode.Range(document.positionAt(o.start), document.positionAt(o.end)),
                        newName
                    );
                }
            }
            return edit;
        }
    };
}

export function createWorkspaceSymbolProvider(): any {
    const KIND_RE: Array<[RegExp, any, string?]> = [
        [/\bpub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Function],
        [/(?<!pub\s)fn\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Function],
        [/\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Struct],
        [/\bclass\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Class],
        [/\benum\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Enum],
        [/\btrait\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Interface],
        [/\binterface\s+([A-Za-z_][A-Za-z0-9_]*)/g, vscode.SymbolKind.Interface],
    ];
    return {
        async provideWorkspaceSymbols(query: string): Promise<any[]> {
            const out: any[] = [];
            const q = (query || '').toLowerCase();
            const docs = await collectLexDocuments();
            for (const d of docs) {
                const base = d.uri.path.split('/').pop() || d.uri.toString();
                for (const [re, kind] of KIND_RE) {
                    const rx = new RegExp(re.source, re.flags);
                    let m: RegExpExecArray | null;
                    while ((m = rx.exec(d.text)) !== null) {
                        const name: string = m[1];
                        if (q && !name.toLowerCase().includes(q)) {
                            continue;
                        }
                        const info = new vscode.SymbolInformation(
                            name,
                            kind,
                            base,
                            new vscode.Location(d.uri, new vscode.Position(0, 0))
                        );
                        out.push(info);
                        if (out.length >= 500) {
                            return out;
                        }
                    }
                }
            }
            return out;
        }
    };
}

// ---------------------------------------------------------------------------
// 7. semantic tokens (regex-based, deterministic, no overlap)
// ---------------------------------------------------------------------------

export const SEMANTIC_TOKEN_TYPES = ['namespace', 'type', 'function', 'variable', 'parameter', 'decorator', 'comment'];

export function createSemanticTokensLegend(): any {
    return new vscode.SemanticTokensLegend(SEMANTIC_TOKEN_TYPES, []);
}

interface RawToken { start: number; length: number; type: string; }

/** Pure tokenizer: char-offset tokens, sorted, non-overlapping. */
export function computeSemanticTokens(text: string): RawToken[] {
    const typeIdx = (t: string): boolean => SEMANTIC_TOKEN_TYPES.includes(t);
    void typeIdx;
    const raw: RawToken[] = [];
    const taken: Array<{ start: number; end: number }> = [];
    const claim = (start: number, length: number, type: string): void => {
        if (length <= 0) {
            return;
        }
        const end = start + length;
        for (const t of taken) {
            if (start < t.end && end > t.start) {
                return; // overlap: first claim wins (deterministic)
            }
        }
        taken.push({ start, end });
        raw.push({ start, length, type });
    };
    // Masked: identifiers inside comments/strings (e.g. `"fn"`, `// let x`)
    // never become tokens. The per-line `//`/string guards below stay as
    // defense in depth.
    const lines = maskNonCode(text).split('\n');
    const offsets: number[] = [];
    let acc = 0;
    for (const ln of lines) {
        offsets.push(acc);
        acc += ln.length + 1;
    }
    const at = (line: number, col: number): number => offsets[line] + col;

    for (let i = 0; i < lines.length; i++) {
        const ln = lines[i];
        let m: RegExpExecArray | null;
        // Claims are offset ranges confined to a single line (every `at(i, …)`
        // start lies within line i and no token spans a newline), so overlap
        // can only occur within the current line. Resetting the guard each
        // iteration keeps the "first claim wins" rule while avoiding an
        // O(n²) scan that grows with the whole file.
        taken.length = 0;
        // comments (highest priority on the line tail)
        const cm = /\/\/.*$/.exec(ln);
        const codeEnd = cm ? cm.index : ln.length;
        const code = ln.slice(0, codeEnd);
        if (cm) {
            claim(at(i, cm.index), ln.length - cm.index, 'comment');
        }
        // decorators: @Attr and #[cfg(...)]
        const decRe = /@[\w]+|#\[[\w]+/g;
        while ((m = decRe.exec(code))) {
            claim(at(i, m.index), m[0].length, 'decorator');
        }
        // strings: claim nothing, but remember spans to exclude
        const strSpans: Array<{ start: number; end: number }> = [];
        const strRe = /"(?:[^"\\]|\\.)*"/g;
        while ((m = strRe.exec(code))) {
            strSpans.push({ start: m.index, end: m.index + m[0].length });
        }
        const inStr = (idx: number): boolean => strSpans.some(s => idx >= s.start && idx < s.end);
        // `Mod::` namespace prefixes
        const nsRe = /([A-Za-z_][A-Za-z0-9_]*)::/g;
        while ((m = nsRe.exec(code))) {
            if (!inStr(m.index)) {
                claim(at(i, m.index), m[1].length, 'namespace');
            }
        }
        // `::member`: PascalCase -> type, otherwise -> function
        const memRe = /::([A-Za-z_][A-Za-z0-9_]*)/g;
        while ((m = memRe.exec(code))) {
            if (inStr(m.index)) {
                continue;
            }
            const kind = /^[A-Z]/.test(m[1]) ? 'type' : 'function';
            claim(at(i, m.index + 2), m[1].length, kind);
        }
        // `fn name` definitions
        const fnRe = /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)/g;
        while ((m = fnRe.exec(code))) {
            if (!inStr(m.index)) {
                claim(at(i, m.index + m[0].lastIndexOf(m[1])), m[1].length, 'function');
            }
        }
        // `let|const|var [mut] name` bindings + `for [let] x in` loop vars.
        const letRe = /\b(?:let|const|var)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)/g;
        while ((m = letRe.exec(code))) {
            if (!inStr(m.index)) {
                claim(at(i, m.index + m[0].lastIndexOf(m[1])), m[1].length, 'variable');
            }
        }
        const forRe = /\bfor\s*\(?\s*(?:let\s+|var\s+|const\s+|mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s+in\b/g;
        while ((m = forRe.exec(code))) {
            if (!inStr(m.index)) {
                claim(at(i, m.index + m[0].lastIndexOf(m[1])), m[1].length, 'variable');
            }
        }
        // params inside `fn name(...)`
        const sigRe = /\bfn\s+[A-Za-z_][A-Za-z0-9_]*\s*\(([^)]*)\)/g;
        while ((m = sigRe.exec(code))) {
            const inner = m[1];
            const base = m.index + m[0].indexOf(inner);
            const paramRe = /([A-Za-z_][A-Za-z0-9_]*)\s*:/g;
            let pm: RegExpExecArray | null;
            while ((pm = paramRe.exec(inner))) {
                if (!inStr(base + pm.index)) {
                    claim(at(i, base + pm.index), pm[1].length, 'parameter');
                }
            }
        }
        // remaining PascalCase identifiers -> type
        const tyRe = /\b([A-Z][A-Za-z0-9_]*)\b/g;
        while ((m = tyRe.exec(code))) {
            if (!inStr(m.index)) {
                claim(at(i, m.index), m[0].length, 'type');
            }
        }
    }
    raw.sort((a, b) => a.start - b.start);
    return raw;
}

export function createSemanticTokensProvider(): any {
    return {
        provideDocumentSemanticTokens(document: any): any {
            const text = document.getText();
            const tokens = computeSemanticTokens(text);
            const builder = new vscode.SemanticTokensBuilder(createSemanticTokensLegend());
            const idxOf = (t: string): number => SEMANTIC_TOKEN_TYPES.indexOf(t);
            for (const tok of tokens) {
                const start = document.positionAt(tok.start);
                builder.push(start.line, start.character, tok.length, idxOf(tok.type), 0);
            }
            return builder.build();
        }
    };
}

// ---------------------------------------------------------------------------
// 8. quick fixes (all text-based and safe)
// ---------------------------------------------------------------------------

export function createQuickFixProvider(): any {
    return {
        provideCodeActions(document: any, range: any): any[] {
            const actions: any[] = [];
            const text = document.getText();
            // Masked: `// X::y`, `x == true` inside comments/strings and
            // ghost `match` blocks never produce fixes. hasImport below
            // still reads the original text (imports are code).
            const lines = maskNonCode(text).split('\n');
            const fix = (title: string, build: (edit: any) => void): any => {
                const action = new vscode.CodeAction(title, vscode.CodeActionKind.QuickFix);
                const edit = new vscode.WorkspaceEdit();
                build(edit);
                action.edit = edit;
                return action;
            };
            const startLine = range ? range.start.line : 0;
            const endLine = range ? range.end.line : lines.length - 1;
            const lo = Math.max(0, startLine - 2);
            const hi = Math.min(lines.length - 1, endLine + 2);

            // (a) `X::` where X is a known stdlib module but not imported.
            for (let i = lo; i <= hi; i++) {
                const useRe = /\b([A-Za-z_][A-Za-z0-9_]*)::/g;
                let m: RegExpExecArray | null;
                const seenMod = new Set<string>();
                while ((m = useRe.exec(lines[i]))) {
                    const short = m[1];
                    if (seenMod.has(short)) {
                        continue;
                    }
                    seenMod.add(short);
                    const mod = findModule(short);
                    if (mod && !hasImport(text, mod.fullPath)) {
                        const fullPath = mod.fullPath;
                        actions.push(fix(`Add import \`import ${fullPath};\``, (edit: any) => {
                            for (const e of importEditFor(document, fullPath)) {
                                edit.insert(document.uri, e.range ? e.range.start : new vscode.Position(0, 0), e.newText);
                            }
                        }));
                    }
                }
            }

            // (b) `== true` / `== false` simplification.
            for (let i = lo; i <= hi; i++) {
                const cmpRe = /([A-Za-z_0-9_\)\]]+)\s*==\s*(true|false)/g;
                let m: RegExpExecArray | null;
                while ((m = cmpRe.exec(lines[i]))) {
                    const expr = m[1];
                    const replacement = m[2] === 'true' ? expr : `!${expr}`;
                    const s = m.index;
                    const e = m.index + m[0].length;
                    actions.push(fix(
                        m[2] === 'true' ? `Simplify \`${m[0]}\` to \`${expr}\`` : `Simplify \`${m[0]}\` to \`${replacement}\``,
                        (edit: any) => {
                            edit.replace(
                                document.uri,
                                new vscode.Range(new vscode.Position(i, s), new vscode.Position(i, e)),
                                replacement
                            );
                        }
                    ));
                }
            }

            // (c) `match` without a wildcard `_ =>` arm / `switch` without `default:`.
            const blockClose = (fromLine: number): number => {
                let depth = 0;
                let started = false;
                for (let j = fromLine; j < lines.length; j++) {
                    const ln = stripLineForBraces(lines[j]);
                    for (const c of ln) {
                        if (c === '{') {
                            depth++;
                            started = true;
                        } else if (c === '}') {
                            depth--;
                        }
                    }
                    if (started && depth <= 0) {
                        return j;
                    }
                }
                return -1;
            };
            for (let i = 0; i < lines.length; i++) {
                const t = lines[i];
                if (/\bmatch\b/.test(t) && t.includes('{')) {
                    const close = blockClose(i);
                    if (close > i) {
                        const body = lines.slice(i, close + 1).join('\n');
                        if (!/(^|\s)_\s*=>/.test(body)) {
                            const insertAt = close;
                            actions.push(fix('Add wildcard arm `_ => ...` to match', (edit: any) => {
                                edit.insert(document.uri, new vscode.Position(insertAt, 0), '    _ => todo,\n');
                            }));
                        }
                    }
                }
                if (/\bswitch\b/.test(t) && t.includes('{')) {
                    const close = blockClose(i);
                    if (close > i) {
                        const body = lines.slice(i, close + 1).join('\n');
                        if (!/default\s*:/.test(body)) {
                            const insertAt = close;
                            actions.push(fix('Add `default:` arm to switch', (edit: any) => {
                                edit.insert(document.uri, new vscode.Position(insertAt, 0), '    default: break;\n');
                            }));
                        }
                    }
                }
            }

            // (d) JS-like `for (let x in xs)` / `for (x in xs)` -> canonical
            // `for x in xs` (the parser takes a bare pattern, no `let`/parens).
            for (let i = lo; i <= hi; i++) {
                const jm = /\bfor\s*\(\s*(?:let\s+|var\s+|const\s+|mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s+in\s+([^\)\{\n;]+?)\)/.exec(lines[i]);
                if (jm) {
                    const canon = `for ${jm[1]} in ${jm[2].trim()}`;
                    const s = jm.index;
                    const e = jm.index + jm[0].length;
                    actions.push(fix(
                        `Use canonical for-in: \`${canon}\``,
                        (edit: any) => {
                            edit.replace(
                                document.uri,
                                new vscode.Range(new vscode.Position(i, s), new vscode.Position(i, e)),
                                canon
                            );
                        }
                    ));
                }
            }
            return actions;
        }
    };
}

// ---------------------------------------------------------------------------
// 10. test discovery (pure vscode: workspace.findFiles + openTextDocument)
// ---------------------------------------------------------------------------

export interface DiscoveredTest {
    uri: any;
    fnName: string;
    line: number;
}

export async function discoverTests(): Promise<DiscoveredTest[]> {
    const out: DiscoveredTest[] = [];
    let files: any[] = [];
    try {
        const found = await vscode.workspace.findFiles('**/*.lex', '**/node_modules/**');
        files = Array.isArray(found) ? found : [];
    } catch {
        return [];
    }
    for (const uri of files) {
        let doc: any;
        try {
            doc = await vscode.workspace.openTextDocument(uri);
        } catch {
            continue;
        }
        let lines: string[];
        try {
            lines = doc.getText().split('\n');
        } catch {
            continue;
        }
        for (let i = 0; i < lines.length; i++) {
            const fnm = /(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/.exec(lines[i]);
            if (!fnm) {
                continue;
            }
            const isTestPrefixed = fnm[1].startsWith('test_');
            const hasAttr = /@Test/.test(lines[i]) || (i > 0 && /@Test/.test(lines[i - 1]));
            if (isTestPrefixed || hasAttr) {
                out.push({ uri, fnName: fnm[1], line: i });
            }
        }
    }
    return out;
}

// ---------------------------------------------------------------------------
// 11. task provider for task type `lex` (binary path injected from entry)
// ---------------------------------------------------------------------------

function lexTaskProblemMatcher(cmd: string): string[] {
    switch (cmd) {
        case 'build': return ['lexicon-build'];
        case 'check': return ['lexicon-check'];
        case 'test':
        case 'bench': return ['lexicon-test'];
        case 'vet':
        case 'lint': return ['lexicon-vet'];
        // Network commands emit PASS/FAIL or server logs, not file:line
        // diagnostics — no matcher (attaching one would false-positive).
        case 'serve':
        case 'test-net':
        case 'dns-check':
        case 'tls-check': return [];
        default: return [];
    }
}

function lexTaskArgsFor(def: any): string[] {
    const cmd = typeof def.command === 'string' && def.command.length > 0 ? def.command : 'run';
    const file = typeof def.file === 'string' && def.file.length > 0 ? def.file : undefined;
    const extra: string[] = Array.isArray(def.args) ? def.args.filter((a: any) => typeof a === 'string') : [];
    const host = typeof def.host === 'string' && def.host.length > 0 ? def.host : '127.0.0.1';
    const port = typeof def.port === 'number' && Number.isFinite(def.port) ? def.port : 3000;
    const args: string[] = [cmd];
    switch (cmd) {
        case 'run':
        case 'debug':
        case 'trace':
        case 'check':
        case 'vet':
        case 'doc':
            if (file) { args.push(file); }
            args.push(...extra);
            break;
        case 'build':
            if (file) { args.push(file); }
            if (def.release) { args.push('--release'); }
            args.push(...extra);
            break;
        case 'lint':
        case 'test':
        case 'bench':
            args.push(...extra);
            break;
        case 'fmt':
            if (file) { args.push(file); }
            args.push(...extra);
            break;
        case 'serve':
            if (file) { args.push(file); }
            args.push('--host', host, '--port', String(port), ...extra);
            break;
        case 'test-net':
            args.push('--host', host, '--port', String(port), ...extra);
            break;
        case 'dns-check': {
            // CLI default host is example.com (not 127.0.0.1) and it
            // accepts --timeout like 5s/500ms.
            const dnsHost = (typeof def.host === 'string' && def.host.length > 0 ? def.host : 'example.com');
            const dnsTimeout = (typeof def.timeout === 'string' && def.timeout.length > 0 ? def.timeout : '5s');
            args.push('--host', dnsHost, '--timeout', dnsTimeout, ...extra);
            break;
        }
        case 'tls-check': {
            // CLI defaults: host example.com, port 443.
            const tlsHost = (typeof def.host === 'string' && def.host.length > 0 ? def.host : 'example.com');
            const tlsPort = (typeof def.port === 'number' && Number.isFinite(def.port) ? def.port : 443);
            args.push('--host', tlsHost, '--port', String(tlsPort), ...extra);
            break;
        }
        case 'profile':
            if (file) { args.push(file); }
            args.push(...extra);
            break;
        default:
            if (file) { args.push(file); }
            args.push(...extra);
            break;
    }
    if (def.watch === true && (cmd === 'run' || cmd === 'serve')) {
        args.push('--watch');
    }
    // --features/--target from `lexicon.features` / `lexicon.target`
    // (mirrors extension.ts lexArgs: features for build/check/vet, target for build).
    try {
        const cfg = vscode.workspace.getConfiguration('lexicon');
        const features = (((cfg.get('features') as string | undefined) || '').trim());
        const target = (((cfg.get('target') as string | undefined) || '').trim());
        if (features.length > 0 && (cmd === 'build' || cmd === 'check' || cmd === 'vet')) {
            args.push('--features', features);
        }
        if (target.length > 0 && cmd === 'build') {
            args.push('--target', target);
        }
    } catch {
        // config unavailable in tests/web stubs: skip injection
    }
    return args;
}

function lexTaskDetail(cmd: string, args: string[]): string {
    return `lex ${args.join(' ')} (${cmd})`;
}

function makeLexTask(getBin: () => string, def: any, opts?: { group?: any; isBackground?: boolean; presentation?: any }): any {
    const args = lexTaskArgsFor(def);
    const cmd = args[0] || 'run';
    let cwd: string | undefined;
    try {
        const folders = vscode.workspace.workspaceFolders;
        if (Array.isArray(folders) && folders.length > 0 && folders[0].uri) {
            cwd = folders[0].uri.fsPath;
        }
    } catch {
        cwd = undefined;
    }
    const env = def.env && typeof def.env === 'object' && !Array.isArray(def.env) ? def.env : undefined;
    const execOpts: any = {};
    if (cwd) { execOpts.cwd = cwd; }
    if (env) { execOpts.env = env; }
    const execution = new vscode.ShellExecution(getBin(), args, execOpts);
    const task = new vscode.Task(
        def,
        vscode.TaskScope.WorkspaceFolder,
        `lex ${args.join(' ')}`,
        'lex',
        execution,
        lexTaskProblemMatcher(cmd)
    );
    try {
        if (opts && opts.group) {
            task.group = opts.group;
        } else if (cmd === 'build') {
            task.group = vscode.TaskGroup.Build;
        } else if (cmd === 'test') {
            task.group = vscode.TaskGroup.Test;
        }
    } catch {
        // TaskGroup may be missing in stubs
    }
    task.detail = lexTaskDetail(cmd, args);
    task.presentationOptions = (opts && opts.presentation) || { reveal: 'always', panel: 'shared' };
    if ((opts && opts.isBackground) || cmd === 'serve') {
        try { task.isBackground = true; } catch { /* stub */ }
    }
    task.problemMatchers = lexTaskProblemMatcher(cmd);
    try {
        task.options = { cwd, env };
    } catch {
        // optional
    }
    return task;
}

export function createLexTaskProvider(getBin: () => string): any {
    return {
        provideTasks(_token: any): any[] {
            const build = makeLexTask(getBin, { type: 'lex', command: 'build' }, {
                group: (() => { try { return vscode.TaskGroup.Build; } catch { return { kind: 'build', isDefault: true }; } })(),
                presentation: { reveal: 'always', panel: 'shared' },
            });
            try { build.group = { kind: 'build', isDefault: true }; } catch { /* keep TaskGroup */ }
            // Mark defaults explicitly for the auto-detected pair.
            try {
                build.group = vscode.TaskGroup.Build;
                (build as any).isDefault = true;
            } catch { /* stub */ }
            const test = makeLexTask(getBin, { type: 'lex', command: 'test' }, {
                group: (() => { try { return vscode.TaskGroup.Test; } catch { return { kind: 'test', isDefault: true }; } })(),
                presentation: { reveal: 'always', panel: 'shared' },
            });
            const check = makeLexTask(getBin, { type: 'lex', command: 'check' }, {
                presentation: { reveal: 'silent', panel: 'shared' },
            });
            const vet = makeLexTask(getBin, { type: 'lex', command: 'vet' });
            const lint = makeLexTask(getBin, { type: 'lex', command: 'lint' });
            const fmt = makeLexTask(getBin, { type: 'lex', command: 'fmt', args: ['--check'] });
            const serve = makeLexTask(
                getBin,
                { type: 'lex', command: 'serve', host: '127.0.0.1', port: 3000 },
                { isBackground: true, presentation: { reveal: 'always', panel: 'dedicated' } }
            );
            const testNet = makeLexTask(
                getBin,
                { type: 'lex', command: 'test-net', host: '127.0.0.1', port: 3000 },
                { presentation: { reveal: 'always', panel: 'shared' } }
            );
            const dnsCheck = makeLexTask(
                getBin,
                { type: 'lex', command: 'dns-check', host: 'example.com', timeout: '5s' },
                { presentation: { reveal: 'always', panel: 'shared' } }
            );
            const tlsCheck = makeLexTask(
                getBin,
                { type: 'lex', command: 'tls-check', host: 'example.com', port: 443 },
                { presentation: { reveal: 'always', panel: 'shared' } }
            );
            // Ensure default flags survive stubs that ignore TaskGroup objects.
            const withDefaults = (t: any, kind: string): any => {
                try {
                    if (kind === 'build') { t.group = { kind: 'build', isDefault: true }; }
                    if (kind === 'test') { t.group = { kind: 'test', isDefault: true }; }
                } catch { /* ignore */ }
                return t;
            };
            withDefaults(build, 'build');
            withDefaults(test, 'test');
            return [build, test, check, vet, lint, fmt, serve, testNet, dnsCheck, tlsCheck];
        },
        resolveTask(task: any, _token: any): any {
            const def = (task && task.definition) || {};
            if (!def.type) { def.type = 'lex'; }
            if (typeof def.command !== 'string' || def.command.length === 0) { def.command = 'run'; }
            const isServe = def.command === 'serve';
            const presentation = isServe
                ? { reveal: 'always', panel: 'dedicated' }
                : def.command === 'check'
                    ? { reveal: 'silent', panel: 'shared' }
                    : { reveal: 'always', panel: 'shared' };
            return makeLexTask(getBin, def, { isBackground: isServe || undefined, presentation });
        }
    };
}

// ---------------------------------------------------------------------------
// 12. formatting provider (runner injected from entry; re-read via workspace.fs)
// ---------------------------------------------------------------------------

export function createFormattingProvider(runLex: (args: string[]) => Promise<{ code: number; out: string }>): any {
    return {
        async provideDocumentFormattingEdits(document: any, _options: any, _token: any): Promise<any[]> {
            try {
                if (document.isDirty) {
                    try {
                        await document.save();
                    } catch {
                        // fall through: formatter works on the file on disk
                    }
                }
                const file = document.fileName;
                if (!file) {
                    return [];
                }
                await runLex(['fmt', file]);
                let fresh: string | undefined;
                try {
                    const bytes = await vscode.workspace.fs.readFile(document.uri);
                    const Dec = (globalThis as any).TextDecoder;
                    fresh = typeof Dec === 'function' ? new Dec().decode(bytes) : undefined;
                } catch {
                    return [];
                }
                if (fresh === undefined || fresh === document.getText()) {
                    return [];
                }
                const lastLine = Math.max(0, document.lineCount - 1);
                const end = document.lineAt(lastLine).range.end;
                return [vscode.TextEdit.replace(new vscode.Range(new vscode.Position(0, 0), end), fresh)];
            } catch {
                return [];
            }
        }
    };
}
