# install.ps1 — instala o Lexicon no Windows a partir do release do GitHub.
#
#   powershell -ExecutionPolicy Bypass -File install.ps1
#   powershell -ExecutionPolicy Bypass -File install.ps1 -Flavor lang
#
# O que acontece: baixa o .zip do release, extrai num diretório temporário e
# chama `lex install`, que copia o binário para %USERPROFILE%\.lexicon\bin,
# cria os 38 launchers, adiciona o PATH (sem duplicar) e exporta o SDK em
# %USERPROFILE%\.lexicon\sdk. Não precisa de admin.
param(
    [ValidateSet("sdk", "lang")]
    [string]$Flavor = "sdk",
    [string]$Version = "",
    [string]$Repo = "alguemqualquer123/vantor-lenguage"
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.IO.Compression.FileSystem

$asset = if ($Flavor -eq "sdk") { "lex-sdk" } else { "lex" }
$release = if ($Version) { "download/v$Version" } else { "latest/download" }
$url = "https://github.com/$Repo/releases/$release/${asset}-*-windows-x64.zip"

# O nome do artefato carrega a versão, então para "latest" deixamos o GitHub
# resolver o redirect e lemos o nome final do asset da página de release.
if (-not $Version) {
    $api = "https://api.github.com/repos/$Repo/releases/latest"
    try {
        $rel = Invoke-RestMethod -Uri $api -Headers @{ "User-Agent" = "lexicon-installer" }
        $found = $rel.assets | Where-Object { $_.name -like "${asset}-*-windows-x64.zip" } | Select-Object -First 1
        if (-not $found) { throw "nenhum asset ${asset}-*-windows-x64.zip no release $($rel.tag_name)." }
        $url = $found.browser_download_url
        $Version = $rel.tag_name.TrimStart("v")
    } catch {
        Write-Error "Não consegui resolver o release mais recente de $Repo. Ele existe? (`$_`)"
        throw
    }
} else {
    $url = "https://github.com/$Repo/releases/download/v$Version/${asset}-$Version-windows-x64.zip"
}

Write-Host "Baixando $url" -ForegroundColor Cyan
$tmp = Join-Path $env:TEMP "lexicon-install-$([Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
$zip = Join-Path $tmp "download.zip"
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing
    $sha = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLower()
    Write-Host "SHA-256: $sha"

    $x = Join-Path $tmp "x"
    [System.IO.Compression.ZipFile]::ExtractToDirectory($zip, $x)
    $lex = Get-ChildItem -LiteralPath $x -Recurse -Filter lex.exe | Select-Object -First 1
    if (-not $lex) { throw "lex.exe não está no zip baixado." }

    & $lex.FullName install
    if ($LASTEXITCODE -ne 0) { throw "lex install terminou com erro ($LASTEXITCODE)." }

    Write-Host ""
    Write-Host "Lexicon $Version instalado. Abra um terminal novo e rode:" -ForegroundColor Green
    Write-Host "    lex version"
    Write-Host "    lex run hello.lex"
} finally {
    Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
