// Load test: mock the `vscode` API, activate the compiled extension, then
// exercise the registered providers. Verifies:
//   - activate() does not throw and registers the expected providers
//   - the SDK definition provider resolves stdlib symbols to real files
//   - the semantic tokens provider survives a large file (stress)
const fs = require('fs');
const path = require('path');
const os = require('os');
const Module = require('module');

const ROOT = path.resolve(__dirname, '..');

// ---- capture registrations ----
const registered = { definition: [], semantic: [], hover: [], completion: [], other: 0 };

function makeRange(a, b) { return { start: a, end: b }; }
class Position { constructor(line, character) { this.line = line; this.character = character; } }
class Range { constructor(a, b, c, d) { if (a instanceof Position) { this.start = a; this.end = b; } else { this.start = new Position(a, b); this.end = new Position(c, d); } } }
class Location { constructor(uri, range) { this.uri = uri; this.range = range; } }
class Uri {
  constructor(fsPath) { this.fsPath = fsPath; this.scheme = 'file'; }
  static file(p) { return new Uri(p); }
  toString() { return 'file://' + this.fsPath; }
}
class SemanticTokensBuilder {
  constructor() { this._tokens = []; }
  push(line, char, len, type, mods) { this._tokens.push({ line, char, len, type, mods }); }
  build() { return { data: this._tokens, count: this._tokens.length }; }
}
class SemanticTokensLegend { constructor(types, mods) { this.tokenTypes = types; this.tokenModifiers = mods; } }

const configStore = { path: 'lex', sdkPath: '', features: '', target: '' };
const mockVscode = {
  Position, Range, Location, Uri, SemanticTokensBuilder, SemanticTokensLegend,
  StatusBarAlignment: { Left: 1, Right: 2 },
  workspace: {
    workspaceFolders: [{ uri: Uri.file(ROOT) }],
    getConfiguration: () => ({ get: (k) => configStore[k], update: async () => {} }),
    onDidChangeTextDocument: () => ({ dispose() {} }),
    onDidSaveTextDocument: () => ({ dispose() {} }),
    findFiles: async () => [],
    openTextDocument: async (u) => ({ uri: u, getText: () => fs.existsSync(u.fsPath) ? fs.readFileSync(u.fsPath, 'utf8') : '' }),
  },
  window: {
    createOutputChannel: () => ({ appendLine() {}, show() {}, dispose() {} }),
    createStatusBarItem: () => ({ show() {}, hide() {}, dispose() {}, text: '', tooltip: '', command: '' }),
    onDidChangeActiveTextEditor: () => ({ dispose() {} }),
    activeTextEditor: undefined,
    showInformationMessage: async () => {},
    showErrorMessage: async () => {},
    terminals: [], createTerminal: () => ({ show() {}, sendText() {} }),
  },
  languages: {
    createDiagnosticCollection: () => ({ set() {}, delete() {}, clear() {}, dispose() {} }),
    registerDefinitionProvider: (sel, p) => { registered.definition.push(p); return { dispose() {} }; },
    registerDocumentSemanticTokensProvider: (sel, p, l) => { registered.semantic.push({ p, l }); return { dispose() {} }; },
    registerHoverProvider: (sel, p) => { registered.hover.push(p); return { dispose() {} }; },
    registerCompletionItemProvider: (sel, p) => { registered.completion.push(p); return { dispose() {} }; },
    registerDocumentSymbolProvider: () => ({ dispose() {} }),
    registerCodeLensProvider: () => ({ dispose() {} }),
    registerSignatureHelpProvider: () => ({ dispose() {} }),
    registerInlayHintsProvider: () => ({ dispose() {} }),
    registerFoldingRangeProvider: () => ({ dispose() {} }),
    registerDocumentColorProvider: () => ({ dispose() {} }),
    registerReferenceProvider: () => ({ dispose() {} }),
    registerRenameProvider: () => ({ dispose() {} }),
    registerCodeActionsProvider: () => ({ dispose() {} }),
    registerDocumentFormattingEditProvider: () => ({ dispose() {} }),
    registerWorkspaceSymbolProvider: () => ({ dispose() {} }),
  },
  commands: { registerCommand: (id, cb) => { registered.other++; return { dispose() {} }; }, executeCommand: async () => {} },
  tasks: { registerTaskProvider: () => ({ dispose() {} }) },
  tests: undefined,
  env: { machineId: 'test', sessionId: 'test' },
  DebugAdapterExecutable: function () {},
  EventEmitter: class { constructor() { this.event = () => ({ dispose() {} }); } fire() {} },
  CodeActionKind: { QuickFix: 'quickfix' },
  TestRunProfileKind: { Run: 1 },
};

// inject the mock into require cache
const origResolve = Module._resolveFilename;
Module._resolveFilename = function (request, ...args) {
  if (request === 'vscode') return 'vscode';
  return origResolve.call(this, request, ...args);
};
require.cache['vscode'] = { id: 'vscode', filename: 'vscode', loaded: true, exports: mockVscode };

let errors = 0;
const fail = (m) => { console.error('  x ' + m); errors++; };
const ok = (m) => console.log('  + ' + m);

async function main() {
  const ext = require(path.join(ROOT, 'out/extension.js'));
  if (typeof ext.activate !== 'function') { fail('extension.js has no activate()'); process.exit(1); }

  const subs = [];
  const ctx = { subscriptions: subs, extensionPath: ROOT, extensionUri: Uri.file(ROOT), globalState: { get: () => undefined, update: async () => {} }, workspaceState: { get: () => undefined, update: async () => {} } };

  console.log('== activate() ==');
  try {
    ext.activate(ctx);
    ok(`activate() returned; ${subs.length} subscriptions; providers: def=${registered.definition.length} sem=${registered.semantic.length} hover=${registered.hover.length} compl=${registered.completion.length}`);
  } catch (e) {
    fail('activate() threw: ' + (e && e.stack || e));
    process.exit(1);
  }
  if (registered.definition.length < 1) fail('no definition provider registered');
  if (registered.semantic.length < 1) fail('no semantic tokens provider registered');

  // ---- SDK go-to-definition ----
  console.log('== SDK go-to-definition ==');
  const sdkHome = path.join(os.homedir(), '.lexicon', 'sdk', 'lib');
  const doc = (text) => ({
    getText: () => text,
    lineAt: (n) => ({ text: text.split('\n')[n] || '' }),
    positionAt: (off) => { const before = text.slice(0, off).split('\n'); return new Position(before.length - 1, before[before.length - 1].length); },
    uri: Uri.file(path.join(ROOT, 'test/scratch.lex')),
  });
  const cases = [
    { line: 'strings::ToUpper("a")', col: 3, expectFile: 'std/strings.lex', expectMember: 'ToUpper' },
    { line: 'Console.writeLine("x")', col: 3, expectFile: 'std/native/console.lex', expectMember: 'writeLine' },
    { line: 'let s = math::Max(1,2)', col: 12, expectFile: 'std/math.lex', expectMember: 'Max' },
    { line: 'json::Marshal(v)', col: 2, expectFile: 'std/encoding/json.lex', expectMember: 'Marshal' },
  ];
  let resolvedAny = false;
  for (const c of cases) {
    const d = doc(c.line);
    let loc = null;
    for (const p of registered.definition) {
      try { const r = p.provideDefinition(d, new Position(0, c.col)); if (r) { loc = r; break; } } catch (e) { /* next provider */ }
    }
    if (!loc) { fail(`no definition for "${c.line}" @${c.col}`); continue; }
    const fp = loc.uri.fsPath || loc.uri.path;
    const expected = path.join(sdkHome, c.expectFile);
    if (path.resolve(fp) !== path.resolve(expected)) { fail(`"${c.line}" -> ${fp} (expected ${expected})`); continue; }
    resolvedAny = true;
    ok(`"${c.line}" -> ${path.relative(sdkHome, fp)}:${loc.range.start.line}`);
  }
  if (!resolvedAny) fail('SDK definition provider resolved 0 symbols — Ctrl+Click would do nothing');

  // ---- semantic tokens stress ----
  console.log('== semantic tokens (stress) ==');
  const sem = registered.semantic[0].p;
  const big = [];
  for (let i = 0; i < 4000; i++) {
    big.push(`pub fn f${i}(x${i}: i32) -> String { let y${i} = Module${i}::call(x${i}); return "v${i}"; } // c${i}`);
  }
  const bigText = big.join('\n');
  const t0 = Date.now();
  let tokens = 0;
  try {
    const res = sem.provideDocumentSemanticTokens(doc(bigText));
    tokens = res.count !== undefined ? res.count : (res.data ? res.data.length : 0);
    const dt = Date.now() - t0;
    ok(`tokenized ${bigText.length.toLocaleString()} chars / ${big.length} lines in ${dt}ms -> ${tokens} tokens`);
    if (dt > 5000) fail('semantic tokenization too slow (>5s)');
    if (tokens < 1000) fail('suspiciously few semantic tokens for a large file');
  } catch (e) {
    fail('semantic provider threw on large file: ' + (e && e.stack || e));
  }

  if (errors) { console.error(`\nLOAD TEST FAILED: ${errors} error(s)`); process.exit(1); }
  console.log('\nLOAD TEST OK');
}

main().catch(e => { console.error('HARNESS ERROR:', e); process.exit(2); });
