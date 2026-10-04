# release.ps1 — empacota os downloads que o site Lexicon oferece.
#
# Produz, em build\release\:
#   lex-sdk-<ver>-windows-x64.zip  -> SDK completo (binario + 38 shims + 89 libs + exemplos + templates + docs)
#   lex-<ver>-windows-x64.zip      -> so a linguagem (um binario); o SDK se instala sozinho no primeiro run
#   install.ps1 / install.sh       -> copiados da landing, para irem como assets do release
#   checksums.txt                  -> SHA-256 de tudo acima
#
# Uso:  powershell -ExecutionPolicy Bypass -File release.ps1
#       powershell -ExecutionPolicy Bypass -File release.ps1 -SkipBuild
param(
    [switch]$SkipBuild,
    [string]$Out = ""
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.IO.Compression.FileSystem

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $Out) { $Out = Join-Path $Root "build\release" }
$Platform = "windows-x64"

# O build.rs auto-bumpa o patch a cada compilacao release/dist. Aqui isso e
# indesejado: empacotar nao e lanciar versao, e o nome do artefato precisa
# continuar batendo com o binario que ja esta na maquina.
$env:LEX_NO_AUTO_BUMP = "1"

if (-not $SkipBuild) {
    Write-Host "[1/4] cargo build --profile dist -p lexicon-cli"
    Push-Location $Root
    cargo build --profile dist -p lexicon-cli
    $code = $LASTEXITCODE
    Pop-Location
    if ($code -ne 0) { throw "cargo build falhou (exit $code)." }
}

$Bin = Join-Path $Root "target\dist\lex.exe"
if (-not (Test-Path -LiteralPath $Bin)) { throw "Binario nao encontrado: $Bin (rode sem -SkipBuild)." }

# A versao sai do proprio binario (`lex version`), nao do Cargo.toml: o
# manifesto pode estar um patch a frente de quem foi compilado, e quem o
# usuario roda e o exe.
$Version = ((& $Bin version | Select-Object -First 1) -split '\s+')[1]
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Nao consegui ler a versao do binario: '$Version'." }

Write-Host "== Lexicon $Version ($Platform) ==" -ForegroundColor Cyan

Write-Host "[2/4] lex sdk export"
if (Test-Path -LiteralPath $Out) { Remove-Item -LiteralPath $Out -Recurse -Force }
New-Item -ItemType Directory -Force -Path $Out | Out-Null

$SdkDir = Join-Path $Out "lex-sdk-$Version-$Platform"
& $Bin sdk export --out $SdkDir
if ($LASTEXITCODE -ne 0) { throw "lex sdk export falhou." }
& $Bin sdk verify --out $SdkDir
if ($LASTEXITCODE -ne 0) { throw "lex sdk verify falhou no kit exportado." }

$LangDir = Join-Path $Out "lex-$Version-$Platform"
New-Item -ItemType Directory -Force -Path $LangDir | Out-Null
Copy-Item -LiteralPath $Bin -Destination (Join-Path $LangDir "lex.exe")

Write-Host "[3/4] zipando os dois sabores"

# Zip à mão: CreateFromDirectory gravia separador '\' nos nomes das entradas,
# e isso quebra quem descompacta em Linux/macOS (o '\' vira parte do nome do
# arquivo). As entradas saem com '/' e sem diretórios vazios.
function Zip-Dir {
    param([string]$Source, [string]$Destination)
    $zip = [System.IO.Compression.ZipFile]::Open($Destination, 'Create')
    try {
        $base = (Resolve-Path -LiteralPath $Source).Path.TrimEnd('\') + '\'
        Get-ChildItem -LiteralPath $Source -Recurse -File -Force | ForEach-Object {
            $relative = $_.FullName.Substring($base.Length).Replace('\', '/')
            [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $_.FullName, $relative) | Out-Null
        }
    } finally {
        $zip.Dispose()
    }
    (Get-Item -LiteralPath $Destination).Length
}

$SdkZip = Join-Path $Out "lex-sdk-$Version-$Platform.zip"
$LangZip = Join-Path $Out "lex-$Version-$Platform.zip"
"sdk zip: {0:N0} bytes" -f (Zip-Dir $SdkDir $SdkZip)
"lang zip: {0:N0} bytes" -f (Zip-Dir $LangDir $LangZip)
Remove-Item -LiteralPath $SdkDir, $LangDir -Recurse -Force

# Os instaladores vivem na landing para o site poder servi-los; aqui viram
# assets do release, que e de onde os usuarios os baixam de verdade.
$Assets = @($SdkZip, $LangZip)
foreach ($name in @("install.ps1", "install.sh")) {
    $src = Join-Path $Root "lexicon-landing\public\$name"
    if (Test-Path -LiteralPath $src) {
        Copy-Item -LiteralPath $src -Destination (Join-Path $Out $name)
        $Assets += (Join-Path $Out $name)
    }
}

# A super extensao VS Code ja vem empacotada no repo; publicar o .vsix no
# mesmo release da site um link de instalacao que funciona.
$vsix = Get-ChildItem -LiteralPath (Join-Path $Root "themes\lexicon-vscode") -Filter "lexicon-super-*.vsix" |
    ForEach-Object {
        if ($_.BaseName -match '^lexicon-super-(\d+\.\d+\.\d+)$') {
            [pscustomobject]@{ File = $_; Ver = [version]$Matches[1] }
        }
    } | Sort-Object Ver -Descending | Select-Object -First 1 -ExpandProperty File
if ($vsix) {
    Copy-Item -LiteralPath $vsix.FullName -Destination (Join-Path $Out $vsix.Name)
    $Assets += (Join-Path $Out $vsix.Name)
}

Write-Host "[4/4] checksums"
$Lines = $Assets | ForEach-Object {
    "{0}  {1}" -f (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash.ToLower(), (Split-Path -Leaf $_)
}
$Lines -join "`n" | Set-Content -LiteralPath (Join-Path $Out "checksums.txt") -Encoding ascii

Write-Host ""
Write-Host "Artefatos prontos em $Out" -ForegroundColor Green
Get-ChildItem -LiteralPath $Out | ForEach-Object {
    Write-Host ("  {0,-42} {1,10:N0} bytes" -f $_.Name, $_.Length)
}
