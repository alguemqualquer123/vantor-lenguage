// Tokenization harness: proves the TextMate grammar produces scopes/colors.
const fs = require('fs');
const path = require('path');
const onigasm = require('onigasm');
const vsctm = require('vscode-textmate');

const ROOT = path.resolve(__dirname, '..');

async function main() {
  const wasm = fs.readFileSync(path.join(ROOT, 'node_modules/onigasm/lib/onigasm.wasm'));
  await onigasm.loadWASM(wasm.buffer);

  const registry = new vsctm.Registry({
    onigLib: Promise.resolve({
      createOnigScanner: (p) => new onigasm.OnigScanner(p),
      createOnigString: (s) => new onigasm.OnigString(s),
    }),
    loadGrammar: (scopeName) => {
      if (scopeName === 'source.lexicon') {
        const g = JSON.parse(fs.readFileSync(path.join(ROOT, 'syntaxes/lexicon.tmLanguage.json'), 'utf8'));
        return vsctm.parseRawGrammar(JSON.stringify(g), 'lexicon.tmLanguage.json');
      }
      return null;
    },
  });

  const grammar = await registry.loadGrammar('source.lexicon');
  if (!grammar) { console.error('FAILED to load grammar'); process.exit(2); }

  const file = process.argv[2];
  const text = file ? fs.readFileSync(file, 'utf8') : fs.readFileSync(path.join(ROOT, 'sample.lex'), 'utf8');
  const lines = text.split('\n');
  let ruleStack = vsctm.INITIAL;
  let totalTokens = 0;
  let coloredTokens = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const res = grammar.tokenizeLine(line, ruleStack);
    ruleStack = res.ruleStack;
    let out = [];
    for (const t of res.tokens) {
      const txt = line.substring(t.startIndex, t.endIndex);
      if (!txt.trim()) continue;
      totalTokens++;
      const scopes = t.scopes;
      // "colored" = has a scope beyond the root source.lexicon
      const meaningful = scopes.some(s => s !== 'source.lexicon');
      if (meaningful) coloredTokens++;
      out.push(`   [${txt}] -> ${scopes.join(' ')}`);
    }
    if (process.env.VERBOSE) console.log(`L${i + 1}: ${line}\n${out.join('\n')}`);
  }
  console.log(`\nTOTAL non-ws tokens: ${totalTokens}, with meaningful scope: ${coloredTokens}`);
  console.log(`Coverage: ${((coloredTokens / Math.max(1, totalTokens)) * 100).toFixed(1)}%`);
  if (coloredTokens / Math.max(1, totalTokens) < 0.5) {
    console.error('WARNING: less than 50% of tokens are scoped — grammar may be broken.');
    process.exit(3);
  }
  console.log('GRAMMAR TOKENIZATION OK');
}

main().catch(e => { console.error('HARNESS ERROR:', e); process.exit(1); });
