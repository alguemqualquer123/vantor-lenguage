#!/usr/bin/env node
/*
 * release.js — one-shot build + test + package for the Lexicon VSCode extension.
 *
 * Usage:
 *   node scripts/release.js            # bump patch, compile, test, package
 *   node scripts/release.js --minor    # bump minor
 *   node scripts/release.js --major    # bump major
 *   node scripts/release.js --version 2.0.0
 *   node scripts/release.js --no-bump  # keep current version
 *   node scripts/release.js --skip-tests
 *
 * Produces lexicon-super-<version>.vsix in the extension root. No external
 * dependencies: the .vsix (a ZIP) is written with Node's built-in zlib.
 */
'use strict';
const fs = require('fs');
const path = require('path');
const zlib = require('zlib');
const cp = require('child_process');

const ROOT = path.resolve(__dirname, '..');

// ---------------------------------------------------------------------------
// args
// ---------------------------------------------------------------------------
const argv = process.argv.slice(2);
const has = (f) => argv.includes(f);
const getVal = (f) => { const i = argv.indexOf(f); return i >= 0 ? argv[i + 1] : null; };

// ---------------------------------------------------------------------------
// CRC32 (for ZIP)
// ---------------------------------------------------------------------------
const CRC_TABLE = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = (c & 1) ? (0xEDB88320 ^ (c >>> 1)) : (c >>> 1);
    t[n] = c >>> 0;
  }
  return t;
})();
function crc32(buf) {
  let c = 0xFFFFFFFF;
  for (let i = 0; i < buf.length; i++) c = CRC_TABLE[(c ^ buf[i]) & 0xFF] ^ (c >>> 8);
  return (c ^ 0xFFFFFFFF) >>> 0;
}

// ---------------------------------------------------------------------------
// minimal ZIP writer (deflate)
// ---------------------------------------------------------------------------
function zip(files) {
  // files: [{ name, data(Buffer), time(Date) }]
  const chunks = [];
  const central = [];
  let offset = 0;
  for (const f of files) {
    const nameBuf = Buffer.from(f.name, 'utf8');
    const comp = zlib.deflateRawSync(f.data, { level: 9 });
    const crc = crc32(f.data);
    const dosTime = ((f.time.getHours() << 11) | (f.time.getMinutes() << 5) | (Math.floor(f.time.getSeconds() / 2))) & 0xFFFF;
    const dosDate = (((f.time.getFullYear() - 1980) << 9) | ((f.time.getMonth() + 1) << 5) | f.time.getDate()) & 0xFFFF;
    const local = Buffer.alloc(30);
    local.writeUInt32LE(0x04034b50, 0);
    local.writeUInt16LE(20, 4);            // version needed
    local.writeUInt16LE(0x0800, 6);        // flags: UTF-8 names
    local.writeUInt16LE(8, 8);             // method: deflate
    local.writeUInt16LE(dosTime, 10);
    local.writeUInt16LE(dosDate, 12);
    local.writeUInt32LE(crc, 14);
    local.writeUInt32LE(comp.length, 18);
    local.writeUInt32LE(f.data.length, 22);
    local.writeUInt16LE(nameBuf.length, 26);
    local.writeUInt16LE(0, 28);
    chunks.push(local, nameBuf, comp);
    const cd = Buffer.alloc(46);
    cd.writeUInt32LE(0x02014b50, 0);
    cd.writeUInt16LE(20, 4);
    cd.writeUInt16LE(20, 6);
    cd.writeUInt16LE(0x0800, 8);
    cd.writeUInt16LE(8, 10);
    cd.writeUInt16LE(dosTime, 12);
    cd.writeUInt16LE(dosDate, 14);
    cd.writeUInt32LE(crc, 16);
    cd.writeUInt32LE(comp.length, 20);
    cd.writeUInt32LE(f.data.length, 24);
    cd.writeUInt16LE(nameBuf.length, 28);
    cd.writeUInt16LE(0, 30);
    cd.writeUInt16LE(0, 32);
    cd.writeUInt16LE(0, 34);
    cd.writeUInt16LE(0, 36);
    cd.writeUInt32LE(0, 38);
    cd.writeUInt32LE(offset, 42);
    central.push(Buffer.concat([cd, nameBuf]));
    offset += local.length + nameBuf.length + comp.length;
  }
  const centralBuf = Buffer.concat(central);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(0, 4);
  end.writeUInt16LE(0, 6);
  end.writeUInt16LE(files.length, 8);
  end.writeUInt16LE(files.length, 10);
  end.writeUInt32LE(centralBuf.length, 12);
  end.writeUInt32LE(offset, 16);
  end.writeUInt16LE(0, 20);
  return Buffer.concat([...chunks, centralBuf, end]);
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------
function readJson(p) { return JSON.parse(fs.readFileSync(p, 'utf8')); }
function writeJson(p, obj) { fs.writeFileSync(p, JSON.stringify(obj, null, 2) + '\n', 'utf8'); }
function run(cmd, args, opts = {}) {
  const r = cp.spawnSync(cmd, args, { cwd: ROOT, stdio: 'inherit', shell: false, ...opts });
  if (r.status !== 0) throw new Error(`command failed (${r.status}): ${cmd} ${args.join(' ')}`);
}
function xmlEscape(s) {
  return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
function bump(version, kind) {
  const m = /^(\d+)\.(\d+)\.(\d+)(.*)$/.exec(version);
  if (!m) throw new Error('bad version: ' + version);
  let [_, maj, min, pat, pre] = m;
  maj = +maj; min = +min; pat = +pat;
  if (kind === 'major') { maj++; min = 0; pat = 0; }
  else if (kind === 'minor') { min++; pat = 0; }
  else { pat++; }
  return `${maj}.${min}.${pat}`;
}

// ---------------------------------------------------------------------------
// collect files to package (deterministic, mirrors vsce defaults)
// ---------------------------------------------------------------------------
const INCLUDE_DIRS = ['icons', 'out', 'snippets', 'syntaxes', 'themes', 'src'];
const INCLUDE_FILES = [
  'package.json', 'package.nls.json', 'package.nls.pt-br.json',
  'language-configuration.json', 'tsconfig.json',
  'GETSTARTED.md', 'IMPLEMENTS.md', 'SNIPPETS.md', 'WALKTHROUGH.md',
];
// manifest Asset paths are lowercase for these two
const RENAME = { 'README.md': 'readme.md', 'CHANGELOG.md': 'changelog.md' };

function walk(dir, base, out) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const abs = path.join(dir, entry.name);
    const rel = (base ? base + '/' : '') + entry.name;
    if (entry.isDirectory()) walk(abs, rel, out);
    else out.push({ abs, rel });
  }
}

function collectFiles() {
  const list = [];
  for (const f of INCLUDE_FILES) {
    const abs = path.join(ROOT, f);
    if (fs.existsSync(abs)) list.push({ abs, rel: f });
  }
  for (const f of Object.keys(RENAME)) {
    const abs = path.join(ROOT, f);
    if (fs.existsSync(abs)) list.push({ abs, rel: RENAME[f] });
  }
  for (const d of INCLUDE_DIRS) {
    const abs = path.join(ROOT, d);
    if (fs.existsSync(abs)) walk(abs, d, list);
  }
  return list;
}

// ---------------------------------------------------------------------------
// manifest + content types (generated from package.json so they never drift)
// ---------------------------------------------------------------------------
function nls(pkg, nlsObj, key) {
  const v = pkg[key];
  if (typeof v === 'string' && v.startsWith('%') && v.endsWith('%')) {
    return nlsObj[v.slice(1, -1)] || v;
  }
  return v || '';
}

function buildManifest(pkg, nlsObj, version) {
  const id = pkg.name;
  const publisher = pkg.publisher;
  const displayName = xmlEscape(nls(pkg, nlsObj, 'displayName'));
  const description = xmlEscape(nls(pkg, nlsObj, 'description'));
  const engine = (pkg.engines && pkg.engines.vscode) || '^1.60.0';
  const categories = (pkg.categories || []).join(',');
  const keywords = (pkg.keywords || []).join(',');
  const repo = (pkg.repository && pkg.repository.url) || '';
  const repoGit = repo.endsWith('.git') ? repo : (repo ? repo + '.git' : '');
  const icon = pkg.icon || 'icons/lexicon-icon.png';
  const bannerColor = (pkg.galleryBanner && pkg.galleryBanner.color) || '#0F1117';
  const bannerTheme = (pkg.galleryBanner && pkg.galleryBanner.theme) || 'dark';
  const tags = [keywords, categories, 'Lexicon', '__ext_lex', '__ext_lang', '__web_extension']
    .filter(Boolean).join(',').replace(/,,+/g, ',');
  return `<?xml version="1.0" encoding="utf-8"?>
<PackageManifest Version="2.0.0" xmlns="http://schemas.microsoft.com/developer/vsx-schema/2011" xmlns:d="http://schemas.microsoft.com/developer/vsx-schema-design/2011">
  <Metadata>
    <Identity Language="en-US" Id="${id}" Version="${version}" Publisher="${publisher}" />
    <DisplayName>${displayName}</DisplayName>
    <Description xml:space="preserve">${description}</Description>
    <Tags>${xmlEscape(tags)}</Tags>
    <Categories>${xmlEscape(categories)}</Categories>
    <GalleryFlags>Public</GalleryFlags>
    <Properties>
      <Property Id="Microsoft.VisualStudio.Code.Engine" Value="${engine}" />
      <Property Id="Microsoft.VisualStudio.Code.ExtensionDependencies" Value="" />
      <Property Id="Microsoft.VisualStudio.Code.ExtensionPack" Value="" />
      <Property Id="Microsoft.VisualStudio.Code.ExtensionKind" Value="workspace,web" />
      <Property Id="Microsoft.VisualStudio.Code.LocalizedLanguages" Value="" />
      <Property Id="Microsoft.VisualStudio.Code.EnabledApiProposals" Value="" />
      <Property Id="Microsoft.VisualStudio.Code.ExecutesCode" Value="true" />
      <Property Id="Microsoft.VisualStudio.Services.Links.Source" Value="${xmlEscape(repoGit)}" />
      <Property Id="Microsoft.VisualStudio.Services.Links.Getstarted" Value="${xmlEscape(repoGit)}" />
      <Property Id="Microsoft.VisualStudio.Services.Links.GitHub" Value="${xmlEscape(repoGit)}" />
      <Property Id="Microsoft.VisualStudio.Services.Links.Support" Value="${xmlEscape((pkg.bugs && pkg.bugs.url) || repo)}" />
      <Property Id="Microsoft.VisualStudio.Services.Links.Learn" Value="${xmlEscape(pkg.homepage || repo)}" />
      <Property Id="Microsoft.VisualStudio.Services.Branding.Color" Value="${bannerColor}" />
      <Property Id="Microsoft.VisualStudio.Services.Branding.Theme" Value="${bannerTheme}" />
      <Property Id="Microsoft.VisualStudio.Services.GitHubFlavoredMarkdown" Value="true" />
      <Property Id="Microsoft.VisualStudio.Services.Content.Pricing" Value="Free"/>
    </Properties>
    <Icon>extension/${icon}</Icon>
  </Metadata>
  <Installation>
    <InstallationTarget Id="Microsoft.VisualStudio.Code"/>
  </Installation>
  <Dependencies/>
  <Assets>
    <Asset Type="Microsoft.VisualStudio.Code.Manifest" Path="extension/package.json" Addressable="true" />
    <Asset Type="Microsoft.VisualStudio.Services.Content.Details" Path="extension/readme.md" Addressable="true" />
    <Asset Type="Microsoft.VisualStudio.Services.Content.Changelog" Path="extension/changelog.md" Addressable="true" />
    <Asset Type="Microsoft.VisualStudio.Services.Icons.Default" Path="extension/${icon}" Addressable="true" />
  </Assets>
</PackageManifest>
`;
}

const CONTENT_TYPES = `<?xml version="1.0" encoding="utf-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension=".js" ContentType="application/javascript"/><Default Extension=".json" ContentType="application/json"/><Default Extension=".md" ContentType="text/markdown"/><Default Extension=".png" ContentType="image/png"/><Default Extension=".svg" ContentType="image/svg+xml"/><Default Extension=".ts" ContentType="video/mp2t"/><Default Extension=".vsixmanifest" ContentType="text/xml"/></Types>`;

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------
function main() {
  const pkgPath = path.join(ROOT, 'package.json');
  const pkg = readJson(pkgPath);
  const nlsObj = readJson(path.join(ROOT, 'package.nls.json'));

  // 1. version
  let version = pkg.version;
  if (has('--no-bump')) {
    console.log(`version: ${version} (no bump)`);
  } else {
    const explicit = getVal('--version');
    const kind = has('--major') ? 'major' : has('--minor') ? 'minor' : 'patch';
    version = explicit || bump(pkg.version, kind);
    pkg.version = version;
    writeJson(pkgPath, pkg);
    console.log(`version: ${pkg.version} -> ${version} (${explicit ? 'explicit' : kind})`);
  }

  // 2. compile
  console.log('\n== compile (tsc) ==');
  run(process.execPath, ['node_modules/typescript/bin/tsc', '-p', './']);

  // 3. tests
  if (!has('--skip-tests')) {
    console.log('\n== tests ==');
    for (const t of ['test/validate-icons.js', 'test/tokenize.js', 'test/load-extension.js']) {
      const p = path.join(ROOT, t);
      if (!fs.existsSync(p)) { console.log(`  (skip ${t}: missing)`); continue; }
      console.log(`-- ${t}`);
      const args = t.endsWith('tokenize.js') ? [t, 'test/sample.lex'] : [t];
      run(process.execPath, args);
    }
  } else {
    console.log('\n== tests skipped ==');
  }

  // 4. package
  console.log('\n== package (.vsix) ==');
  const now = new Date();
  const files = [];
  files.push({ name: '[Content_Types].xml', data: Buffer.from(CONTENT_TYPES, 'utf8'), time: now });
  files.push({ name: 'extension.vsixmanifest', data: Buffer.from(buildManifest(pkg, nlsObj, version), 'utf8'), time: now });
  for (const f of collectFiles()) {
    files.push({ name: 'extension/' + f.rel.split(path.sep).join('/'), data: fs.readFileSync(f.abs), time: fs.statSync(f.abs).mtime });
  }
  const buf = zip(files);
  const outName = `${pkg.name}-${version}.vsix`;
  const outPath = path.join(ROOT, outName);
  fs.writeFileSync(outPath, buf);
  console.log(`  entries: ${files.length}`);
  console.log(`  wrote ${outName} (${(buf.length / 1024).toFixed(1)} KB)`);

  // 5. sanity: re-open the zip and confirm key assets are present
  const names = files.map(f => f.name);
  const required = ['extension/package.json', 'extension/syntaxes/lexicon.tmLanguage.json',
    'extension/themes/lexicon-icon-theme.json', 'extension/out/extension.js',
    'extension/icons/lex.svg', 'extension.vsixmanifest', '[Content_Types].xml'];
  const missing = required.filter(r => !names.includes(r));
  if (missing.length) { console.error('  MISSING from package:', missing); process.exit(1); }
  console.log('  verified required assets present');
  console.log(`\nDONE: ${outPath}`);
}

try { main(); } catch (e) { console.error('\nRELEASE FAILED:', e.message || e); process.exit(1); }
