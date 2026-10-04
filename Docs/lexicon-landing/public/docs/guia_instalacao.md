# Guia de Instalação e Configuração — Lexicon v0.3.5

Tudo abaixo foi executado contra o toolchain `0.3.5`. Se um comando não
existir, ele não está nesta página.

## 1. Pré-requisitos

- Windows 10/11 x64 (binário pronto), **ou** Linux/macOS com Rust (`cargo`) e
  `git` para compilar do fonte.
- ~500 MB livres no disco. O toolchain é **um binário de 9,4 MB**: não há
  runtime, JVM nem Node para instalar.
- Internet só para baixar. Depois disso o `lex` roda 100% local.

## 2. O que existe para baixar

Os artefatos vivem nos **assets da release** do repositório
[`alguemqualquer123/vantor-lenguage`](https://github.com/alguemqualquer123/vantor-lenguage/releases).
O endereço `.../releases/latest/download/<arquivo>` sempre aponta para a
versão mais nova.

| Asset | Conteúdo | Tamanho |
|---|---|---|
| `lex-sdk-0.3.5-windows-x64.zip` | SDK completo: `bin/lex.exe` + 38 shims, `lib/std` (89 pacotes), `examples` (19), `templates` (5), `docs`, `scripts`, `VERSION` | 4,4 MB zip / 9,7 MB extraído |
| `lex-0.3.5-windows-x64.zip` | só a linguagem: um `lex.exe` | 4,3 MB zip / 9,4 MB |
| `install.ps1` | instalador Windows | 3 KB |
| `install.sh` | instalador Linux/macOS (baixa se houver asset da plataforma; senão compila do fonte) | 2,6 KB |
| `lexicon-super-1.2.0.vsix` | super extensão VS Code | 247 KB |
| `checksums.txt` | SHA-256 de todos os assets acima | — |

Os dois zips dão **o mesmo toolchain**. A diferença é que o zip do SDK já vem
com as libs, exemplos e templates extraídos; o zip "só a linguagem" tem um
arquivo e escreve esse mesmo SDK em `~/.lexicon/sdk` na primeira execução.

## 3. Instalação por script (recomendado)

```powershell
# Windows — PowerShell, sem admin
powershell -ExecutionPolicy Bypass -Command "iwr -useb https://github.com/alguemqualquer123/vantor-lenguage/releases/latest/download/install.ps1 | iex"
```

```bash
# Linux e macOS
curl -fsSL https://github.com/alguemqualquer123/vantor-lenguage/releases/latest/download/install.sh | sh
```

O script baixa, extrai num diretório temporário e chama `lex install`, que:

1. copia o binário para `~/.lexicon/bin` (`.lexicon\bin` no Windows);
2. cria os 38 launchers (`lex-run`, `lex-check`, `lex-mod`, …) — shims de ~40
   bytes que repassam o subcomando para o binário vizinho;
3. adiciona `~/.lexicon/bin` ao PATH **só se ele não estiver lá** (checa o
   processo e o registro persistente — sem duplicatas);
4. exporta o SDK em `~/.lexicon/sdk`.

## 4. Instalação manual

1. Baixe e extraia o zip onde quiser.
2. Rode `lex.exe install` (ou `./lex install`).
3. **Abra um terminal novo** — a entrada de PATH gravada no registro só vale
   para sessões novas.
4. Confira: `lex version`.

## 5. Auto-install silencioso

Todo `lex` que roda faz o mesmo reconciliador da seção 3 em silêncio: copia o
binário se ele mudou, garante a entrada no PATH e atualiza o SDK quando o
arquivo `~/.lexicon/sdk/VERSION` diverge da toolchain em execução. Nenhuma
saída, nenhuma duplicata.

Para pular (CI, benchmarks, medição de tempo de execução):

```bash
CI=true lex run main.lex
# ou
LEXICON_NO_AUTO_INSTALL=1 lex run main.lex
```

## 6. Verificando a instalação

```bash
lex version        # lex 0.3.5 (lexc 0.3.5, spec v0.1)
lex sdk verify     # confere os arquivos obrigatórios do SDK instalado
lex sdk info       # sabor, caminho e tamanho do binário
```

`lex sdk verify` imprime `SDK verify OK at <dir> (116 files)` quando o kit
está completo, e reprova se algum pacote, exemplo ou template sumir.

Agora rode algo de verdade (no Windows o SDK instalado está em
`%USERPROFILE%\.lexicon\sdk`; nos demais, `~/.lexicon/sdk`):

```bash
lex run <sdk>/examples/fizzbuzz.lex
lex vet <sdk>/examples/fizzbuzz.lex   # esperado: sem violações
lex check <sdk>/examples/pong.lex     # esperado: Type checking passed!
```

## 7. VS Code

Opção A — super extensão (`lexicon-team.lexicon-super` 1.2.0):

```bash
code --install-extension lexicon-super-1.2.0.vsix
```

Ela traz a gramática, 117 snippets, 5 temas, autocomplete contextual com
auto-import e os diagnósticos do `lex vet` ao salvar.

Opção B — sem extensão, scaffold do toolchain:

```bash
lex ide init     # gera .vscode/tasks.json + snippets + LEX-TOOLS.md
```

Isso cria tasks para `lex run --ci`, `lex run --watch` (hot reload), `lex test`
e `lex check` no arquivo atual, e documenta como apontar qualquer cliente LSP
genérico para `lex lsp` (ou o depurador para `lex dap`).

## 8. Compilar do fonte (e gerar o seu próprio release)

```bash
git clone https://github.com/alguemqualquer123/vantor-lenguage.git
cd vantor-lenguage
cargo build --profile dist -p lexicon-cli     # opt-level=z + LTO gorda
./target/dist/lex install                      # PATH + SDK
```

Para empacotar os mesmos artefatos desta página:

```powershell
powershell -ExecutionPolicy Bypass -File release.ps1
```

Ele roda o build, faz `lex sdk export`, `lex sdk verify`, zipa os dois sabores
e escreve `checksums.txt` em `build/release/`.

## 9. Removendo

```bash
lex uninstall    # tira o binário, os launchers e a entrada do PATH
```

Reabra o terminal depois. O PATH do Windows é gravado em
`HKCU\Environment`; `lex install` é idempotente e diz exatamente o que
faltava quando você roda de novo.
