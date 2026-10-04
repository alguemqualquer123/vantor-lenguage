#Requires -Version 5.1
<#
.SYNOPSIS
  Harness E2E do Lexicon: roda tests/e2e_*.lex via `lex run` em modo CI.

.DESCRIPTION
  Para cada tests/e2e_NNN.lex executa o compilador em modo nao-interativo
  (`lex run --ci <arquivo>` quando o binario suporta --ci; caso contrario
  `lex run <arquivo>` com $env:CI="true" e stdin fechado) e conta PASS/FAIL
  pelo exit code. Mede o tempo por teste e o tempo total.

  Uma segunda coluna tenta `lex check <arquivo>` apenas como INFO: o prelude
  embutido do `check` usa imports com ponto (ex.: `import core.io.Console;`)
  que o parser real rejeita (paths exigem `::`), portanto o check pode
  falhar por bug do prelude e NUNCA reprova o teste.

  Saida: exit 0 se todos PASS, exit 1 se houver qualquer FAIL.

.EXAMPLE
  powershell -NoProfile -ExecutionPolicy Bypass -File tests/run_e2e.ps1
#>
[CmdletBinding()]
param(
  [string]$Filter = "e2e_*.lex",
  [int]$MaxTests = 0
)

$ErrorActionPreference = "Continue"
$env:CI = "true"

$scriptDir = $PSScriptRoot
if ([string]::IsNullOrEmpty($scriptDir)) { $scriptDir = (Get-Location).Path }
$repoRoot  = Split-Path -Parent $scriptDir
$testsDir  = Join-Path $repoRoot "tests"
if (-not (Test-Path -LiteralPath $testsDir)) { $testsDir = Join-Path (Get-Location).Path "tests" }

# Resolve lex.exe: ../target/debug/lex.exe (CWD=tests) ou target/debug/lex.exe (CWD=raiz)
$candidates = @(
  (Join-Path $repoRoot "target/debug/lex.exe"),
  (Join-Path (Get-Location).Path "target/debug/lex.exe"),
  (Join-Path $repoRoot "../target/debug/lex.exe"),
  (Join-Path (Get-Location).Path "../target/debug/lex.exe")
)
$lexExe = $null
foreach ($c in $candidates) {
  try {
    $full = [System.IO.Path]::GetFullPath($c)
    if (Test-Path -LiteralPath $full) { $lexExe = $full; break }
  } catch { }
}
if ($null -eq $lexExe) {
  Write-Host "ERRO: lex.exe nao encontrado (tentado: $($candidates -join '; '))" -ForegroundColor Red
  exit 1
}
Write-Host "lex: $lexExe"

# Detecta suporte a --ci (binarios antigos nao tem a flag; ai usa fallback com stdin fechado)
$supportsCi = $false
try {
  $helpText = (& $lexExe run --help 2>&1 | Out-String)
  if ($helpText -match "--ci") { $supportsCi = $true }
} catch { $supportsCi = $false }
if ($supportsCi) { Write-Host "modo: run --ci (suportado)" } else { Write-Host "modo: run (fallback, --ci ausente no binario) + CI=true" -ForegroundColor Yellow }

$files = Get-ChildItem -LiteralPath $testsDir -Filter $Filter -File | Sort-Object Name
if ($MaxTests -gt 0 -and $files.Count -gt $MaxTests) { $files = $files | Select-Object -First $MaxTests }
if ($files.Count -eq 0) {
  Write-Host "ERRO: nenhum arquivo '$Filter' em $testsDir" -ForegroundColor Red
  exit 1
}

$rows = @()
$pass = 0
$fail = 0
$failedNames = @()
$totalSw = [System.Diagnostics.Stopwatch]::StartNew()

foreach ($f in $files) {
  # --- coluna run (vale como PASS/FAIL) ---
  $sw = [System.Diagnostics.Stopwatch]::StartNew()
  $runOut = ""
  $runCode = 99
  try {
    if ($supportsCi) {
      $runOut = (& $lexExe run --ci $f.FullName 2>&1 | Out-String)
      $runCode = $LASTEXITCODE
      if ($runCode -eq 2 -and $runOut -match "unexpected argument '--ci'") {
        # Binario mais antigo do que o esperado: cai para o fallback.
        $supportsCi = $false
        Write-Host "aviso: --ci rejeitado; usando fallback para o restante" -ForegroundColor Yellow
      }
    }
    if (-not $supportsCi) {
      $runOut = ("" | & $lexExe run $f.FullName 2>&1 | Out-String)
      $runCode = $LASTEXITCODE
    }
  } catch {
    $runOut = $_.Exception.Message
    $runCode = 1
  }
  $sw.Stop()

  # --- coluna check (INFO apenas; bug de prelude com ponto nao reprova) ---
  $checkInfo = "info-fail"
  try {
    $checkOut = (& $lexExe check $f.FullName 2>&1 | Out-String)
    if ($checkOut -match "Type checking passed") { $checkInfo = "ok" }
  } catch { $checkInfo = "info-fail" }

  $status = "FAIL"
  if ($runCode -eq 0) { $status = "PASS"; $pass++ } else { $fail++; $failedNames += $f.Name }
  $rows += [pscustomobject]@{
    File    = $f.Name
    Run     = $status
    RunCode = $runCode
    Check   = $checkInfo
    Ms      = [math]::Round($sw.Elapsed.TotalMilliseconds, 1)
  }
}
$totalSw.Stop()

$rows | Format-Table -AutoSize File, Run, RunCode, Check, Ms | Out-String | Write-Host

$total = $pass + $fail
$rate = if ($total -gt 0) { [math]::Round(100.0 * $pass / $total, 1) } else { 0 }
Write-Host ""
Write-Host ("Resumo: PASS {0}/{1} ({2}%)  FAIL {3}  tempo total {4}s" -f $pass, $total, $rate, $fail, [math]::Round($totalSw.Elapsed.TotalSeconds, 2))
if ($failedNames.Count -gt 0) {
  Write-Host ("Falhas: " + ($failedNames -join ", ")) -ForegroundColor Red
}

if ($fail -gt 0) { exit 1 } else { exit 0 }
