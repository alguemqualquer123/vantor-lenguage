// Validates the icon theme: every iconDefinition path exists on disk and every
// mapping (file/fileExtensions/languageIds/light/highContrast) points to a
// defined iconDefinition key. Catches the "lex" vs "_lex" class of bugs.
const fs = require('fs');
const path = require('path');

const ROOT = path.resolve(__dirname, '..');
const theme = JSON.parse(fs.readFileSync(path.join(ROOT, 'themes/lexicon-icon-theme.json'), 'utf8'));

let errors = 0;
const fail = (m) => { console.error('  x ' + m); errors++; };

const defs = theme.iconDefinitions || {};
console.log('iconDefinitions:', Object.keys(defs).length);

// 1. every iconDefinition file exists (iconPath resolves relative to the theme file in themes/)
for (const [key, def] of Object.entries(defs)) {
  const p = path.resolve(ROOT, 'themes', def.iconPath);
  if (!fs.existsSync(p)) fail(`iconDefinition ${key} -> missing file ${def.iconPath} (resolved ${p})`);
}

// 2. collect all references and ensure each points to a defined key
function checkMap(obj, label) {
  if (!obj) return;
  for (const [k, v] of Object.entries(obj)) {
    if (typeof v === 'string' && !(v in defs)) fail(`${label}.${k} -> undefined iconDefinition "${v}"`);
  }
}
if (typeof theme.file === 'string' && !(theme.file in defs)) fail(`file -> undefined "${theme.file}"`);
checkMap(theme.fileExtensions, 'fileExtensions');
checkMap(theme.languageIds, 'languageIds');
checkMap(theme.fileNames, 'fileNames');
if (theme.light) {
  if (typeof theme.light.file === 'string' && !(theme.light.file in defs)) fail(`light.file -> undefined "${theme.light.file}"`);
  checkMap(theme.light.fileExtensions, 'light.fileExtensions');
  checkMap(theme.light.languageIds, 'light.languageIds');
}
if (theme.highContrast) {
  if (typeof theme.highContrast.file === 'string' && !(theme.highContrast.file in defs)) fail(`highContrast.file -> undefined "${theme.highContrast.file}"`);
  checkMap(theme.highContrast.fileExtensions, 'highContrast.fileExtensions');
}

// 3. lexicon language id must be mapped (the whole point of this theme)
if (!('lexicon' in (theme.languageIds || {}))) fail('languageIds.lexicon missing');
if (!('lex' in (theme.fileExtensions || {}))) fail('fileExtensions.lex missing');
if (!('lang' in (theme.fileExtensions || {}))) fail('fileExtensions.lang missing');

if (errors) { console.error(`ICON THEME: ${errors} error(s)`); process.exit(1); }
console.log('ICON THEME OK — no dangling references');
