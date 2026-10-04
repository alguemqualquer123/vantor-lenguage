# LexLang Feature-Gate Matrix (Onda 0.5)

Regra: dependência pesada nova entra **atrás de feature**; o binário mini
(`--no-default-features`, perfil `dist`) tem budget afixado e checado por
`lex sdk verify`.

## Gates existentes

| Feature | Default | O que liga | Custo aprox. no binário |
|---|---|---|---|
| `gui` | ON | `eframe`/`egui_extras`, `lex gui` IDE | ~2.6 MB (6.06 → 3.43 MB ao desligar) |

## Gates planejados (Ondas 1–4)

| Feature | Default | O que vai ligar |
|---|---|---|
| `rt` | ON (full) | executor tokio real em `lexicon-async` (Onda 1A) |
| `jit` | OFF | backend Cranelift em `lexicon-utils/src/vm.rs` (Onda 1D) |
| `sign` | ON (full) | `ed25519-dalek` (Onda 1B); sem ela, stub fails-closed |
| `codecs` | ON (full) | MessagePack/CBOR/FlatBuffers completos (Onda 1B) |
| `tls` | OFF | `tokio-rustls` para `lexicon-http` (Onda 1C) |
| `pg` / `brokers` | ON/OFF | drivers `sqlx` não-sqlite, cliente NATS (Onda 1E) |
| `wgpu` | OFF | renderer real `lexicon-gui` (Onda 4.8) |

## Budgets do binário (perfil `dist`, Windows x64)

| Flavour | Budget | Última medição |
|---|---|---|
| mini (`--no-default-features`) | **≤ 6 MB** (falha o `verify` acima) | 3.6 MB |
| full (default) | ≤ 16 MB (falha o `verify` acima) | ~6–8 MB |

Cheque: `lex sdk export && lex sdk verify` — o `verify` mede `bin/lex*`
e reprova fora do budget. CI deve rodar esse par a cada mudança de
dependência.

## Notas v0.2.0 (verificado em código)

- **Dep `webview` removida (2026-09-29)** (`src/lexicon-cli/Cargo.toml`):
  nenhum código Rust a referencia (`webview.rs` é dead code desligado;
  os caminhos do runner estão stubados/comentados em `compiler.rs`) e a
  lib C não linkava no MinGW-GNU (símbolos `CLSID_`/`IID_` indefinidos —
  ver `ole32` em `src/lexicon-cli/build.rs`), o que quebrava **todo**
  `cargo build`/`cargo test` do binário. Restaurar só em toolchain
  MSVC, se o WebView nativo voltar. O gate `gui` (eframe) acima segue
  inalterado.
- **Hot reload não é gate**: `lex run --watch` é o supervisor em
  `compiler.rs::watch` (filho `lex run` + debounce 300 ms + filtro
  só-`.lex` + backoff após 3 mortes rápidas) — sem dependência nova.
- **Runner/lint/security comment-accurate**: `strip_comments` /
  `code_contains` (`src/lexicon-lexer/src/comments.rs`) com testes
  unitários; `run`, `lint` e `analyze_security_source` operam sobre o
  código sem comentários.
