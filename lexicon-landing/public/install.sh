#!/usr/bin/env bash
# install.sh — instala o Lexicon em Linux/macOS a partir do release do GitHub.
#
#   curl -fsSL https://github.com/<repo>/releases/latest/download/install.sh | sh
#
// Se existir um artefato compilado para esta plataforma no release, ele é
// baixado e extraído. Se não existir (hoje o release publica binários
// windows-x64), o script compila a partir do fonte com cargo — mesma versão,
// só que na sua máquina. Nos dois casos o passo final é `lex install`, que
// coloca ~/.lexicon/bin no PATH e exporta o SDK em ~/.lexicon/sdk.
set -eu

REPO="${LEX_REPO:-alguemqualquer123/vantor-lenguage}"
FLAVOR="${LEX_FLAVOR:-sdk}"   # sdk | lang
API="https://api.github.com/repos/${REPO}/releases/latest"

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$os" in linux|darwin) ;; *) os="linux" ;; esac
arch="$(uname -m)"
case "$arch" in x86_64) arch="x64" ;; aarch64|arm64) arch="arm64" ;; esac

asset_prefix() { [ "$FLAVOR" = "sdk" ] && echo "lex-sdk" || echo "lex"; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

say() { printf '%s\n' "$*"; }

# 1. Existe binário pronto para esta plataforma no release mais recente?
json="$(curl -fsSL -H 'User-Agent: lexicon-installer' "$API" 2>/dev/null || true)"
url="$(printf '%s' "$json" | grep -o '"browser_download_url": *"[^"]*"' \
        | cut -d'"' -f4 \
        | grep -E "/$(asset_prefix)-[0-9.]+-${os}-${arch}\.(tar\.gz|zip)$" \
        | head -n1 || true)"

if [ -n "$url" ]; then
    say "Baixando $url"
    pkg="$tmp/$(basename "$url")"
    curl -fL --proto '=https' --tlsv1.2 -o "$pkg" "$url"
    case "$pkg" in
        *.tar.gz) tar -xzf "$pkg" -C "$tmp" ;;
        *.zip)    unzip -q "$pkg" -d "$tmp" ;;
    esac
else
    # 2. Sem binário da plataforma: compila do fonte.
    say "Nenhum artefato ${os}-${arch} no release — compilando a partir do fonte."
    say "Requisitos: git, curl e cargo (rustup.rs)."
    command -v cargo >/dev/null || { say "erro: cargo não encontrado. Instale o Rust: https://rustup.rs"; exit 1; }
    command -v git >/dev/null || { say "erro: git não encontrado."; exit 1; }
    git clone --depth 1 "https://github.com/${REPO}.git" "$tmp/src"
    (cd "$tmp/src" && cargo build --profile dist -p lexicon-cli)
    lex_bin="$tmp/src/target/dist/lex"
fi

lex_bin="${lex_bin:-$(find "$tmp" -type f -name lex | head -n1)}"
[ -n "$lex_bin" ] || { say "erro: não encontrei o binário 'lex' baixado."; exit 1; }
chmod +x "$lex_bin"

"$lex_bin" install
say ""
say "Lexicon instalado. Abra um terminal novo e rode:"
say "    lex version"
say "    lex run examples/hello.lex"
