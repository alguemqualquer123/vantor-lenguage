#!/bin/bash

# Script de compilação multi-plataforma (Windows/Linux/macOS)
# Requisitos: Node.js (vsce), Rust (cargo)

echo "--- Lexicon Build System ---"

# 1. Compilar o CLI (Rust)
echo "[1/3] Compilando CLI do Lexicon..."
cargo build --release
if [ $? -eq 0 ]; then
    echo "CLI compilado com sucesso."
else
    echo "Erro ao compilar CLI."
fi

# 2. Empacotar extensões VS Code (.vsix)
echo "[2/3] Empacotando extensões VS Code..."

# Extensão Principal
cd themes/lexicon-vscode
npx @vscode/vsce package --no-dependencies --skip-license --allow-missing-repository
cd ../..

# Tema Pro
cd themes/lexicon-dark-pro
npx @vscode/vsce package --no-dependencies --skip-license --allow-missing-repository
cd ../..

# 3. Finalização
echo "[3/3] Build completo!"
echo "Binários do CLI: target/release/"
echo "Extensões VS Code: src/lexicon-vscode/ e src/lexicon-dark-pro/"
