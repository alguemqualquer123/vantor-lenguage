// Desktop-only formatting support: an .editorconfig parser/resolver and a
// conservative built-in Lexicon formatter. Uses Node's fs/path, so it is
// imported ONLY from extension.ts (never from providers.ts / extension.web.ts,
// which must stay web-safe).

import * as fs from 'fs';
import * as path from 'path';

export interface EditorConfig {
    indent_style?: 'tab' | 'space';
    indent_size?: number | 'tab';
    tab_width?: number;
    end_of_line?: 'lf' | 'crlf' | 'cr';
    charset?: string;
    trim_trailing_whitespace?: boolean;
    insert_final_newline?: boolean;
    max_line_length?: number | 'off';
}

interface Section { globs: string[]; props: EditorConfig; }
interface ParsedFile { root: boolean; dir: string; sections: Section[]; }

// ---------------------------------------------------------------------------
// .editorconfig glob -> RegExp
// ---------------------------------------------------------------------------
function globToRegExp(pattern: string): RegExp {
    let re = '';
    let i = 0;
    const n = pattern.length;
    while (i < n) {
        const c = pattern[i];
        if (c === '*') {
            if (pattern[i + 1] === '*') {
                // `**` — crosses path separators; `**/` optionally matches nothing
                if (pattern[i + 2] === '/') { re += '(?:.*/)?'; i += 3; continue; }
                re += '.*'; i += 2; continue;
            }
            re += '[^/]*'; i += 1; continue;
        }
        if (c === '?') { re += '[^/]'; i += 1; continue; }
        if (c === '[') {
            let j = i + 1;
            let neg = false;
            if (pattern[j] === '!' || pattern[j] === '^') { neg = true; j += 1; }
            let cls = '';
            while (j < n && pattern[j] !== ']') {
                if (pattern[j] === '\\' && j + 1 < n) { cls += '\\' + pattern[j + 1]; j += 2; continue; }
                cls += pattern[j] === '\\' ? '\\\\' : pattern[j];
                j += 1;
            }
            re += '[' + (neg ? '^' : '') + cls + ']';
            i = j + 1; continue;
        }
        if (c === '{') {
            let j = i + 1;
            const alts: string[] = [];
            let cur = '';
            let depth = 0;
            while (j < n) {
                const ch = pattern[j];
                if (ch === '{') { depth += 1; cur += ch; }
                else if (ch === '}') { if (depth === 0) { alts.push(cur); j += 1; break; } depth -= 1; cur += ch; }
                else if (ch === ',' && depth === 0) { alts.push(cur); cur = ''; }
                else { cur += ch; }
                j += 1;
            }
            re += '(?:' + alts.map(a => globToRegExp(a).source).join('|') + ')';
            i = j; continue;
        }
        // escape regex metacharacters
        re += c.replace(/[.+^$()|\\\]/]/g, '\\$&');
        i += 1;
    }
    return new RegExp('^' + re + '$');
}

function matchesGlob(relPath: string, pattern: string): boolean {
    // A pattern without '/' matches the basename at any depth.
    const p = pattern.includes('/') ? pattern.replace(/^\//, '') : '**/' + pattern;
    try {
        return globToRegExp(p).test(relPath);
    } catch {
        return false;
    }
}

// ---------------------------------------------------------------------------
// parse a single .editorconfig
// ---------------------------------------------------------------------------
function parseBool(v: string): boolean | undefined {
    const s = v.toLowerCase();
    if (s === 'true') { return true; }
    if (s === 'false') { return false; }
    return undefined;
}

function parseEditorConfigFile(file: string): ParsedFile | null {
    let text: string;
    try {
        text = fs.readFileSync(file, 'utf8');
    } catch {
        return null;
    }
    const dir = path.dirname(file);
    const sections: Section[] = [];
    let current: Section | null = null;
    let root = false;
    for (const rawLine of text.split(/\r?\n/)) {
        const line = rawLine.trim();
        if (line === '' || line[0] === '#' || line[0] === ';') { continue; }
        if (line[0] === '[' && line[line.length - 1] === ']') {
            current = { globs: line.slice(1, -1).split(/\s*,\s*/).filter(Boolean), props: {} };
            sections.push(current);
            continue;
        }
        const eq = line.indexOf('=');
        const colon = line.indexOf(':');
        let sep = eq;
        if (colon >= 0 && (eq < 0 || colon < eq)) { sep = colon; }
        if (sep < 0) { continue; }
        const key = line.slice(0, sep).trim().toLowerCase();
        const val = line.slice(sep + 1).trim();
        if (key === 'root') { root = parseBool(val) === true; continue; }
        if (!current) { continue; } // properties before any section header are ignored
        const p = current.props;
        if (key === 'indent_style') { if (val === 'tab' || val === 'space') { p.indent_style = val; } }
        else if (key === 'indent_size') { p.indent_size = val === 'tab' ? 'tab' : (parseInt(val, 10) || undefined); }
        else if (key === 'tab_width') { p.tab_width = parseInt(val, 10) || undefined; }
        else if (key === 'end_of_line') { if (val === 'lf' || val === 'crlf' || val === 'cr') { p.end_of_line = val; } }
        else if (key === 'charset') { p.charset = val.toLowerCase(); }
        else if (key === 'trim_trailing_whitespace') { const b = parseBool(val); if (b !== undefined) { p.trim_trailing_whitespace = b; } }
        else if (key === 'insert_final_newline') { const b = parseBool(val); if (b !== undefined) { p.insert_final_newline = b; } }
        else if (key === 'max_line_length') { p.max_line_length = val.toLowerCase() === 'off' ? 'off' : (parseInt(val, 10) || undefined); }
    }
    return { root, dir, sections };
}

/**
 * Resolve the effective EditorConfig for a file, per the EditorConfig spec:
 * walk up from the file's directory collecting `.editorconfig` files until one
 * declares `root = true`, then apply top-most first so nearer files win.
 */
export function resolveEditorConfig(filePath: string): EditorConfig {
    let abs: string;
    try {
        abs = path.resolve(filePath);
    } catch {
        return {};
    }
    const chain: ParsedFile[] = [];
    let dir = path.dirname(abs);
    // Walk upward, collecting files; stop after the first root=true (inclusive).
    for (let guard = 0; guard < 64; guard += 1) {
        const ec = path.join(dir, '.editorconfig');
        if (fs.existsSync(ec)) {
            const parsed = parseEditorConfigFile(ec);
            if (parsed) {
                chain.push(parsed);
                if (parsed.root) { break; }
            }
        }
        const parent = path.dirname(dir);
        if (parent === dir) { break; }
        dir = parent;
    }
    // Apply farthest (root-most) first so nearer configs override.
    chain.reverse();
    const out: EditorConfig = {};
    for (const file of chain) {
        let rel: string;
        try {
            rel = path.relative(file.dir, abs).split(path.sep).join('/');
        } catch {
            continue;
        }
        for (const sec of file.sections) {
            if (sec.globs.some(g => matchesGlob(rel, g))) {
                Object.assign(out, sec.props);
            }
        }
    }
    return out;
}

/** Concrete indent unit (string) from a resolved config. */
export function indentUnit(cfg: EditorConfig): string {
    const style = cfg.indent_style || 'space';
    let size: number;
    if (cfg.indent_size === 'tab' || cfg.indent_size === undefined) {
        size = cfg.tab_width || 4;
    } else {
        size = cfg.indent_size;
    }
    if (style === 'tab') { return '\t'; }
    return ' '.repeat(Math.max(1, size));
}

// ---------------------------------------------------------------------------
// built-in Lexicon formatter (conservative, brace-depth aware)
// ---------------------------------------------------------------------------

interface ScanLine {
    // net brace delta contributed by CODE on this line (strings/comments skipped)
    delta: number;
    // first significant code char on the line ('}' etc.) or '' if blank/comment-only
    firstCodeChar: string;
    inBlockCommentAtStart: boolean;
    inBlockCommentAtEnd: boolean;
    isCodeLine: boolean;
}

/**
 * Scan one physical line, tracking block-comment and string state, and return
 * the net `{`/`}` delta contributed by real code plus the first code char.
 * Braces inside strings (incl. `{interp}`) and comments are ignored.
 */
function scanLine(line: string, inBlockComment: boolean): ScanLine {
    let delta = 0;
    let firstCodeChar = '';
    let inStr: '"' | "'" | null = null;
    let i = 0;
    const startComment = inBlockComment;
    while (i < line.length) {
        const c = line[i];
        if (inBlockComment) {
            if (c === '*' && line[i + 1] === '/') { inBlockComment = false; i += 2; continue; }
            i += 1; continue;
        }
        if (inStr) {
            if (c === '\\') { i += 2; continue; }
            if (c === inStr) { inStr = null; }
            i += 1; continue;
        }
        if (c === '/' && line[i + 1] === '/') { break; }           // line comment: rest is not code
        if (c === '/' && line[i + 1] === '*') { inBlockComment = true; i += 2; continue; }
        if (c === '"' || c === "'") { inStr = c as '"' | "'"; i += 1; continue; }
        if (c === '{') { delta += 1; if (!firstCodeChar) { firstCodeChar = '{'; } i += 1; continue; }
        if (c === '}') { delta -= 1; if (!firstCodeChar) { firstCodeChar = '}'; } i += 1; continue; }
        if (!firstCodeChar && !/\s/.test(c)) { firstCodeChar = c; }
        i += 1;
    }
    const isCodeLine = firstCodeChar !== '' || delta !== 0;
    return { delta, firstCodeChar, inBlockCommentAtStart: startComment, inBlockCommentAtEnd: inBlockComment, isCodeLine };
}

export interface FormatOptions {
    reindent?: boolean;               // default true
    trimTrailingWhitespace?: boolean; // default from cfg, fallback true
    insertFinalNewline?: boolean;     // default from cfg, fallback true
    eol?: '\n' | '\r\n';              // default '\n'
    indent?: string;                  // default from cfg
    maxLineLength?: number | 'off';   // advisory only (not enforced by reflow)
}

/**
 * Format Lexicon source text. Conservative and idempotent:
 *  - normalizes line endings
 *  - re-indents code lines to brace depth (skipping block comments/strings)
 *  - trims trailing whitespace
 *  - collapses trailing blank lines to a single final newline
 * Never rewrites content inside strings or comments (except trailing spaces).
 */
export function formatLexiconText(input: string, cfg: EditorConfig, opts: FormatOptions = {}): string {
    const reindent = opts.reindent !== false;
    const trim = opts.trimTrailingWhitespace !== undefined ? opts.trimTrailingWhitespace : (cfg.trim_trailing_whitespace !== false);
    const finalNewline = opts.insertFinalNewline !== undefined ? opts.insertFinalNewline : (cfg.insert_final_newline !== false);
    const eol = opts.eol || '\n';
    const unit = opts.indent || indentUnit(cfg);

    // Normalize EOL to \n for processing.
    const normalized = input.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
    const lines = normalized.split('\n');

    const out: string[] = [];
    let depth = 0;
    let inBlockComment = false;
    for (const raw of lines) {
        const scan = scanLine(raw, inBlockComment);
        inBlockComment = scan.inBlockCommentAtEnd;

        let line = raw;
        if (trim) { line = line.replace(/[ \t]+$/g, ''); }

        if (line.trim() === '') {
            out.push('');
            continue;
        }
        if (!reindent || scan.inBlockCommentAtStart || !scan.isCodeLine) {
            // Inside a block comment, or a comment-only/blank line: keep the
            // author's relative indentation but normalize tabs->unit if leading.
            out.push(line);
            // still update depth from any code on comment-only lines (none) — no-op
            depth = Math.max(0, depth + scan.delta);
            continue;
        }
        // Code line: a leading closer dedents this line.
        let lineDepth = depth;
        if (scan.firstCodeChar === '}' || scan.firstCodeChar === ')' || scan.firstCodeChar === ']') {
            lineDepth = Math.max(0, depth - 1);
        }
        const content = line.replace(/^[ \t]+/, '');
        out.push(unit.repeat(lineDepth) + content);
        depth = Math.max(0, depth + scan.delta);
    }

    // Collapse trailing blank lines; ensure exactly one final newline if requested.
    while (out.length > 0 && out[out.length - 1] === '') { out.pop(); }
    let result = out.join(eol);
    if (finalNewline) { result += eol; }
    return result;
}
