# Changelog

All notable changes to the Lexicon programming language toolchain.

## [0.3.1] - 2026-10-04

### Added
- Go-to-definition (`textDocument/definition`): Ctrl+Click jumps to the SDK
  source — real `lib/std/**/*.lex` files for `std::` packages, generated
  `lib/std/native/*.lex` signature stubs for native builtins (shipped in
  the SDK; never import, bodies abort). Resolution mirrors the
  interpreter's module roots, so it lands where the code runs from.
- `lex ide extension`: `editors/vscode-lex` 1.1.2 (TextMate grammar,
  snippets, `lex` task type, `$lexicon-*` matchers).

## [0.3.0] - 2026-10-04

### Runtime (faster)
- ASCII fast paths in `Text::slice/code_at/len` + `char_at` (no alloc/decode walk; regexp ~31% faster).
- In-place string append (`x = x + y` no longer clones the buffer; O(n²) → O(n)).
- `Rc`-shared function/struct tables (no deep AST clone per call).
- Exact i64 fast path kept (no f64 round-trip past 2^53).
- Pipeline: fake sleeps removed from `lex ffi`/`lex deploy` (~1s/2.3s → ms); progress bars hidden in CI/non-TTY.

### Language
- `unsafe::` accepted as a module key (parser lookahead, same precedent as `panic`/`recover`/`new`).
- New natives: `Process::output`, `Atomic::*`, `Hash::crc64/maphash`, `Sys::{goos,arch,ncpu}`, `Json::valid`, `Rand::seed`.
- New stdlib: `os/exec`, `os/signal`, `os/user`, `sync/atomic`, `net/url`, `net/mail`, `net/textproto`, `mime`, `mime/multipart`, `slog`, `flag`, `expvar`, `encoding/base32`, `encoding/ascii85`, `encoding/pem`, `math/bits`, `unicode/utf16`, `image/color`, `archive/tar`, `hash/crc64`, `hash/maphash`, `crypto/sha256` (FIPS vectors), `crypto/hmac` (RFC 4231), `runtime`, `testing`, `unsafe`.
- Extras: `errors::{Wrap,Unwrap,Join}` + recursive `Is`, `fmt::Errorf`, `slices::{Repeat,Chunk}`, `time::{UnixMicro,ParseDate}`, `strconv::Unquote`, `regexp::QuoteMeta`, `strings::{Cut,CutPrefix,CutSuffix,Trim,TrimLeft,TrimRight}`.
- Editor: `complete.rs` rebuilt on the real surface (13 natives + 45 `std::` packages, true lexer keywords); `lex ide` gains TextMate grammar + `extension` scaffold (`editors/vscode-lex`).
- Install & SDK: `lex install` puts binary + SDK (`~/.lexicon`) on the machine; every run silently installs the SDK on first launch and updates it on version drift; Unix/macOS: `chmod +x`, `.zprofile`/fish PATH support, ETXTBSY-safe self-update, `LEX_PATH` via platform split.

## [0.2.0] - 2026-09-29

Implementation baseline.

### Added / Fixed
- Hot-reload supervisor (`lex run --watch`): out-of-process supervised child, debounce, `.lex`-only filter, quick-death backoff.
- Comment-accurate runner: comment stripping in runner/lint/security so commented-out code never executes, registers routes, or raises findings; line numbers preserved.
- Installer: PATH dedup + silent auto-install (CI-aware skip via `CI=true` / `LEXICON_NO_AUTO_INSTALL=1`).
- Linking: webview dependency removal for MinGW linking.
- VS Code super extension 1.1.x.
- demo-api reference project.

## [0.1.0]

- Foundational specification baseline (`Advanced_Programming_Language_Specification_v0.1.md`).
- Initial `lex` toolchain: build/run/test/bench/check/fmt/lint/vet/doc/trace/profile/debug scaffolding.
