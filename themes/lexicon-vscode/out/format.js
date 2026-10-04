"use strict";
// Desktop-only formatting support: an .editorconfig parser/resolver and a
// conservative built-in Lexicon formatter. Uses Node's fs/path, so it is
// imported ONLY from extension.ts (never from providers.ts / extension.web.ts,
// which must stay web-safe).
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.resolveEditorConfig = resolveEditorConfig;
exports.indentUnit = indentUnit;
exports.formatLexiconText = formatLexiconText;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
// ---------------------------------------------------------------------------
// .editorconfig glob -> RegExp
// ---------------------------------------------------------------------------
function globToRegExp(pattern) {
    let re = '';
    let i = 0;
    const n = pattern.length;
    while (i < n) {
        const c = pattern[i];
        if (c === '*') {
            if (pattern[i + 1] === '*') {
                // `**` — crosses path separators; `**/` optionally matches nothing
                if (pattern[i + 2] === '/') {
                    re += '(?:.*/)?';
                    i += 3;
                    continue;
                }
                re += '.*';
                i += 2;
                continue;
            }
            re += '[^/]*';
            i += 1;
            continue;
        }
        if (c === '?') {
            re += '[^/]';
            i += 1;
            continue;
        }
        if (c === '[') {
            let j = i + 1;
            let neg = false;
            if (pattern[j] === '!' || pattern[j] === '^') {
                neg = true;
                j += 1;
            }
            let cls = '';
            while (j < n && pattern[j] !== ']') {
                if (pattern[j] === '\\' && j + 1 < n) {
                    cls += '\\' + pattern[j + 1];
                    j += 2;
                    continue;
                }
                cls += pattern[j] === '\\' ? '\\\\' : pattern[j];
                j += 1;
            }
            re += '[' + (neg ? '^' : '') + cls + ']';
            i = j + 1;
            continue;
        }
        if (c === '{') {
            let j = i + 1;
            const alts = [];
            let cur = '';
            let depth = 0;
            while (j < n) {
                const ch = pattern[j];
                if (ch === '{') {
                    depth += 1;
                    cur += ch;
                }
                else if (ch === '}') {
                    if (depth === 0) {
                        alts.push(cur);
                        j += 1;
                        break;
                    }
                    depth -= 1;
                    cur += ch;
                }
                else if (ch === ',' && depth === 0) {
                    alts.push(cur);
                    cur = '';
                }
                else {
                    cur += ch;
                }
                j += 1;
            }
            re += '(?:' + alts.map(a => globToRegExp(a).source).join('|') + ')';
            i = j;
            continue;
        }
        // escape regex metacharacters
        re += c.replace(/[.+^$()|\\\]/]/g, '\\$&');
        i += 1;
    }
    return new RegExp('^' + re + '$');
}
function matchesGlob(relPath, pattern) {
    // A pattern without '/' matches the basename at any depth.
    const p = pattern.includes('/') ? pattern.replace(/^\//, '') : '**/' + pattern;
    try {
        return globToRegExp(p).test(relPath);
    }
    catch {
        return false;
    }
}
// ---------------------------------------------------------------------------
// parse a single .editorconfig
// ---------------------------------------------------------------------------
function parseBool(v) {
    const s = v.toLowerCase();
    if (s === 'true') {
        return true;
    }
    if (s === 'false') {
        return false;
    }
    return undefined;
}
function parseEditorConfigFile(file) {
    let text;
    try {
        text = fs.readFileSync(file, 'utf8');
    }
    catch {
        return null;
    }
    const dir = path.dirname(file);
    const sections = [];
    let current = null;
    let root = false;
    for (const rawLine of text.split(/\r?\n/)) {
        const line = rawLine.trim();
        if (line === '' || line[0] === '#' || line[0] === ';') {
            continue;
        }
        if (line[0] === '[' && line[line.length - 1] === ']') {
            current = { globs: line.slice(1, -1).split(/\s*,\s*/).filter(Boolean), props: {} };
            sections.push(current);
            continue;
        }
        const eq = line.indexOf('=');
        const colon = line.indexOf(':');
        let sep = eq;
        if (colon >= 0 && (eq < 0 || colon < eq)) {
            sep = colon;
        }
        if (sep < 0) {
            continue;
        }
        const key = line.slice(0, sep).trim().toLowerCase();
        const val = line.slice(sep + 1).trim();
        if (key === 'root') {
            root = parseBool(val) === true;
            continue;
        }
        if (!current) {
            continue;
        } // properties before any section header are ignored
        const p = current.props;
        if (key === 'indent_style') {
            if (val === 'tab' || val === 'space') {
                p.indent_style = val;
            }
        }
        else if (key === 'indent_size') {
            p.indent_size = val === 'tab' ? 'tab' : (parseInt(val, 10) || undefined);
        }
        else if (key === 'tab_width') {
            p.tab_width = parseInt(val, 10) || undefined;
        }
        else if (key === 'end_of_line') {
            if (val === 'lf' || val === 'crlf' || val === 'cr') {
                p.end_of_line = val;
            }
        }
        else if (key === 'charset') {
            p.charset = val.toLowerCase();
        }
        else if (key === 'trim_trailing_whitespace') {
            const b = parseBool(val);
            if (b !== undefined) {
                p.trim_trailing_whitespace = b;
            }
        }
        else if (key === 'insert_final_newline') {
            const b = parseBool(val);
            if (b !== undefined) {
                p.insert_final_newline = b;
            }
        }
        else if (key === 'max_line_length') {
            p.max_line_length = val.toLowerCase() === 'off' ? 'off' : (parseInt(val, 10) || undefined);
        }
    }
    return { root, dir, sections };
}
/**
 * Resolve the effective EditorConfig for a file, per the EditorConfig spec:
 * walk up from the file's directory collecting `.editorconfig` files until one
 * declares `root = true`, then apply top-most first so nearer files win.
 */
function resolveEditorConfig(filePath) {
    let abs;
    try {
        abs = path.resolve(filePath);
    }
    catch {
        return {};
    }
    const chain = [];
    let dir = path.dirname(abs);
    // Walk upward, collecting files; stop after the first root=true (inclusive).
    for (let guard = 0; guard < 64; guard += 1) {
        const ec = path.join(dir, '.editorconfig');
        if (fs.existsSync(ec)) {
            const parsed = parseEditorConfigFile(ec);
            if (parsed) {
                chain.push(parsed);
                if (parsed.root) {
                    break;
                }
            }
        }
        const parent = path.dirname(dir);
        if (parent === dir) {
            break;
        }
        dir = parent;
    }
    // Apply farthest (root-most) first so nearer configs override.
    chain.reverse();
    const out = {};
    for (const file of chain) {
        let rel;
        try {
            rel = path.relative(file.dir, abs).split(path.sep).join('/');
        }
        catch {
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
function indentUnit(cfg) {
    const style = cfg.indent_style || 'space';
    let size;
    if (cfg.indent_size === 'tab' || cfg.indent_size === undefined) {
        size = cfg.tab_width || 4;
    }
    else {
        size = cfg.indent_size;
    }
    if (style === 'tab') {
        return '\t';
    }
    return ' '.repeat(Math.max(1, size));
}
/**
 * Scan one physical line, tracking block-comment and string state, and return
 * the net `{`/`}` delta contributed by real code plus the first code char.
 * Braces inside strings (incl. `{interp}`) and comments are ignored.
 */
function scanLine(line, inBlockComment) {
    let delta = 0;
    let firstCodeChar = '';
    let inStr = null;
    let i = 0;
    const startComment = inBlockComment;
    while (i < line.length) {
        const c = line[i];
        if (inBlockComment) {
            if (c === '*' && line[i + 1] === '/') {
                inBlockComment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if (inStr) {
            if (c === '\\') {
                i += 2;
                continue;
            }
            if (c === inStr) {
                inStr = null;
            }
            i += 1;
            continue;
        }
        if (c === '/' && line[i + 1] === '/') {
            break;
        } // line comment: rest is not code
        if (c === '/' && line[i + 1] === '*') {
            inBlockComment = true;
            i += 2;
            continue;
        }
        if (c === '"' || c === "'") {
            inStr = c;
            i += 1;
            continue;
        }
        if (c === '{') {
            delta += 1;
            if (!firstCodeChar) {
                firstCodeChar = '{';
            }
            i += 1;
            continue;
        }
        if (c === '}') {
            delta -= 1;
            if (!firstCodeChar) {
                firstCodeChar = '}';
            }
            i += 1;
            continue;
        }
        if (!firstCodeChar && !/\s/.test(c)) {
            firstCodeChar = c;
        }
        i += 1;
    }
    const isCodeLine = firstCodeChar !== '' || delta !== 0;
    return { delta, firstCodeChar, inBlockCommentAtStart: startComment, inBlockCommentAtEnd: inBlockComment, isCodeLine };
}
/**
 * Format Lexicon source text. Conservative and idempotent:
 *  - normalizes line endings
 *  - re-indents code lines to brace depth (skipping block comments/strings)
 *  - trims trailing whitespace
 *  - collapses trailing blank lines to a single final newline
 * Never rewrites content inside strings or comments (except trailing spaces).
 */
function formatLexiconText(input, cfg, opts = {}) {
    const reindent = opts.reindent !== false;
    const trim = opts.trimTrailingWhitespace !== undefined ? opts.trimTrailingWhitespace : (cfg.trim_trailing_whitespace !== false);
    const finalNewline = opts.insertFinalNewline !== undefined ? opts.insertFinalNewline : (cfg.insert_final_newline !== false);
    const eol = opts.eol || '\n';
    const unit = opts.indent || indentUnit(cfg);
    // Normalize EOL to \n for processing.
    const normalized = input.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
    const lines = normalized.split('\n');
    const out = [];
    let depth = 0;
    let inBlockComment = false;
    for (const raw of lines) {
        const scan = scanLine(raw, inBlockComment);
        inBlockComment = scan.inBlockCommentAtEnd;
        let line = raw;
        if (trim) {
            line = line.replace(/[ \t]+$/g, '');
        }
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
    while (out.length > 0 && out[out.length - 1] === '') {
        out.pop();
    }
    let result = out.join(eol);
    if (finalNewline) {
        result += eol;
    }
    return result;
}
