import * as vscode from 'vscode';
import * as cp from 'child_process';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import * as providers from './providers';
import * as Telemetry from './telemetry';
import { runNewProjectFlow } from './newProject';
import { precheckLexicon } from './providers';

const LANG_ID = 'lexicon';

// --- lex binary resolution / terminal runner (stays in the desktop entry) ---

function lexBin(): string {
    const cfg = vscode.workspace.getConfiguration('lexicon');
    const configured = ((cfg.get('path') as string | undefined) || '').trim();
    // Explicit `lexicon.path` always wins.
    if (configured.length > 0 && configured !== 'lex' && configured !== 'lex.exe') {
        return configured;
    }
    // Otherwise prefer the per-user install (`lex install` / first-run
    // auto-install puts it there). This matters on macOS/Linux where GUI-
    // launched VS Code never inherits the shell PATH, so bare `lex`
    // would fail with ENOENT even though the toolchain is installed.
    const exe = process.platform === 'win32' ? 'lex.exe' : 'lex';
    try {
        const homeLex = path.join(os.homedir(), '.lexicon', 'bin', exe);
        if (fs.existsSync(homeLex)) {
            return homeLex;
        }
    } catch {
        // homedir/stat failures fall through to PATH lookup
    }
    return exe;
}

function lexArgs(extra: string[]): string[] {
    const cfg = vscode.workspace.getConfiguration('lexicon');
    const features = (((cfg.get('features') as string | undefined) || '').trim());
    const target = (((cfg.get('target') as string | undefined) || '').trim());
    const out = [...extra];
    if (features.length > 0 && (extra[0] === 'build' || extra[0] === 'check' || extra[0] === 'vet')) {
        out.push('--features', features);
    }
    if (target.length > 0 && extra[0] === 'build') {
        out.push('--target', target);
    }
    return out;
}

function channel(): vscode.OutputChannel {
    return LexState.output;
}

function runInTerminal(title: string, args: string[], cwd?: string): void {
    const term = vscode.window.terminals.find((t: any) => t.name === title)
        || vscode.window.createTerminal({ name: title, cwd });
    term.show();
    const quoted = [lexBin(), ...lexArgs(args)].map(a => (a.includes(' ') ? `"${a}"` : a)).join(' ');
    channel().appendLine(`$ ${quoted}`);
    term.sendText(quoted);
}

function activeFile(): vscode.Uri | undefined {
    return vscode.window.activeTextEditor?.document.uri;
}

/** Generic `lex <args...>` runner (used by vet + the formatting provider). */
function runLex(args: string[]): Promise<{ code: number; out: string }> {
    return new Promise((resolve) => {
        const proc = cp.execFile(lexBin(), lexArgs(args), { timeout: 60000 }, (err: any, stdout: any, stderr: any) => {
            const out = `${stdout}\n${stderr}`;
            resolve({ code: err ? (err as { code?: number }).code ?? 1 : 0, out });
        });
        proc.on('error', () => resolve({ code: 127, out: `Cannot execute ${lexBin()}: is it on PATH? (setting: lexicon.path)` }));
    });
}

function runVet(file: string): Promise<{ code: number; out: string }> {
    return runLex(['vet', file]);
}

// --- SDK go-to-definition (Ctrl+Click on `Module::member`) ---
//
// Jumps into the SDK the toolchain really runs from: real `lib/std`
// sources for `std::` packages, `native/*.lex` signature stubs for
// native builtins. Falls back to the local (same-file) definition
// provider registered separately when nothing matches here.

const SDK_PATHS: Record<string, string> = {
    strings: 'std/strings', strconv: 'std/strconv', math: 'std/math',
    sort: 'std/sort', slices: 'std/slices', errors: 'std/errors',
    path: 'std/path', fmt: 'std/fmt', time: 'std/time',
    bytes: 'std/bytes', io: 'std/io', bufio: 'std/bufio',
    maps: 'std/maps', cmp: 'std/cmp', iter: 'std/iter',
    utf8: 'std/unicode/utf8', utf16: 'std/unicode/utf16',
    list: 'std/container/list', heap: 'std/container/heap',
    ring: 'std/container/ring', os: 'std/os', exec: 'std/os/exec',
    signal: 'std/os/signal', user: 'std/os/user',
    sync: 'std/sync', atomic: 'std/sync/atomic',
    context: 'std/context', log: 'std/log',
    filepath: 'std/path/filepath', json: 'std/encoding/json',
    base64: 'std/encoding/base64', base32: 'std/encoding/base32',
    hex: 'std/encoding/hex', csv: 'std/encoding/csv',
    pem: 'std/encoding/pem', ascii85: 'std/encoding/ascii85',
    html: 'std/html', regexp: 'std/regexp',
    fnv: 'std/hash/fnv', crc32: 'std/hash/crc32',
    crc64: 'std/hash/crc64', adler32: 'std/hash/adler32',
    maphash: 'std/hash/maphash', rand: 'std/rand',
    subtle: 'std/crypto/subtle', sha256: 'std/crypto/sha256',
    hmac: 'std/crypto/hmac', slog: 'std/slog', flag: 'std/flag',
    mime: 'std/mime', multipart: 'std/mime/multipart',
    url: 'std/net/url', mail: 'std/net/mail',
    textproto: 'std/net/textproto', bits: 'std/math/bits',
    unsafe: 'std/unsafe', runtime: 'std/runtime',
    testing: 'std/testing', color: 'std/image/color',
    tar: 'std/archive/tar',
    Console: 'std/native/console', Json: 'std/native/json',
    Env: 'std/native/env', Http: 'std/native/http',
    Time: 'std/native/time', File: 'std/native/file',
    Process: 'std/native/process', Text: 'std/native/text',
    Math: 'std/native/math', List: 'std/native/list',
    Hash: 'std/native/hash', Rand: 'std/native/rand',
    Atomic: 'std/native/atomic', Sys: 'std/native/sys',
};

// Cache so we only pay for a PATH lookup (`where`/`which`) once per session.
let _lexBinAbsCache: string | null | undefined;

/**
 * Resolve the absolute path of the `lex` executable, mirroring how the Go
 * toolchain finds GOROOT from the binary location. Order:
 *   1. explicit `lexicon.path` config (if it points at a real file)
 *   2. per-user install `~/.lexicon/bin/lex[.exe]`
 *   3. `~/.lexicon/sdk/bin/lex[.exe]`
 *   4. a PATH lookup via `where` (win) / `which` (posix)
 * Returns null when the toolchain cannot be located.
 */
function resolveLexBinaryAbs(): string | null {
    if (_lexBinAbsCache !== undefined) {
        return _lexBinAbsCache;
    }
    const exe = process.platform === 'win32' ? 'lex.exe' : 'lex';
    const isFile = (p: string): boolean => {
        try {
            return p.length > 0 && fs.existsSync(p) && fs.statSync(p).isFile();
        } catch {
            return false;
        }
    };
    let found: string | null = null;
    try {
        const cfg = vscode.workspace.getConfiguration('lexicon');
        const configured = ((cfg.get('path') as string | undefined) || '').trim();
        if (configured && isFile(configured)) {
            found = configured;
        }
    } catch {
        // config unavailable (web/older host): fall through
    }
    if (!found) {
        for (const cand of [
            path.join(os.homedir(), '.lexicon', 'bin', exe),
            path.join(os.homedir(), '.lexicon', 'sdk', 'bin', exe),
        ]) {
            if (isFile(cand)) {
                found = cand;
                break;
            }
        }
    }
    if (!found) {
        try {
            const lookup = process.platform === 'win32' ? 'where' : 'which';
            const res = cp.spawnSync(lookup, [process.platform === 'win32' ? 'lex' : exe], { encoding: 'utf8', timeout: 4000 });
            const first = (res.stdout || '').split(/\r?\n/).map((s: string) => s.trim()).find((s: string) => isFile(s));
            if (first) {
                found = first;
            }
        } catch {
            // PATH lookup is best-effort
        }
    }
    _lexBinAbsCache = found;
    return found;
}

function sdkRoots(): string[] {
    const roots: string[] = [];
    const push = (p: string): void => {
        if (p && !roots.includes(p)) {
            roots.push(p);
        }
    };
    // 1. Explicit user override (like GOROOT / go.goroot).
    try {
        const cfg = vscode.workspace.getConfiguration('lexicon');
        const sdkCfg = ((cfg.get('sdkPath') as string | undefined) || '').trim();
        if (sdkCfg) {
            push(path.join(sdkCfg, 'lib'));
            push(sdkCfg);
        }
    } catch {
        // config unavailable: skip
    }
    // 2. Env vars (LEXICON_HOME/SDK point at the SDK root; LEX_PATH at lib dirs).
    const home = process.env.LEXICON_HOME || process.env.LEX_HOME || process.env.LEXICON_ROOT || '';
    if (home) {
        push(path.join(home, 'lib'));
        push(home);
    }
    const sdkEnv = process.env.LEXICON_SDK || process.env.LEX_SDK || '';
    if (sdkEnv) {
        push(path.join(sdkEnv, 'lib'));
        push(sdkEnv);
    }
    // 3. Derive from the lex binary location (Go-style GOROOT discovery).
    const bin = resolveLexBinaryAbs();
    if (bin) {
        const binDir = path.dirname(bin);              // .../bin
        const parent = path.dirname(binDir);           // .../ (sdk or install root)
        push(path.join(parent, 'lib'));                // <root>/lib
        push(path.join(parent, 'sdk', 'lib'));         // <root>/sdk/lib
        push(path.join(binDir, 'lib'));                // <bin>/lib
        push(path.join(parent, 'share', 'lexicon', 'lib'));
    }
    // 4. Per-user install (default `lex install` target).
    try {
        push(path.join(os.homedir(), '.lexicon', 'sdk', 'lib'));
        push(path.join(os.homedir(), '.lexicon', 'lib'));
    } catch {
        // homedir unavailable: skip
    }
    // 5. Common machine-wide install locations (like C:\Program Files\Go).
    if (process.platform === 'win32') {
        const pf = process.env.ProgramFiles || 'C:\\Program Files';
        const pfx86 = process.env['ProgramFiles(x86)'] || 'C:\\Program Files (x86)';
        const local = process.env.LOCALAPPDATA || '';
        for (const base of [pf, pfx86]) {
            for (const name of ['Lexicon', 'lexicon', 'Lex']) {
                push(path.join(base, name, 'lib'));
                push(path.join(base, name, 'sdk', 'lib'));
            }
        }
        if (local) {
            push(path.join(local, 'Programs', 'Lexicon', 'lib'));
            push(path.join(local, 'Programs', 'Lexicon', 'sdk', 'lib'));
        }
    } else {
        for (const base of ['/usr/local', '/usr', '/opt']) {
            for (const name of ['lexicon', 'lex']) {
                push(path.join(base, name, 'lib'));
                push(path.join(base, 'lib', name));
                push(path.join(base, 'share', name, 'lib'));
            }
        }
    }
    // 6. Workspace-relative (monorepo checkouts of the language itself).
    try {
        for (const ws of vscode.workspace.workspaceFolders || []) {
            push(path.join(ws.uri.fsPath, 'lib'));
            push(path.join(ws.uri.fsPath, 'build', 'sdk', 'lib'));
        }
    } catch {
        // workspace unavailable (web stubs): skip
    }
    // 7. Legacy LEX_PATH (delimiter-separated list of lib dirs).
    const lexPath = process.env.LEX_PATH || '';
    for (const p of lexPath.split(path.delimiter)) {
        if (p) {
            push(p);
        }
    }
    return roots;
}

function moduleWordAt(line: string, character: number): string | null {
    const isWord = (c: string) => /[A-Za-z0-9_:.@]/.test(c);
    let start = Math.min(character, line.length);
    while (start > 0 && isWord(line[start - 1])) {
        start -= 1;
    }
    let end = Math.min(character, line.length);
    while (end < line.length && isWord(line[end])) {
        end += 1;
    }
    const word = line.slice(start, end);
    return word.length > 0 ? word : null;
}

function splitModuleWord(word: string): { key: string; member: string } | null {
    const w = word.trim().replace(/^@/, '');
    const dc = w.indexOf('::');
    if (dc >= 0) {
        const head = w.slice(0, dc);
        const rest = w.slice(dc + 2);
        if (head === 'std') {
            const key = rest.split('::').pop() || '';
            return key ? { key, member: '' } : null;
        }
        if (!head || rest.includes('::')) {
            return null;
        }
        return { key: head, member: rest };
    }
    const dot = w.lastIndexOf('.');
    if (dot >= 0) {
        const head = w.slice(0, dot);
        const member = w.slice(dot + 1);
        if (!head || !member) {
            return null;
        }
        return { key: head, member };
    }
    return { key: w, member: '' };
}

function findMemberLine(src: string, member: string): number {
    if (!member) {
        return 0;
    }
    const prefixes = ['pub fn ', 'pub struct ', 'pub const ', 'const ', 'fn '];
    const lines = src.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const t = lines[i].trim();
        for (const p of prefixes) {
            if (t.startsWith(p)) {
                const name = t.slice(p.length).split(/[^A-Za-z0-9_]/)[0];
                if (name === member) {
                    return i;
                }
            }
        }
    }
    return 0;
}

function createSdkDefinitionProvider(): any {
    return {
        provideDefinition(document: any, position: any): any {
            try {
                const line = document.lineAt(position.line).text;
                const word = moduleWordAt(line, position.character);
                if (!word) {
                    return null;
                }
                const split = splitModuleWord(word);
                if (!split) {
                    return null;
                }
                const rel = SDK_PATHS[split.key];
                if (!rel) {
                    return null;
                }
                for (const root of sdkRoots()) {
                    const file = path.join(root, rel + '.lex');
                    if (!fs.existsSync(file)) {
                        continue;
                    }
                    const src = fs.readFileSync(file, 'utf8');
                    const ln = findMemberLine(src, split.member);
                    const uri = vscode.Uri.file(file);
                    const pos = new vscode.Position(ln, 0);
                    return new vscode.Location(uri, new vscode.Range(pos, pos));
                }
            } catch {
                // best-effort: fall through to the local provider
            }
            return null;
        },
    };
}

namespace LexState {
    export let output!: vscode.OutputChannel;
    export let diagnostics!: vscode.DiagnosticCollection;
    export let status!: vscode.StatusBarItem;
}

export function activate(context: vscode.ExtensionContext): void {
    LexState.output = vscode.window.createOutputChannel('Lexicon');
    LexState.diagnostics = vscode.languages.createDiagnosticCollection('lexicon');
    LexState.status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    LexState.status.text = '$(flame) Lexicon';
    LexState.status.tooltip = 'Lexicon toolchain status';
    LexState.status.command = 'lexicon.env';
    context.subscriptions.push(LexState.output, LexState.diagnostics, LexState.status);
    Telemetry.initTelemetry(LexState.output);

    const updateStatus = (): void => {
        const ed = vscode.window.activeTextEditor;
        if (ed && ed.document.languageId === LANG_ID) {
            LexState.status.show();
        } else {
            LexState.status.hide();
        }
    };
    updateStatus();
    vscode.window.onDidChangeActiveTextEditor(updateStatus, null, context.subscriptions);

    // --- pipe/arrow decorations (provider logic lives in providers.ts) ---
    // Wrapped: decoration failures must never abort activate() (which would
    // leave `lexicon.run` etc. unregistered -> "command not found").
    try {
        const deco = providers.createDecorationTypes();
        let activeEditor = vscode.window.activeTextEditor;
        const refreshDeco = (): void => {
            try {
                providers.updatePipeArrowDecorations(activeEditor, deco.pipe, deco.arrow);
            } catch {
                // ignore per-editor decoration errors
            }
        };
        if (activeEditor) {
            refreshDeco();
        }
        vscode.window.onDidChangeActiveTextEditor((editor: any) => {
            activeEditor = editor;
            if (editor) {
                refreshDeco();
            }
            updateStatus();
        }, null, context.subscriptions);
        vscode.workspace.onDidChangeTextDocument((event: any) => {
            if (activeEditor && event.document === activeEditor.document) {
                refreshDeco();
            }
        }, null, context.subscriptions);
    } catch {
        // decorations are best-effort only
    }

    // --- language providers (all factories in providers.ts) ---
    // Wrapped: any single provider throwing must not abort activate() and
    // orphan the `lexicon.*` commands. Each factory is best-effort.
    let formattingProvider: any;
    try {
        formattingProvider = providers.createFormattingProvider(runLex);
    } catch {
        formattingProvider = undefined;
    }
    try {
        const semanticLegend = providers.createSemanticTokensLegend();
        const registrations: any[] = [];
        const safe = (fn: () => any): void => {
            try {
                registrations.push(fn());
            } catch {
                // skip providers unsupported by this host
            }
        };
        safe(() => vscode.languages.registerHoverProvider(LANG_ID, providers.createHoverProvider()));
        safe(() => vscode.languages.registerCompletionItemProvider(LANG_ID, providers.createCompletionProvider(), '.', ':'));
        safe(() => vscode.languages.registerDefinitionProvider(LANG_ID, providers.createDefinitionProvider()));
        safe(() => vscode.languages.registerDefinitionProvider(LANG_ID, createSdkDefinitionProvider()));
        safe(() => vscode.languages.registerDocumentSymbolProvider(LANG_ID, providers.createDocumentSymbolProvider()));
        safe(() => vscode.languages.registerCodeLensProvider(LANG_ID, providers.createCodeLensProvider()));
        safe(() => vscode.languages.registerSignatureHelpProvider(LANG_ID, providers.createSignatureHelpProvider(), '(', ','));
        safe(() => vscode.languages.registerInlayHintsProvider(LANG_ID, providers.createInlayHintsProvider()));
        safe(() => vscode.languages.registerFoldingRangeProvider(LANG_ID, providers.createFoldingRangeProvider()));
        safe(() => vscode.languages.registerDocumentColorProvider(LANG_ID, providers.createDocumentColorProvider()));
        safe(() => vscode.languages.registerReferenceProvider(LANG_ID, providers.createReferenceProvider()));
        safe(() => vscode.languages.registerRenameProvider(LANG_ID, providers.createRenameProvider()));
        safe(() => vscode.languages.registerDocumentSemanticTokensProvider(LANG_ID, providers.createSemanticTokensProvider(), semanticLegend));
        safe(() => vscode.languages.registerCodeActionsProvider(LANG_ID, providers.createQuickFixProvider(), {
            providedCodeActionKinds: [vscode.CodeActionKind.QuickFix],
        }));
        if (formattingProvider) {
            safe(() => vscode.languages.registerDocumentFormattingEditProvider(LANG_ID, formattingProvider));
        }
        safe(() => vscode.languages.registerWorkspaceSymbolProvider(providers.createWorkspaceSymbolProvider()));
        safe(() => vscode.tasks.registerTaskProvider('lex', providers.createLexTaskProvider(lexBin)));
        for (const r of registrations) {
            context.subscriptions.push(r);
        }
    } catch {
        // providers are best-effort; commands below must still register
    }

    // --- test explorer (TestController id `lexiconTests`) ---
    // Discovery is pure (providers.discoverTests via workspace.findFiles);
    // running shells out to `lex test`: visible run in the terminal, and
    // pass/fail marking driven by the `lex test` process exit code.
    // Wrapped: `vscode.tests` is undefined on older hosts — that must not
    // abort activate() and orphan `lexicon.run`.
    try {
        const testController = (vscode.tests as any)?.createTestController?.('lexiconTests', 'Lexicon Tests');
        if (testController) {
            context.subscriptions.push(testController);
    const refreshTests = async (): Promise<void> => {
        try {
            const found = await providers.discoverTests();
            testController.items.replace(found.map((t: providers.DiscoveredTest) =>
                testController.createTestItem(`${t.uri.toString()}#${t.fnName}`, t.fnName, t.uri)
            ));
        } catch {
            // discovery must never break activation
        }
    };
    testController.refreshHandler = refreshTests;
    testController.createRunProfile('Run Tests', vscode.TestRunProfileKind.Run, async (request: any, _token: any) => {
        Telemetry.recordCommand('lexicon.test');
        const run = testController.createTestRun(request);
        const items: any[] = [];
        if (request && Array.isArray(request.include) && request.include.length > 0) {
            request.include.forEach((i: any) => items.push(i));
        } else {
            testController.items.forEach((i: any) => items.push(i));
        }
        for (const item of items) {
            try {
                run.started(item);
            } catch {
                // tolerate stub/host differences
            }
        }
        runInTerminal('Lexicon Test', ['test']);
        let code = 1;
        let out = '';
        try {
            const res = await runLex(['test']);
            code = res.code;
            out = res.out;
        } catch {
            code = 1;
        }
        try {
            run.appendOutput(out);
        } catch {
            // optional on older hosts
        }
        for (const item of items) {
            try {
                if (code === 0) {
                    run.passed(item);
                } else {
                    run.failed(item, new Error(out.split('\n').slice(0, 5).join('\n') || 'lex test failed'));
                }
            } catch {
                // ignore per-item errors
            }
        }
        try {
            run.end();
        } catch {
            // ignore
        }
    }, true);
            try {
                void refreshTests();
            } catch {
                // never break activation
            }
        }
    } catch {
        // test explorer is best-effort; commands below must still register
    }

    // --- debug: config provider + real `lex dap` adapter factory (item 16) ---
    try {
        context.subscriptions.push(
        vscode.debug.registerDebugAdapterDescriptorFactory('lexicon', {
            createDebugAdapterDescriptor(_session: any) {
                return new vscode.DebugAdapterExecutable(lexBin(), ['dap']);
            },
        }),
        vscode.debug.registerDebugConfigurationProvider('lexicon', {
            provideDebugConfigurations(_folder: any) {
                return [{ type: 'lexicon', request: 'launch', name: 'Lexicon: Launch file', program: '${file}' }];
            },
            resolveDebugConfiguration(_folder: any, config: any) {
                if (!config.type) {
                    config.type = 'lexicon';
                }
                if (!config.request) {
                    config.request = 'launch';
                }
                if (!config.name) {
                    config.name = 'Lexicon: Launch file';
                }
                if (!config.program) {
                    config.program = '${file}';
                }
                return config;
            },
        })
        );
    } catch {
        // debug setup is best-effort; commands below must still register
    }

    // --- diagnostics via lex vet (item 9: exact file:line:col ranges) ---
    const setVetStatus = (errors: number | undefined): void => {
        if (errors === undefined) {
            LexState.status.text = '$(flame) Lexicon';
            LexState.status.tooltip = 'Lexicon toolchain status';
            LexState.status.command = 'lexicon.env';
            return;
        }
        if (errors === 0) {
            LexState.status.text = '$(check) lex vet: limpo';
            LexState.status.tooltip = 'lex vet: no findings. Click for Problems.';
        } else {
            LexState.status.text = `$(error) lex: ${errors} erro(s)`;
            LexState.status.tooltip = 'lex vet findings. Click to open Problems.';
        }
        LexState.status.command = 'lexicon.showProblems';
    };
    const stripAnsi = (s: string): string =>
        s.replace(/\x1b\[[0-9;]*m/g, '');
    const refreshDiagnostics = async (document: vscode.TextDocument): Promise<void> => {
        if (document.languageId !== LANG_ID) {
            return;
        }
        const cfg = vscode.workspace.getConfiguration('lexicon');
        if (!cfg.get('vetOnSave', true)) {
            LexState.diagnostics.delete(document.uri);
            setVetStatus(undefined);
            return;
        }
        try {
            await document.save();
        } catch {
            return;
        }
        const { code, out } = await runVet(document.fileName);
        const found: vscode.Diagnostic[] = [];
        const norm = (s: string): string => s.replace(/\\/g, '/').toLowerCase();
        const thisFile = norm(document.fileName);
        const baseOf = (s: string): string => {
            const n = norm(s);
            const i = n.lastIndexOf('/');
            return i >= 0 ? n.slice(i + 1) : n;
        };
        const pushAt = (line1: number, col1: number, msg: string, ecode?: string): void => {
            const line = Math.max(0, Math.min(line1 - 1, document.lineCount - 1));
            let range: vscode.Range;
            try {
                const lineText = document.lineAt(line).text;
                const c = Math.max(0, Math.min(col1 - 1, lineText.length));
                const rest = lineText.slice(c);
                const w = /^[A-Za-z_][A-Za-z0-9_]*/.exec(rest);
                range = w
                    ? new vscode.Range(line, c, line, c + w[0].length)
                    : new vscode.Range(line, 0, line, lineText.length);
            } catch {
                range = new vscode.Range(line, 0, line, 0);
            }
            const diag = new vscode.Diagnostic(
                range,
                ecode ? `${ecode}: ${msg}` : msg,
                /^(E\d{4}|LEX-[A-Z]+-\d+)$/.test(ecode || '') || /error|fail/i.test(msg)
                    ? vscode.DiagnosticSeverity.Error
                    : vscode.DiagnosticSeverity.Warning
            );
            if (ecode) {
                diag.code = {
                    value: ecode,
                    target: vscode.Uri.parse(`https://github.com/lexicon-lang/lexicon#${ecode.toLowerCase()}`),
                };
            }
            diag.source = 'lex';
            found.push(diag);
        };
        // Client-side syntax precheck FIRST: clear errors with exact
        // line:col before the binary even runs (works offline, catches
        // unbalanced delimiters, unterminated literals/comments and
        // dotted imports with precise positions).
        for (const f of precheckLexicon(document.getText())) {
            const line = Math.max(0, Math.min(f.line, document.lineCount - 1));
            let range: vscode.Range;
            try {
                const lineText = document.lineAt(line).text;
                const c = Math.max(0, Math.min(f.col, lineText.length));
                const rest = lineText.slice(c);
                const w = /^[A-Za-z_][A-Za-z0-9_]*/.exec(rest);
                range = w
                    ? new vscode.Range(line, c, line, c + w[0].length)
                    : new vscode.Range(line, 0, line, lineText.length);
            } catch {
                range = new vscode.Range(line, 0, line, 0);
            }
            const diag = new vscode.Diagnostic(
                range,
                `${f.code}: ${f.message}`,
                f.severity === 'error' ? vscode.DiagnosticSeverity.Error : vscode.DiagnosticSeverity.Warning
            );
            diag.code = {
                value: f.code,
                target: vscode.Uri.parse(`https://github.com/lexicon-lang/lexicon#${f.code.toLowerCase()}`),
            };
            diag.source = 'lex (client)';
            found.push(diag);
        }
        for (const raw of stripAnsi(out).split(/\r?\n/)) {
            const ln = raw.trim();
            if (!ln) {
                continue;
            }
            // `path:line:col: msg` and `<input>:line:col: msg` (current file).
            let m = /^(.+?):(\d+):(\d+):?\s*(.*)$/.exec(ln);
            if (m) {
                const f = m[1];
                if (f === '<input>' || norm(f) === thisFile || baseOf(f) === baseOf(thisFile)) {
                    pushAt(parseInt(m[2], 10), parseInt(m[3], 10), m[4] || ln);
                }
                continue;
            }
            // `path:line [CODE] msg` (lint-style).
            m = /^(.+?):(\d+)\s+\[([A-Za-z]+-[A-Za-z]+-\d+|E\d{4})\]\s*(.*)$/.exec(ln);
            if (m) {
                const f = m[1];
                if (norm(f) === thisFile || baseOf(f) === baseOf(thisFile)) {
                    pushAt(parseInt(m[2], 10), 1, m[4] || ln, m[3]);
                }
                continue;
            }
            // Bare `[E0000] msg` (typeck via log): pin to line 1.
            m = /\[(E\d{4})\]\s*(.*)$/.exec(ln);
            if (m) {
                pushAt(1, 1, m[2] || ln, m[1]);
                continue;
            }
            if (/error|fail/i.test(ln) && !/no .*violation|passed/i.test(ln)) {
                pushAt(1, 1, ln);
            }
        }
        LexState.diagnostics.set(document.uri, found);
        setVetStatus(code === 0 && found.length === 0 ? 0 : found.length);
    };
    // on save: vet diagnostics + optional `lexicon.formatOnSave` formatting.
    vscode.workspace.onDidSaveTextDocument(async (document: any) => {
        await refreshDiagnostics(document);
        try {
            const cfg = vscode.workspace.getConfiguration('lexicon');
            if (cfg.get('formatOnSave', false) && document.languageId === LANG_ID && formattingProvider) {
                const edits = await formattingProvider.provideDocumentFormattingEdits(document, {}, undefined);
                if (edits && edits.length > 0) {
                    const wsEdit = new vscode.WorkspaceEdit();
                    for (const e of edits) {
                        wsEdit.replace(document.uri, e.range, e.newText);
                    }
                    await vscode.workspace.applyEdit(wsEdit);
                }
            }
        } catch {
            // formatting on save must never break the save flow
        }
    }, null, context.subscriptions);

    // --- commands (all invocations counted by local-only telemetry) ---
    const fileOf = (uri?: vscode.Uri): string | undefined =>
        uri?.fsPath ?? activeFile()?.fsPath;

    const reg = (id: string, fn: (...args: any[]) => any): any => {
        try {
            return vscode.commands.registerCommand(id, (...args: any[]) => {
                Telemetry.recordCommand(id);
                return fn(...args);
            });
        } catch {
            return { dispose: (): void => undefined };
        }
    };

    const runCmd = (id: string, buildArgs: (file: string) => string[]) =>
        reg(id, async (uri?: vscode.Uri) => {
            const file = fileOf(uri);
            if (!file) {
                vscode.window.showWarningMessage(`Lexicon: open a .lex file first (${id}).`);
                return;
            }
            // Syntax gate BEFORE running: clear line:col errors, never a
            // surprise at runtime. Errors block with options; warnings
            // only notify and let execution proceed.
            try {
                const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(file));
                const findings = precheckLexicon(doc.getText());
                const errors = findings.filter((f) => f.severity === 'error');
                const warnings = findings.filter((f) => f.severity !== 'error');
                for (const w of warnings) {
                    vscode.window.showWarningMessage(
                        `Lexicon: ${w.code} line ${w.line + 1}, col ${w.col + 1}: ${w.message}`
                    );
                }
                if (errors.length > 0) {
                    const first = errors[0];
                    const choice = await vscode.window.showErrorMessage(
                        `Lexicon: ${errors.length} syntax error(s) — first at line ${first.line + 1}, col ${first.col + 1}: ${first.message}`,
                        'Show Problems',
                        'Run Anyway'
                    );
                    if (choice === 'Show Problems') {
                        await vscode.commands.executeCommand('workbench.actions.view.problems');
                        return;
                    }
                    if (choice !== 'Run Anyway') {
                        return;
                    }
                }
            } catch {
                // Gate is best-effort: if the document cannot be read,
                // fall through and let `lex` report.
            }
            runInTerminal('Lexicon', buildArgs(file));
        });

    context.subscriptions.push(
        runCmd('lexicon.run', (f) => ['run', f]),
        reg('lexicon.runWithArgs', async (uri?: vscode.Uri) => {
            const file = fileOf(uri);
            if (!file) {
                vscode.window.showWarningMessage('Lexicon: open a .lex file first (lexicon.runWithArgs).');
                return;
            }
            const extra = await vscode.window.showInputBox({ prompt: 'Arguments (appended after --)', value: '' });
            const args = ['run', file];
            if (extra && extra.trim().length > 0) {
                args.push('--', ...extra.trim().split(/\s+/));
            }
            runInTerminal('Lexicon', args);
        }),
        runCmd('lexicon.check', (f) => ['check', f]),
        runCmd('lexicon.vet', (f) => ['vet', f]),
        reg('lexicon.lint', () => runInTerminal('Lexicon', ['lint'])),
        runCmd('lexicon.fmt', (f) => ['fmt', f]),
        runCmd('lexicon.doc', (f) => ['doc', f]),
        runCmd('lexicon.trace', (f) => ['trace', f]),
        reg('lexicon.test', () => runInTerminal('Lexicon', ['test'])),
        runCmd('lexicon.build', (f) => ['build', f, '--release']),
        runCmd('lexicon.debug', (f) => ['debug', f]),
        reg('lexicon.serve', async (uri?: vscode.Uri) => {
            const file = fileOf(uri);
            const host = (await vscode.window.showInputBox({ prompt: 'Bind host for lex serve', value: '127.0.0.1' })) ?? '127.0.0.1';
            const port = (await vscode.window.showInputBox({ prompt: 'Bind port for lex serve', value: '3000' })) ?? '3000';
            const args = ['serve'];
            if (file) {
                args.push(file);
            }
            args.push('--host', host.trim() || '127.0.0.1', '--port', port.trim() || '3000');
            runInTerminal('Lexicon Serve', args);
        }),
        reg('lexicon.testNet', async () => {
            const host = (await vscode.window.showInputBox({ prompt: 'Target host for lex test-net', value: '127.0.0.1' })) ?? '127.0.0.1';
            const port = (await vscode.window.showInputBox({ prompt: 'Target port for lex test-net', value: '3000' })) ?? '3000';
            runInTerminal('Lexicon Net', ['test-net', '--host', host.trim() || '127.0.0.1', '--port', port.trim() || '3000']);
        }),
        reg('lexicon.dnsCheck', async () => {
            const host = (await vscode.window.showInputBox({ prompt: 'Host or URL for lex dns-check', value: 'example.com' })) ?? 'example.com';
            runInTerminal('Lexicon DNS', ['dns-check', '--host', host.trim() || 'example.com']);
        }),
        reg('lexicon.tlsCheck', async () => {
            const host = (await vscode.window.showInputBox({ prompt: 'SNI host for lex tls-check', value: 'example.com' })) ?? 'example.com';
            const port = (await vscode.window.showInputBox({ prompt: 'TLS port for lex tls-check', value: '443' })) ?? '443';
            runInTerminal('Lexicon TLS', ['tls-check', '--host', host.trim() || 'example.com', '--port', port.trim() || '443']);
        }),
        reg('lexicon.newProject', async () => {
            await runNewProjectFlow();
        }),
        reg('lexicon.showProblems', async () => {
            await vscode.commands.executeCommand('workbench.actions.view.problems');
        }),
        reg('lexicon.showSnippets', async () => {
            const doc = await vscode.workspace.openTextDocument(
                vscode.Uri.joinPath(context.extensionUri, 'SNIPPETS.md')
            );
            await vscode.window.showTextDocument(doc, { preview: true });
        }),
        reg('lexicon.env', async () => {
            const out = LexState.output;
            const cfg = vscode.workspace.getConfiguration('lexicon');
            out.appendLine(`lex binary : ${lexBin()}`);
            out.appendLine(`lex (abs)  : ${resolveLexBinaryAbs() || '(not found on PATH)'}`);
            const roots = sdkRoots();
            const firstHit = roots.find(r => { try { return fs.existsSync(path.join(r, 'std')); } catch { return false; } });
            out.appendLine(`sdk lib    : ${firstHit || '(none of the candidate roots contain std/)'}`);
            out.appendLine(`sdk roots  : ${roots.length} candidate(s)`);
            for (const r of roots) {
                let mark = '  ';
                try { mark = fs.existsSync(r) ? '+ ' : '- '; } catch { mark = '? '; }
                out.appendLine(`${mark}${r}`);
            }
            out.appendLine(`features   : ${cfg.get('features') || '(none)'}`);
            out.appendLine(`target     : ${cfg.get('target') || '(native)'}`);
            out.show();
        }),
        reg('lexicon.telemetry.show', async () => {
            Telemetry.showStats();
            void vscode.window.showInformationMessage('Lexicon: command stats written to the Lexicon output channel (local only).');
        }),
        reg('lexicon.telemetry.reset', async () => {
            Telemetry.resetStats();
            void vscode.window.showInformationMessage('Lexicon: local command stats cleared.');
        }),
    );
}

export function deactivate(): void {
    // nothing to dispose beyond subscriptions
}
