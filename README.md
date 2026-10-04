<div align="center">

<img src="lexicon-landing/public/mascote_lex_lang_256.png" width="220" alt="Lex, o axolote roxo que programa">

# Lex · Lexicon

**Code beyond limits** — uma linguagem com toolchain próprio (compilador em Rust +
interpretador de verdade), sintaxe amigável, **89 pacotes** na biblioteca padrão,
**motor gráfico de jogos** e um **gerenciador de pacotes** — tudo cabendo em
**um único binário de ~9 MB**.

[![versão](https://img.shields.io/badge/vers%C3%A3o-0.3.5-7c3aed)](https://github.com/alguemqualquer123/vantor-lenguage/releases/tag/v0.3.5)
[![testes](https://img.shields.io/badge/e2e-148%20%7C%20148-22c55e)](tests)
[![plataforma](https://img.shields.io/badge/Windows%20%C2%B7%20Linux%20%C2%B7%20macOS-38bdf8)](Docs/README.md)

</div>

---

## 🧐 Espera… o que é o Lex?

Imagina uma linguagem que fala a sua língua. Você escreve `Console::writeLine("oi")`,
joga valores dentro de tubos com `|>`, chama bibliotecas com `import std::...` e, quando
quer fazer um jogo, abre uma janela **de verdade** sem instalar nada.

O **Lexicon** (carinhosamente **Lex**) é exatamente isso: um toolchain completo escrito em
Rust com **13 crates**, que traz compilador, interpretador, formatador, linter, servidor de
linguagem, depurador, gerenciador de pacotes e motor gráfico dentro do mesmo `lex`.

```text
você  ──▶  lex run app.lex  ──▶  🦎 interpreta / executa  ──▶  stdout, janelas, jogos
```

---

## 🚀 Comece em 60 segundos

Primeiro o `lex` — dois sabores na [página de releases](https://github.com/alguemqualquer123/vantor-lenguage/releases),
o mesmo toolchain nos dois:

```powershell
# Windows — SDK completo ou só o binário; não precisa de admin
powershell -ExecutionPolicy Bypass -Command "iwr -useb https://github.com/alguemqualquer123/vantor-lenguage/releases/latest/download/install.ps1 | iex"
```

```bash
# Linux / macOS — baixa se houver artefato da plataforma, senão compila do fonte
curl -fsSL https://github.com/alguemqualquer123/vantor-lenguage/releases/latest/download/install.sh | sh
```

```bash
# ou compile você mesmo (perfil dist: opt-level=z + LTO gorda)
cargo build --profile dist -p lexicon-cli && ./target/dist/lex install
```

Depois:

```bash
lex new meu-projeto      # cria a estrutura (templates: default, api, plugin, service, gui)
cd meu-projeto
lex run src/main.lex     # roda na hora
```

O arquivo mais honesto do mundo:

```lex
pub fn main() -> void {
    Console::writeLine("Olá, Lexicon!");
}
```

> Sem ponto e vírgula para esquecer, sem 400 MB de `node_modules`.
> Um arquivo, uma função, um grito.

---

## 🧙 Um passeio rápido pela sintaxe

### Funções e tubos (`|>`)

O operador `|>` pega o valor da esquerda e o empurra como primeiro argumento da função da
direita. Nada de ler de dentro para fora:

```lex
fn dobrar(n: i64) -> i64 { return n * 2; }
fn somar_um(n: i64) -> i64 { return n + 1; }

import std::strings;

pub fn main() -> void {
    let r = 5 |> dobrar |> somar_um;          // 11
    Console::writeLine("pipe = " + r);
    Console::writeLine("lex" |> strings::ToUpper);   // LEX
}
```

### Estruturas, laços e asserções

```lex
pub struct Usuario {
    nome: String,
    nivel: i64,
}

pub fn main() -> void {
    let u = Usuario { nome: "Lex", nivel: 3 };
    Console::writeLine(u.nome + " nivel " + u.nivel);

    let i = 1;
    while i <= 3 {
        assert(i > 0, "sempre positivo");
        i = i + 1;
    }
}
```

### Bibliotecas padrão

```lex
import std::crypto::sha256;

pub fn main() -> void {
    Console::writeLine(sha256::Sum("lexicon"));
    // 239aec50c94b6ea398dadb908b531bbe124c08fbbe756ce60cd59c37cfd719f2
}
```

### Reflexão e asserções

```lex
pub fn main() -> void {
    assert(typeOf(1) == "int", "inteiros têm nome");
    assert("s".type() == "String", "strings também");
    Console::writeLine("tudo certo em " + typeOf([1]));
}
```

---

## 🎮 Motor gráfico: janelas e jogos sem sair do `lex`

Sim, a mesma ferramenta que formata o seu código abre uma janela e desenha frames. O
renderizador é **wgpu** (o mesmo motor gráfico do Firefox), então o backend é escolhido
pela sua placa: **Vulkan**, **DirectX 12**, **OpenGL**, **Metal** ou **WebGPU**.

```lex
pub fn main() -> void {
    let win = Window::create(Title { title: "Minha janela", width: 640, height: 400 });

    while !Window::shouldClose(win) {
        Canvas::clear(win, 0.08, 0.06, 0.15, 1.0);
        Canvas::fillCircle(win, 320.0, 200.0, 60.0, 0.49, 0.23, 0.95, 1.0);
        Canvas::text(win, 20.0, 20.0, Text { s: "backend: " + Window::backend(win), size: 20.0 }, 1.0, 1.0, 1.0, 1.0);
        Window::present(win);
    }
}
```

| Nativo | O que faz |
|---|---|
| `Window` | `create`, `shouldClose`, `poll`, `present`, `close`, `setTitle`, `setSize`, `backend` |
| `Canvas` | `clear`, `fillRect`, `fillCircle`, `line`, `text` — desenho 2D imediato |
| `Input` | `keyDown`, `mouseX`, `mouseY`, `mouseDown`, `clicked`, `value`, `checked` |
| `Gpu` | `backend()` — qual driver a placa escolheu |
| Widgets | `Label`, `Button`, `TextField`, `Checkbox`, `MenuBar`, `Menu`, `MenuItem` |

🕹️ **Rode o Pong completo do repo** (raquetes, CPU, placar) com:

```bash
lex run examples/pong.lex
```

Prefere começar do zero? O template `gui` já traz a janela, o loop e o input
prontos:

```bash
lex new minha-janela -t gui && cd minha-janela && lex run src/main.lex
```

Em CI, `lex run --ci` fecha a janela sozinho depois de ~1 s — pipelines não travam.

---

## 📦 `lex mod`: gerenciador de pacotes (projetos e libs)

```bash
lex mod init meu-lib                       # cria o lexicon.toml
lex mod add github:alice/texto-utils       # baixa, instala e fixa no lock
lex mod add path:../outra-lib              # dependência local, sem servidor
lex mod install                            # restaura tudo do lexicon.lock
lex mod graph                              # árvore de imports do projeto
lex mod verify src/main.lex                # confere se todos os imports resolvem
```

| Comando | Para quê |
|---|---|
| `init` / `add` / `remove` / `tidy` | ciclo de vida das dependências |
| `install` / `list` / `graph` / `verify` | reprodutibilidade e diagnóstico |

Fontes aceitas: `github:owner/repo[@tag|@branch|#sha]`, `gitlab:…`, `https://…`, `path:../dir`.
Os pacotes vivem em `lex_packages/` e as versões ficam presas em `lexicon.lock` — o mesmo
contrato do `go.mod`/`package-lock.json`, em dois arquivos pequenos.

---

## 🧰 O canivete: tudo que o `lex` sabe fazer

| Grupo | Comandos |
|---|---|
| Rodar | `run` (com `--watch` e `--ci`), `repl`, `test`, `bench`, `serve`, `test-net` |
| Qualidade | `check`, `fmt`, `lint`, `vet`, `fix`, `doc`, `trace`, `profile` |
| Projeto | `new`, `init`, `mod`, `publish`, `deploy`, `clean`, `env`, `version` |
| Depuração | `debug`, `trace`, `lsp`, `dap`, `ide` |
| Gráficos | `gui`, `visualize` |
| Interop | `ffi` (bindings C/Rust), `generate` (macros), `dns-check`, `tls-check` |
| SDK | `install`, `uninstall`, `sdk export`, `sdk verify` |

Editor? A extensão VS Code pronta está em [`editors/vscode-lex`](editors/vscode-lex), e
`lex lsp` / `lex dap` falam os protocolos padrão com qualquer IDE moderna.

---

## 📚 89 pacotes na biblioteca padrão

Escritos **na própria Lex** (você pode ler a fonte e aprender), embutidos no SDK e
verificados por `lex check`. A organização segue o modelo do Go — `import std::pacote::sub`.

| Família | Pacotes |
|---|---|
| Texto e dados | `strings` `text/tabwriter` `strconv` `fmt` `bytes` `bufio` `regexp` `html` `unicode/utf8` `unicode/utf16` |
| Estruturas | `slices` `maps` `cmp` `iter` `container/list` `container/heap` `container/ring` `sort` |
| Concorrência e sistema | `sync` `sync/atomic` `sync/pool` `context` `unsafe` `runtime` `os` `os/exec` `os/signal` `os/user` `errors` `log` `slog` `flag` `time` `expvar` |
| Criptografia e hash | `crypto/sha256` `crypto/sha1` `crypto/md5` `crypto/hmac` `crypto/rc4` `crypto/subtle` `hash/crc32` `hash/crc64` `hash/adler32` `hash/fnv` `hash/maphash` |
| Codificação | `encoding/json` `encoding/csv` `encoding/base64` `encoding/base32` `encoding/hex` `encoding/pem` `encoding/ascii85` `encoding/binary` |
| Rede e arquivos | `net/http` `net/url` `net/mail` `net/textproto` `net/netip` `mime` `mime/multipart` `mime/quotedprintable` `path` `path/filepath` `archive/tar` |
| Matemática | `math` `math/bits` `math/cmplx` `rand` |
| Imagem e testes | `image/color` `testing` |
| Nativos do motor | `native/window` `native/canvas` `native/input` `native/gpu` + 14 stubs de assinatura para o editor |

```bash
lex doc lib/std/strings.lex      # documentação gerada dos doc comments
```

---

## 🐣 Por que um binário só (e por que ele é pequeno)

O `lex` é **um** binário: os 38 launchers (`lex-run`, `lex-mod`, `lex-fmt`, …) são *shims*
de ~40 bytes que apenas repassam o subcomando para o vizinho. Nada de 38 cópias de 270 MB.

```bash
lex sdk export      # build/sdk: bin/lex.exe + 38 shims + 89 libs + 19 exemplos + 5 templates + docs
lex sdk verify      # confere arquivos, orçamento de tamanho e se o binário roda
```

| Sabor | Orçamento | Medido |
|---|---|---|
| mini (`--no-default-features`, perfil `dist`) | ≤ 6 MB | ~3,6 MB |
| full (padrão, com GUI) | ≤ 16 MB | ~9,4 MB |

O perfil `dist` (`opt-level = "z"` + LTO gorda) existe justamente para isso, e o
`lex sdk verify` **reprova** qualquer mudança de dependência que estoure o orçamento.
Detalhes em [`Docs/FEATURE_MATRIX.md`](Docs/FEATURE_MATRIX.md).

---

## 🧪 Como a gente sabe que funciona

```bash
bash tests/run_e2e.sh          # 148 programas Lex de ponta a ponta
cargo test -p lexicon-cli      # 71 testes unitários do toolchain
lex check lib/std/strings.lex  # cada pacote da stdlib precisa type-checkar
```

- **148 e2es** em [`tests/`](tests) cobrem linguagem, stdlib, hash vetores RFC, rede, GUI.
- **71 testes Rust** cobrem lexer, parser, typeck, interpretador, SDK, templates e instalador.
- Os vetores de hash saem das RFCs (MD5/SHA-1/SHA-256) e os de `math/cmplx` e `net/netip`
  foram conferidos contra as implementações de referência.

---

## 🗂️ Onde as coisas moram

```text
src/lexicon-cli/        o binário `lex`: run, check, fmt, mod, sdk, gui, lsp, dap…
src/lexicon-lexer|parser|analysis|codegen|core|utils|async|http|cache|gui|wasm|db/
lib/std/                89 pacotes da biblioteca padrão, escritos em Lex
examples/               18 programas comentados (hello → pong → tar_pack)
templates/              default · api · plugin · service · gui (janela wgpu)
tests/                  148 e2es + scripts do harness
Docs/                   planos, matriz de features, especificação e guias
editors/vscode-lex/     extensão para o VS Code
apps/ · vinix/          projetos reais que usam a linguagem
lexicon-landing/        site e documentação pública (Next.js)
```

---

## 🐉 O mascote

**Lex** é um axolote roxo com brilho de código na barriga. Axolotes regeneram membros —
a gente regenera binários. Os arquivos estão em [`lexicon-landing/public/`](lexicon-landing/public),
todos com fundo transparente e recortados no conteúdo:

| Arquivo | Uso |
|---|---|
| `mascote_lex_lang.png` | mascote quadrado (889 px) |
| `mascote_lex_lang_16/24/32/48/64/128/256/512.png` | ícones em todos os tamanhos |
| `favicon.ico` | 16 · 24 · 32 · 48 · 64 |
| `mascote_lex_lang_full.png` | mascote + wordmark |
| `mascote_lex_lang_wordmark.png` | só a tipografia LEX LANG |

---

## 📜 Licença

MIT — veja [`LICENSE-MIT.md`](LICENSE-MIT.md) e o [`CHANGELOG.md`](CHANGELOG.md).

<div align="center">

<img src="lexicon-landing/public/mascote_lex_lang_64.png" width="48" alt="axolote pequeno">

*Feito com Rust, teimosia e um axolote.*

</div>
