// Grammar proof: tokenizes sample Lex with the SHIPPED grammar through
// vscode-textmate (the same engine VS Code uses), backed by a tiny pure-JS
// Oniguruma shim (no native/WASM deps, deterministic exit code).
// Run: node gramtest.js   (exit 0 = all expectations met)
const fs = require('fs');
const path = require('path');
const vsctm = require('vscode-textmate');

class JsOnigString {
    constructor(str) {
        this.content = str;
    }
}

class JsOnigScanner {
    constructor(sources) {
        this.res = sources.map((s) => {
            try {
                return new RegExp(s, 'dy');
            } catch {
                return null;
            }
        });
    }
    findNextMatchSync(string, startPosition) {
        const s = typeof string === 'string' ? string : string.content;
        let best = null;
        for (let i = 0; i < this.res.length; i++) {
            const re = this.res[i];
            if (!re) {
                continue;
            }
            for (let pos = startPosition; pos <= s.length; pos++) {
                re.lastIndex = pos;
                const m = re.exec(s);
                if (m) {
                    const caps = (m.indices || []).map((ix) =>
                        ix ? { start: ix[0], end: ix[1] } : { start: -1, end: -1 }
                    );
                    const cand = {
                        index: i,
                        captureIndices: [{ start: pos, end: pos + m[0].length }, ...caps.slice(1)],
                    };
                    if (!best || pos < best.captureIndices[0].start) {
                        best = cand;
                    }
                    break;
                }
            }
        }
        return best;
    }
}

const SAMPLE = [
    'import std::strings;',
    'pub fn main() -> void {',
    '    let s = "oi {nome}";',
    '    Console::log(strings::ToUpper(s));',
    '    let f = |x| x * 2;',
    '    @Get("/")',
    '}',
].join('\n');

const EXPECT = [
    ['import', 'keyword'],
    ['std', 'entity.name.namespace'],
    ['pub', 'keyword'],
    ['fn', 'keyword'],
    ['main', 'entity.name.function'],
    ['let', 'keyword'],
    ['"oi {nome}"', 'string.quoted.double'],
    ['{nome}', 'string-interpolation'],
    ['Console', 'entity.name.type'],
    ['::', 'keyword.operator.path'],
    ['log', 'entity.name.function'],
    ['|x|', 'keyword.operator.closure'],
    ['@Get', 'keyword'],
];

async function main() {
    const grammarJson = JSON.parse(
        fs.readFileSync(path.join(__dirname, 'syntaxes', 'lexicon.tmLanguage.json'), 'utf8')
    );
    const registry = new vsctm.Registry({
        onigLib: Promise.resolve({
            createOnigScanner: (sources) => new JsOnigScanner(sources),
            createOnigString: (str) => new JsOnigString(str),
        }),
        loadGrammar: async () => grammarJson,
    });
    const grammar = await registry.loadGrammar('source.lexicon');
    if (!grammar) {
        console.error('FAIL: grammar did not load');
        process.exit(1);
    }
    const seen = [];
    let rule = vsctm.INITIAL;
    for (const line of SAMPLE.split('\n')) {
        const r = grammar.tokenizeLine(line, rule);
        rule = r.ruleStack;
        // Per-character scope map: expectations match a character window
        // where EVERY char carries the scope (handles split tokens).
        const charScopes = Array.from({ length: line.length }, () => []);
        for (const t of r.tokens) {
            seen.push({ text: line.slice(t.startIndex, t.endIndex), scopes: t.scopes });
            for (let i = t.startIndex; i < t.endIndex; i++) {
                charScopes[i] = t.scopes;
            }
        }
        seen.charScopes = (seen.charScopes || []).concat([{ line, charScopes }]);
    }
    let fails = 0;
    for (const [text, scopeFrag] of EXPECT) {
        let hit = false;
        for (const { line, charScopes } of seen.charScopes) {
            for (let i = 0; i + text.length <= line.length; i++) {
                if (line.slice(i, i + text.length) !== text) {
                    continue;
                }
                let ok = true;
                for (let j = i; j < i + text.length; j++) {
                    if (!charScopes[j].some((s) => s.includes(scopeFrag))) {
                        ok = false;
                        break;
                    }
                }
                if (ok) {
                    hit = true;
                    break;
                }
            }
            if (hit) {
                break;
            }
        }
        if (!hit) {
            console.error(`FAIL: no window ${JSON.stringify(text)} fully scoped ${scopeFrag}`);
            fails++;
        }
    }
    if (fails > 0) {
        process.exit(1);
    }
    console.log(`GRAMMAR-OK: ${seen.length} tokens, all ${EXPECT.length} expectations met`);
    process.exit(0);
}

main().catch((e) => {
    console.error('FAIL:', (e && e.message) || e);
    process.exit(1);
});
