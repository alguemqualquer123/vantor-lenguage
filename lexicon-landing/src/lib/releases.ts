// Fonte única de verdade dos downloads do site.
//
// Os nomes dos artefatos são gerados por `release.ps1` (raiz do repositório):
//   lex-sdk-<versão>-windows-x64.zip  → SDK completo
//   lex-<versão>-windows-x64.zip      → só a linguagem
// A versão do nome é a que o próprio binário reporta (`lex version`), então
// mudar aqui sem subir o release resulta em 404 — atualize as duas pontas.
export const REPO = "alguemqualquer123/vantor-lenguage";

export const VERSION = process.env.NEXT_PUBLIC_LEX_VERSION ?? "0.3.5";

export const TAG = `v${VERSION}`;

// A extensão VS Code é versionada à parte (themes/lexicon-vscode) e o
// release.ps1 publica o .vsix mais recente como asset do mesmo release.
export const EXTENSION = "lexicon-super";
export const EXTENSION_VERSION = "1.2.0";

const GH = `https://github.com/${REPO}`;
const LATEST_ASSET = `${GH}/releases/latest/download`;

export const links = {
  repo: GH,
  releases: `${GH}/releases`,
  latestRelease: `${GH}/releases/latest`,
  sdkZip: `${LATEST_ASSET}/lex-sdk-${VERSION}-windows-x64.zip`,
  langZip: `${LATEST_ASSET}/lex-${VERSION}-windows-x64.zip`,
  installPs1: `${LATEST_ASSET}/install.ps1`,
  installSh: `${LATEST_ASSET}/install.sh`,
  extensionVsix: `${LATEST_ASSET}/${EXTENSION}-${EXTENSION_VERSION}.vsix`,
  checksums: `${GH}/releases/latest/download/checksums.txt`,
};

// Tamanhos reais medidos no kit empacotado (zip comprimido / tamanho após
// `lex install`). Nada de número decorativo.
export const flavors = [
  {
    id: "sdk",
    file: `lex-sdk-${VERSION}-windows-x64.zip`,
    zip: "4,4 MB",
    installed: "9,7 MB",
  },
  {
    id: "lang",
    file: `lex-${VERSION}-windows-x64.zip`,
    zip: "4,3 MB",
    installed: "9,4 MB",
  },
] as const;

export const installCommandPs1 =
  `powershell -ExecutionPolicy Bypass -Command "iwr -useb ${links.installPs1} | iex"`;

export const installCommandSh = `curl -fsSL ${links.installSh} | sh`;
