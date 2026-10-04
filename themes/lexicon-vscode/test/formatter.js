// Tests for src/format.ts (compiled to out/format.js): the .editorconfig
// resolver and the built-in formatter. The strongest guarantee is idempotency
// across every real SDK module — format(format(x)) === format(x) and no crash.
const fs = require('fs');
const path = require('path');
const os = require('os');

const ROOT = path.resolve(__dirname, '..');
const REPO = path.resolve(ROOT, '../..');           // lex-lenguage repo root
const fmt = require(path.join(ROOT, 'out/format.js'));

let errors = 0;
const fail = (m) => { console.error('  x ' + m); errors++; };
const ok = (m) => console.log('  + ' + m);

function walkLex(dir, out) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walkLex(p, out);
    else if (e.name.endsWith('.lex')) out.push(p);
  }
  return out;
}

// ---- 1. editorconfig resolver ----
console.log('== resolveEditorConfig ==');
(function () {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'ec-'));
  fs.writeFileSync(path.join(tmp, '.editorconfig'),
    'root = true\n\n[*]\nindent_style = space\nindent_size = 2\nmax_line_length = 80\n\n[*.lex]\nindent_size = 4\ntrim_trailing_whitespace = true\n');
  fs.mkdirSync(path.join(tmp, 'src'));
  const lexFile = path.join(tmp, 'src', 'a.lex');
  fs.writeFileSync(lexFile, 'x');
  const cfgLex = fmt.resolveEditorConfig(lexFile);
  if (cfgLex.indent_size !== 4) fail(`.lex indent_size expected 4, got ${cfgLex.indent_size}`);
  if (cfgLex.indent_style !== 'space') fail('.lex indent_style expected space');
  if (cfgLex.max_line_length !== 80) fail(`.lex should inherit max_line_length 80, got ${cfgLex.max_line_length}`);
  if (cfgLex.trim_trailing_whitespace !== true) fail('.lex trim_trailing_whitespace expected true');
  const jsonFile = path.join(tmp, 'b.json');
  fs.writeFileSync(jsonFile, '{}');
  const cfgJson = fmt.resolveEditorConfig(jsonFile);
  if (cfgJson.indent_size !== 2) fail(`.json indent_size expected 2 (from [*]), got ${cfgJson.indent_size}`);
  ok('section matching + inheritance + root=true stop');

  // repo .editorconfig resolves for a .lex file
  const repoCfg = fmt.resolveEditorConfig(path.join(REPO, 'pipes.lex'));
  if (repoCfg.indent_size !== 4) fail(`repo pipes.lex indent_size expected 4, got ${repoCfg.indent_size}`);
  if (repoCfg.charset !== 'utf-8') fail('repo charset expected utf-8');
  ok('repo .editorconfig resolves for *.lex (indent_size=4, charset=utf-8)');

  // indentUnit
  if (fmt.indentUnit({ indent_style: 'space', indent_size: 2 }) !== '  ') fail('indentUnit space/2 wrong');
  if (fmt.indentUnit({ indent_style: 'tab' }) !== '\t') fail('indentUnit tab wrong');
  ok('indentUnit');
  fs.rmSync(tmp, { recursive: true, force: true });
})();

// ---- 2. formatter behavior ----
console.log('== formatLexiconText (behavior) ==');
(function () {
  const cfg = { indent_style: 'space', indent_size: 4, trim_trailing_whitespace: true, insert_final_newline: true };

  // reindent by brace depth
  const src = 'pub fn main() -> void {\nlet x = 1;\nif x > 0 {\nConsole.writeLine("hi");\n}\n}\n';
  const want = 'pub fn main() -> void {\n    let x = 1;\n    if x > 0 {\n        Console.writeLine("hi");\n    }\n}\n';
  const got = fmt.formatLexiconText(src, cfg);
  if (got !== want) fail('reindent mismatch:\n---got---\n' + got + '---want---\n' + want);
  else ok('brace-depth reindent');

  // closing brace dedents its own line
  const src2 = 'fn a() {\n    let y = 2;\n        }\n';
  const got2 = fmt.formatLexiconText(src2, cfg);
  if (!got2.endsWith('    let y = 2;\n}\n')) fail('closer dedent wrong:\n' + got2);
  else ok('leading closer dedents');

  // braces inside strings/interp are NOT counted for depth
  const src3 = 'fn b() {\n    let s = "a { b } c";\n    let t = "interp {x} here";\n}\n';
  const got3 = fmt.formatLexiconText(src3, cfg);
  if (got3 !== src3) fail('string braces affected depth:\n' + got3);
  else ok('braces in strings ignored');

  // block comment content preserved, not reindented
  const src4 = 'fn c() {\n    /* keep\n   this indentation\n    */\n    let z = 0;\n}\n';
  const got4 = fmt.formatLexiconText(src4, cfg);
  if (!got4.includes('   this indentation')) fail('block comment reflowed:\n' + got4);
  else ok('block comment lines preserved');

  // trailing whitespace trimmed + final newline enforced
  const src5 = 'fn d() {   \n    let w = 1;   \n}\n\n\n';
  const got5 = fmt.formatLexiconText(src5, cfg);
  if (got5 !== 'fn d() {\n    let w = 1;\n}\n') fail('trim/final-newline wrong:\n' + JSON.stringify(got5));
  else ok('trailing whitespace trimmed + single final newline');

  // CRLF normalization
  const got6 = fmt.formatLexiconText('fn e() {\r\nlet v = 1;\r\n}\r\n', cfg);
  if (got6.includes('\r')) fail('CRLF not normalized');
  else ok('EOL normalized to LF');

  // tabs indent style
  const got7 = fmt.formatLexiconText('fn f() {\nlet q = 1;\n}\n', { indent_style: 'tab' });
  if (!got7.includes('\tlet q = 1;')) fail('tab indent wrong:\n' + JSON.stringify(got7));
  else ok('tab indent style');
})();

// ---- 3. idempotency across all real SDK modules ----
console.log('== idempotency over SDK ==');
(function () {
  const sdkDir = path.join(REPO, 'lib', 'std');
  if (!fs.existsSync(sdkDir)) { console.log('  (skip: lib/std not found)'); return; }
  const files = walkLex(sdkDir, []);
  const cfg = { indent_style: 'space', indent_size: 4, trim_trailing_whitespace: true, insert_final_newline: true };
  let checked = 0, nonIdempotent = 0, crashed = 0;
  for (const f of files) {
    const text = fs.readFileSync(f, 'utf8');
    let once, twice;
    try {
      once = fmt.formatLexiconText(text, cfg);
      twice = fmt.formatLexiconText(once, cfg);
    } catch (e) { crashed++; fail(`crash on ${path.relative(REPO, f)}: ${e.message}`); continue; }
    checked++;
    if (once !== twice) {
      nonIdempotent++;
      if (nonIdempotent <= 3) fail(`NOT idempotent: ${path.relative(REPO, f)}`);
    }
  }
  console.log(`  checked ${checked} SDK files; non-idempotent: ${nonIdempotent}; crashed: ${crashed}`);
  if (crashed === 0 && nonIdempotent === 0) ok('formatter is stable + idempotent on all SDK modules');
})();

if (errors) { console.error(`\nFORMATTER TEST FAILED: ${errors} error(s)`); process.exit(1); }
console.log('\nFORMATTER TEST OK');
