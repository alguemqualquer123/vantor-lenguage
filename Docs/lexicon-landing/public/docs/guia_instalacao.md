# Guia de Instalação e Configuração - Lexicon SDK

## 1. Pré-requisitos
- Sistema Operacional: Windows 10+, Linux (Ubuntu 20.04+), macOS (Catalina+).
- Espaço em Disco: 500 MB.
- Conexão de Internet: Necessária para download de pacotes e deploy.

## 2. Instalação Automática (Recomendado)
A maneira mais rápida de configurar o ambiente Lexicon.

### Windows (PowerShell)
Execute no terminal PowerShell como Administrador:
```powershell
iwr https://get.lexicon.dev/install.ps1 -useb | iex
```

### Linux e macOS (Terminal)
Execute no seu terminal Bash ou Zsh:
```bash
curl -fsSL https://get.lexicon.dev | sh
```

## 3. Instalação Manual
### Passo 1: Download
Acesse a seção de **Downloads** no portal oficial (`http://localhost:3002`) e baixe o instalador adequado:
- **Windows**: `lexicon-sdk-win64.msi`
- **Linux**: `lexicon-sdk-linux.tar.gz`
- **macOS**: `lexicon-sdk-macos.pkg`

### Passo 2: Extração e PATH
Após o download, extraia o arquivo e adicione o diretório `bin` ao seu PATH do sistema.

#### No Windows:
1. Abra as Variáveis de Ambiente.
2. Edite a variável `Path`.
3. Adicione o caminho completo da pasta `bin`.

#### No Linux/macOS:
Adicione ao seu `.bashrc` ou `.zshrc`:
```bash
export PATH=$PATH:/caminho/para/lexicon/bin
```

## 4. Configuração do VS Code
Para a melhor experiência de desenvolvimento:
1. Abra o VS Code.
2. Vá em **Extensões** (Ctrl+Shift+X).
3. Procure por **Lexicon Language Support**.
4. Instale e reinicie o VS Code.

## 5. Verificação da Instalação
No terminal, execute:
```bash
lex --version
```
Se o comando retornar a versão (ex: `v0.1.0-alpha`), a instalação foi concluída com sucesso.
