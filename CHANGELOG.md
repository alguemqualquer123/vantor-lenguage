# Changelog

All notable changes to the Lexicon programming language toolchain.

## [0.3.5] - 2026-10-04

### Added
- **`release.ps1`** (raiz): empacota os artefatos de download. Trava o
  auto-bump (`LEX_NO_AUTO_BUMP=1`), lê a versão do **próprio binário**
  (`lex version`), roda `lex sdk export` + `lex sdk verify`, zipa os dois
  sabores e escreve `checksums.txt` em `build/release/`:
  - `lex-sdk-<ver>-windows-x64.zip` — SDK completo (4,4 MB zip / 9,7 MB extraído);
  - `lex-<ver>-windows-x64.zip` — só a linguagem, um `lex.exe` (4,3 MB zip);
  - `install.ps1`, `install.sh` e `lexicon-super-<ver>.vsix` vão como assets.
  Os zips são montados à mão (`ZipFile` + `CreateEntryFromFile`) para as
  entradas saírem com `/` — `CreateFromDirectory` gravava `\` e quebrava
  quem descompacta em Linux/macOS.
- **Instaladores reais** em `lexicon-landing/public/`: `install.ps1` resolve o
  asset pelo GitHub API (`releases/latest`), baixa, extrai em pasta temporária
  e chama `lex install`; `install.sh` faz o mesmo e, quando não existe artefato
  da plataforma, compila do fonte (`cargo build --profile dist`) antes do mesmo
  `lex install`. Nenhum dos dois precisa de admin.
- **Landing atualizada inteira**: `src/lib/releases.ts` é a única fonte de
  repo/versão/URLs. A seção *Download* agora oferece os dois sabores com
  tamanho real, lista do que vem em cada um, comandos de instalação com botão
  de copiar, tabela de plataformas honesta (Windows = binário pronto;
  Linux/macOS = `install.sh`; WASM = em breve), checksums e link de releases.
  Card do template `gui`, nav mobile (docs + download) e `og-image.png` novo.

### Fixed
- `Cargo.toml` volta para **0.3.5**, a versão que o binário `dist` realmente
  reporta — o auto-bump do `build.rs` roda *antes* do `CARGO_PKG_VERSION` ser
  lido, então o manifesto ficava um patch à frente do que é publicado.
- Landing mentia: botões apontando para `#`, repo `lexicon-team/lexicon`,
  `get.lexicon.dev`, `.msi`/`.pkg`/`.wasm` inexistentes e versão `v0.2.0`.
- Trecho de código do hero agora é um programa Lex verificado (`import
  std::strings` + `std::crypto::sha256`, pipe qualificado, `sha256::Sum`) que
  roda com `lex run` sem cair em fallback de mock — a saída exibida é a saída real.

### Docs
- `guia_instalacao.md` (PT/EN) reescrito com os assets da release, tabela de
  tamanhos, instalação por script/manual, `lex sdk verify`, e os limites
  reais; `manual_usuario.md`, `documentacao_tecnica.md` e
  `troubleshooting_glossario.md` (PT/EN) vão para v0.3.5 com a seção
  "um binário, um SDK" + motor gráfico wgpu.
- Números da extensão conferidos contra `themes/lexicon-vscode/package.json`:
  `lexicon-team.lexicon-super` **1.2.0**, 117 snippets, 4 temas + icon theme.

### Tests
- `bash tests/run_e2e.sh` → **PASS 148 / FAIL 0** (inclui `e2e_148.lex`, o
  rasterizador: winding, backface culling, pesos baricêntricos e Gouraud);
  `cargo test -p lexicon-cli` → **71/71**.
- `npx tsc --noEmit` limpo e `npm run build` OK na landing; `/`, `/docs`,
  `/docs/*.md`, `/install.ps1` e `/og-image.png` respondem 200 no dev server.
- Chaves `en`/`pt` de `translations.ts` comparadas por script: 94 × 94, sem
  diferença.
- `release.ps1 -SkipBuild` roda de ponta a ponta; os dois zips extraem e o
  `lex.exe` extraído do SDK zip roda (`lex version`, `lex run` de um exemplo).

## [0.3.4] - 2026-10-04

### Added
- **Quarta onda Go-parity na stdlib** — 3 pacotes novos escritos em Lex e
  validados contra as implementações de referência:
  - `lib/std/crypto/rc4.lex` — `Keystream`/`Cipher` + `Hex`/`HexLower`
    (vetores oficiais da nota aplicatória RC4 do RFC 6229:
    `Key`/`Plaintext` → `BBF316E8D940AF0AD3`).
  - `lib/std/math/cmplx.lex` — `Complex{re,im}` com `Abs/Phase/Add/Sub/Neg/
    Scale/Mul/Div/Conj/Sqrt/Exp/Log/Log2/Log10/Pow/PowInt/Sin/Cos/Tan`
    (Smith para a divisão, C99 Annex G para a raiz; conferido contra `cmath`).
  - `lib/std/net/netip.lex` — `NetIP`/`NetPrefix`/`NetAddrPort`: parser e
    formatter IPv4/IPv6 (RFC 5952), `As16/FromBytes`, `ParsePrefix/Masked/
    Contains`, `ParseAddrPort`. Desvios documentados no cabeçalho:
    `::ffff:a.b.c.d` é Unmapped para IPv4 puro e zones (`fe80::1%eth0`) são
    rejeitadas.
- **8 pacotes que já existiam em `lib/std` mas NÃO eram embarcados no SDK**
  passam a ser exportados: `crypto/md5`, `crypto/sha1`,
  `mime/quotedprintable`, `sync/pool`, `text/tabwriter`, `encoding/binary`,
  `net/http` e os stubs de assinatura `native/{window,canvas,input,gpu}`.
  O SDK passa de 75 para **89 arquivos `.lex`** de stdlib.
- `README.md` na raiz: apresentação da linguagem com o mascote, tour de
  sintaxe, `lex mod`, motor gráfico, tabela de comandos, stdlib, orçamento
  de binário e como rodar a suíte.
- **Mascote em todos os tamanhos** (`lexicon-landing/public/`), recortado no
  conteúdo e sem fundo: `mascote_lex_lang.png` (889 px), `_16/_24/_32/_48/
  _64/_128/_256/_512.png`, `favicon.ico` (16·24·32·48·64),
  `mascote_lex_lang_full.png` (mascote + wordmark) e
  `mascote_lex_lang_wordmark.png`.
- `LICENSE-MIT.md` na raiz (mesmo texto que o SDK já embarca).
- **Template `gui`** (`lex new -t gui`): projeto com janela real, loop,
  `Input::keyDown` e `Canvas::*` — parseia, type-checka e roda (validado com
  `lex run --ci`). Teste unitário novo (`gui_template_opens_a_window`).
- **SDK agora embarca os exemplos de verdade**: os 18 arquivos de
  `examples/*.lex` (incluindo `pong.lex`) são `include_str!` no binário e
  escritos em `examples/` no `lex sdk export`, mais o `api.lex` do SDK.
  Também faltava o template `plugin` na exportação — são **5 templates**
  (`default`, `api`, `service`, `plugin`, `gui`) e **19 exemplos**.
- Testes: e2e **141–147** (hash legacy, cmplx, netip, quotedprintable +
  tabwriter, pool + binary, coexistência de pacotes, pipes).

### Fixed
- **Interpretador: símbolos de um pacote não são mais sombreados pelos de
  outro.** `Cx` ganhou `cur_mod` e as resoluções passaram por `user_fn` /
  `user_struct` / `call_user_home`, então dentro de `crypto/md5.lex` uma
  chamada não qualificada resolve o **próprio** pacote primeiro e só depois a
  tabela achatada global. Antes, `sha1::Sum` devolvia digest MD5 e
  `rc4::Hex` devolvia minúsculas quando os dois pacotes eram importados
  (regressão coberta por `tests/e2e_146.lex`).
- `net/textproto::ReadMIMEHeader` perdia o **último** header quando o bloco
  não terminava em linha em branco — o que derrubava todos os parts de
  `mime/multipart`.
- `encoding/binary::PutBE64` escrevia o byte mais significativo como
  `v / 2^72` (sempre 0); agora usa as potências corretas e round-tripa.
- Pipes `|>` aceitam alvo qualificado (`"lex" |> strings::ToUpper`) e alvos
  que só existem como builtin/nativo; antes caía em
  `pipe into non-function` e o programa inteiro silenciosamente virava o
  fallback de mock.
- `build.rs` do auto-bump corrompia o manifesto a cada build de release
  (`version = "0.3.3"` → `version version = "0.3.4"`), quebrando o build
  seguinte. A substituição agora preserva só a indentação.
- **Tamanho do SDK**: `lex sdk export` publica **um** binário + 38 launchers
  como *shims* de ~40 bytes (antes: 39 hardlinks do binário **debug** de
  272 MB). `bin/lex.exe` sai do perfil `dist` (9,4 MB, orçamento 16 MB) e o
  `lex sdk verify` mede e reprova fora do budget.
- Os launchers sem extensão do SDK (`lex-run`, para Git Bash/MSYS) recebiam
  o corpo `.cmd` (`@echo off`); agora cada shim tem o corpo do seu shell
  (`#!/bin/sh` + `exec "$(dirname "$0")/lex.exe" run "$@"`).

### Tests
- `bash tests/run_e2e.sh` → **PASS 147 / FAIL 0**; `cargo test -p
  lexicon-cli` → **71/71**; `lex sdk verify` OK.
- Os 19 exemplos exportados pelo SDK rodam sem erro em `lex run --ci`
  (`pong.lex` inclusive, com janela real em Vulkan).

## [0.3.2] - 2026-10-04

### Added
- **Motor gráfico nativo (jogos/janelas/programas)** — `Window::create` deixa
  de ser mock: janelas REAIS no eframe+**wgpu** com backends **Vulkan /
  DirectX 12 / OpenGL / Metal** (o mesmo motor gráfico do Firefox; o backend
  é escolhido pela máquina e reportado por `Window::backend(win)`). Feature
  `gui` (default) liga tudo; builds `--no-default-features` caem no mock
  antigo.
  - Nativos novos: `Window` (create/shouldClose/poll/present/close/setTitle/
    setSize/width/height/backend/show), `Canvas` (clear/fill_rect/fill_circle/
    line/text — desenho 2D imediato, origem no topo-esquerdo), `Input`
    (keyDown/mouseX/mouseY/mouseDown/clicked/value/checked) e `Gpu::backend`.
  - Widgets + menus: `Label/Button/TextField/Checkbox` (create +
    setText/setPlaceholder/setChecked), `MenuBar/Menu/MenuItem` (new + add) —
    `window.add(...)`, `window.setMenuBar(...)`; estilo do app
    `apps/minha-janela` agora roda de verdade (2 janelas, sem mock).
  - Modelo: **host único eframe + uma viewport egui por janela** (o winit
    0.30 só permite UM event loop por processo — `EVENT_LOOP_CREATED` é
    global; a 2ª `build()` devolve `RecreationAttempt`). Interpretador ⇄
    host por canal+condvar; viewports são re-declaradas a cada pass (o egui
    poda filhas não re-declaradas) com repaint explícito (`request_repaint_of`);
    `Window::present` faz o pacing; o processo fica vivo até todas as
    janelas fecharem; `LEXICON_GUI_AUTOQUIT=<frames>` fecha sozinho (usado
    por `lex run --ci`, que agora nunca bloqueia pipelines).
  - Demo: `examples/pong.lex` (Pong completo: raquetes, CPU, placar,
    colisões) — `lex run examples/pong.lex`.
  - janelas fora da main thread: `EventLoopBuilderExtWindows::with_any_thread`
    no Windows (limite: macOS usa `lex gui`, documentado).
- **Gerenciador de pacotes** — `lex mod` vira o `go get` do Lex:
  `init`/`add`/`install`/`remove`/`tidy`/`list`/`graph`/`verify`.
  - Fontes: `github:owner/repo[@tag|@branch|#sha]`, `gitlab:…`, URLs git
    (`https://…`, `git@…`, `file://…`) e `path:../dir` (shorthand
    `owner/repo` = GitHub).
  - Cache global `~/.lexicon/pkg/<owner>_<repo>@<ref>` + materialização em
    `lex_packages/<pkg>/` (código em `src/` é achatado → `import pkg::x;`);
    transitive-safe (ciclos são detectados), `git` CLI como backend.
  - **`lexicon.lock`**: fonte + ref + commit + data → builds reproduzíveis;
    `lex mod install` restaura do lock, `tidy` sincroniza toml/lock/disco.
  - Loader (`interp::find_module_file`) passa a resolver imports dentro de
    `lex_packages/` (walk-up a partir do arquivo que importa, com prioridade
    sobre a SDK) — pacotes baixados funcionam sem configuração extra.
- Completions/LSP: `Window/Canvas/Input/Gpu + 7 widgets` no registro de
  autocomplete e stubs de assinatura `lib/std/native/{window,canvas,input,
  gpu}.lex` (go-to-definition).
- Testes: 8 unit tests de `pkg.rs` (fontes, split de ref, manifesto, nomes);
  suíte run e2e **140/140**.

### Fixed
- `Cargo.toml` do workspace com linha quebrada (`version version = "0.3.2"`)
  que impedia QUALQUER build/teste do toolchain.

## [0.3.1] - 2026-10-04

### Added
- Runtime reflection: `typeOf(x)` (+ aliases `typeof`, `type`; method
  forms `x.type()`, `x.typeOf()`) reporting precise names (`int`,
  `String`, `List`, …) and declared names for structs — fulfilling the
  `type(x)`-as-`typeOf` alias the parser always promised.
- Tool launchers: `lex-<tool>` busybox hardlinks (`lex-run`, `lex-lsp`,
  `lex-version`, … 38 total) shipped in `sdk/bin` and `~/.lexicon/bin`,
  synced on install/update, verified by `lex sdk verify`, removed by
  `lex uninstall`.

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
