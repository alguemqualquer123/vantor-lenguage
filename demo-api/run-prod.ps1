# Sobe a API em PRODUÇÃO: carrega .env.prod e executa `lex run main.lex`.
# Uso (dentro de demo-api): powershell -ExecutionPolicy Bypass -File run-prod.ps1
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
foreach ($raw in [IO.File]::ReadAllLines((Join-Path $here '.env.prod'))) {
    $line = $raw.Trim()
    if ($line -eq '' -or $line.StartsWith('#')) { continue }
    $i = $line.IndexOf('=')
    if ($i -lt 0) { continue }
    $k = $line.Substring(0, $i).Trim()
    $v = $line.Substring($i + 1).Trim()
    [Environment]::SetEnvironmentVariable($k, $v, 'Process')
    Set-Item "env:$k" $v
}
$lex = Join-Path (Split-Path -Parent $here) 'target\debug\lex.exe'
Write-Output "lex-api (production) - http://localhost:3000"
& $lex run (Join-Path $here 'main.lex')
