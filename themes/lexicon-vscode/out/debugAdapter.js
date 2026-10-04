"use strict";
// Lexicon DAP v1 — honest source-level debug adapter over stdio.
// No dependencies: hand-rolled Content-Length framing on process stdin/stdout.
//
// What this adapter REALLY does (capabilities reflect this):
//   initialize  -> { supportsConfigurationDoneRequest: true, no step/pause
//                    support, exceptionBreakpointFilters: [] }
//   launch      -> { program: <file.lex> }: reads + parses the file, then
//                    announces `initialized`.
//   setBreakpoints -> verifies per line: breakable = in range, non-blank,
//                    non-comment (`//`). Anything else is unverified.
//   threads     -> single thread id 1 ("main").
//   stackTrace  -> static frames from `fn` symbols parsed from the file
//                    (no live call stack — the `lex` runtime exposes none yet).
//   scopes      -> Locals (top-level `let name = <literal>` bindings) +
//                    Globals (parsed `fn` signatures).
//   variables   -> literal values with inferred types; fns listed by signature.
//   evaluate    -> resolves names bound to literals, nothing else.
//   continue    -> acknowledged; real execution goes through `lex run` via the
//                    CodeLens/terminal because an adapter cannot open terminals.
//   disconnect  -> exits.
//   step (next/stepIn/stepOut/pause) -> explicit "not supported" error.
//
// ROADMAP: live pause/step/locals require runtime support in `lex debug`
// (a pause/step protocol over this same channel). Until then this adapter is
// source-level navigation, and says so.
// `require` comes from @types/node; no local declare needed.
const fs = require('fs');
const Buf = require('buffer').Buffer;
let seq = 0;
let pending = '';
let programPath = '';
let programLines = [];
let fnSymbols = [];
let letBindings = [];
let bpId = 1;
function send(msg) {
    msg.seq = ++seq;
    msg.type = msg.type || 'response';
    const body = JSON.stringify(msg);
    const len = Buf.byteLength(body, 'utf8');
    process.stdout.write('Content-Length: ' + len + '\r\n\r\n' + body);
}
function respond(requestSeq, command, success, body, message) {
    const msg = { seq: 0, type: 'response', request_seq: requestSeq, success, command };
    if (body !== undefined) {
        msg.body = body;
    }
    if (message !== undefined) {
        msg.message = message;
    }
    send(msg);
}
function event(name, body) {
    const msg = { seq: 0, type: 'event', event: name };
    if (body !== undefined) {
        msg.body = body;
    }
    send(msg);
}
function inferType(lit) {
    const t = lit.trim();
    if (/^(true|false)$/.test(t)) {
        return 'bool';
    }
    if (/^[0-9][0-9_]*$/.test(t)) {
        return 'int';
    }
    if (/^[0-9][0-9_]*\.[0-9][0-9_]*$/.test(t)) {
        return 'f64';
    }
    if (/^".*"$/.test(t) || /^'.*'$/.test(t)) {
        return 'String';
    }
    return 'unknown';
}
function baseName(p) {
    return p.split(/[\\/]/).pop() || p;
}
function parseProgram(path) {
    programPath = path;
    programLines = [];
    fnSymbols = [];
    letBindings = [];
    try {
        const text = fs.readFileSync(path, 'utf8');
        programLines = text.split('\n');
    }
    catch {
        return;
    }
    for (let i = 0; i < programLines.length; i++) {
        const ln = programLines[i];
        const fnm = /(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*(->\s*[A-Za-z0-9_?<>]+)?/.exec(ln);
        if (fnm) {
            fnSymbols.push({ name: fnm[1], line: i + 1, detail: `fn ${fnm[1]}(${fnm[2] || ''})${fnm[3] ? ' ' + fnm[3] : ''}` });
        }
        const letm = /^\s*(?:let|const|var)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+?);\s*$/.exec(ln);
        if (letm && inferType(letm[2]) !== 'unknown') {
            letBindings.push({ name: letm[1], value: letm[2].trim(), type: inferType(letm[2]), line: i + 1 });
        }
    }
}
function isBreakable(line1Based) {
    if (line1Based < 1 || line1Based > programLines.length) {
        return false;
    }
    const t = programLines[line1Based - 1].trim();
    return t.length > 0 && !t.startsWith('//');
}
function sourceRef() {
    return { name: baseName(programPath) || 'main.lex', path: programPath };
}
function handle(req) {
    const args = req.arguments || {};
    switch (req.command) {
        case 'initialize':
            respond(req.seq, 'initialize', true, {
                supportsConfigurationDoneRequest: true,
                supportsEvaluateForHovers: true,
                exceptionBreakpointFilters: [],
            });
            break;
        case 'launch':
            parseProgram(typeof args.program === 'string' ? args.program : '');
            respond(req.seq, 'launch', true);
            event('initialized');
            break;
        case 'configurationDone':
            respond(req.seq, 'configurationDone', true);
            break;
        case 'setBreakpoints': {
            const lines = Array.isArray(args.breakpoints)
                ? args.breakpoints.map((b) => b.line)
                : Array.isArray(args.lines)
                    ? args.lines
                    : [];
            respond(req.seq, 'setBreakpoints', true, {
                breakpoints: lines.map((l) => ({
                    id: bpId++,
                    verified: isBreakable(l),
                    line: l,
                    source: sourceRef(),
                })),
            });
            break;
        }
        case 'threads':
            respond(req.seq, 'threads', true, { threads: [{ id: 1, name: 'main' }] });
            break;
        case 'stackTrace': {
            const frames = fnSymbols.length > 0
                ? fnSymbols.map((f, i) => ({ id: i + 1, name: f.name, line: f.line, column: 1, source: sourceRef() }))
                : [{ id: 1, name: 'main', line: 1, column: 1, source: sourceRef() }];
            const start = typeof args.startFrame === 'number' ? args.startFrame : 0;
            const count = typeof args.levels === 'number' ? args.levels : frames.length;
            respond(req.seq, 'stackTrace', true, {
                stackFrames: frames.slice(start, start + count),
                totalFrames: frames.length,
            });
            break;
        }
        case 'scopes':
            respond(req.seq, 'scopes', true, {
                scopes: [
                    { name: 'Locals', variablesReference: 1, expensive: false },
                    { name: 'Globals', variablesReference: 2, expensive: false },
                ],
            });
            break;
        case 'variables': {
            if (args.variablesReference === 1) {
                respond(req.seq, 'variables', true, {
                    variables: letBindings.map(b => ({
                        name: b.name, value: b.value, type: b.type,
                        variablesReference: 0, namedVariables: 0, indexedVariables: 0,
                    })),
                });
            }
            else if (args.variablesReference === 2) {
                respond(req.seq, 'variables', true, {
                    variables: fnSymbols.map(f => ({
                        name: f.name, value: f.detail, type: 'fn',
                        variablesReference: 0, namedVariables: 0, indexedVariables: 0,
                    })),
                });
            }
            else {
                respond(req.seq, 'variables', true, { variables: [] });
            }
            break;
        }
        case 'evaluate': {
            const expr = typeof args.expression === 'string' ? args.expression.trim() : '';
            const hit = letBindings.find(b => b.name === expr);
            if (hit) {
                respond(req.seq, 'evaluate', true, { result: hit.value, type: hit.type, variablesReference: 0 });
            }
            else {
                respond(req.seq, 'evaluate', false, undefined, `cannot evaluate '${expr}': only literal let-bindings are visible (no live runtime yet)`);
            }
            break;
        }
        case 'continue':
            // No live runtime to resume; acknowledge so the client doesn't hang.
            respond(req.seq, 'continue', true, { allThreadsContinued: true });
            break;
        case 'disconnect':
        case 'terminate':
            respond(req.seq, req.command, true);
            process.exit(0);
            break;
        case 'next':
        case 'stepIn':
        case 'stepOut':
        case 'pause':
            respond(req.seq, req.command, false, undefined, 'not supported: the lex runtime has no pause/step protocol yet (roadmap)');
            break;
        default:
            respond(req.seq, req.command, false, undefined, `unsupported command: ${req.command}`);
            break;
    }
}
function pump() {
    for (;;) {
        const headEnd = pending.indexOf('\r\n\r\n');
        if (headEnd < 0) {
            return;
        }
        const head = pending.slice(0, headEnd);
        const hm = /Content-Length:\s*(\d+)/i.exec(head);
        if (!hm) {
            return;
        }
        const len = parseInt(hm[1], 10);
        const start = headEnd + 4;
        if (pending.length < start + len) {
            return;
        }
        const body = pending.slice(start, start + len);
        pending = pending.slice(start + len);
        try {
            const msg = JSON.parse(body);
            if (msg && msg.type === 'request') {
                handle(msg);
            }
        }
        catch {
            // ignore malformed frames; keep the stream alive
        }
    }
}
process.stdin.on('data', (chunk) => {
    pending += chunk.toString('utf8');
    pump();
});
process.stdin.on('end', () => process.exit(0));
process.stdin.resume();
