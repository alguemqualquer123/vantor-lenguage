#Requires -Version 5.1
<#
.SYNOPSIS
  Load battery for Lexicon: stress programs, parallelism, CLI and LSP floods.

.DESCRIPTION
  1. runs every tests/stress/load_*.lex with a hard timeout
     (files ending in _fail.lex MUST exit non-zero, quickly and cleanly —
     stack overflow / step budget / parse depth must never abort the host);
  2. runs 8 copies of the import storm concurrently (shared-nothing check);
  3. hammers `lex complete` 50x (engine robustness + timing);
  4. pipelines 31 LSP requests through ONE `lex lsp` session (deadlock check).

  Exit 0 when everything behaves, 1 otherwise.
#>
[CmdletBinding()]
param(
  [int]$TimeoutSec = 240
)

$ErrorActionPreference = "Continue"
$env:CI = "true"

$repoRoot = Split-Path -Parent $PSScriptRoot
$lexExe = Join-Path $repoRoot "target/debug/lex.exe"
if (-not (Test-Path -LiteralPath $lexExe)) {
  Write-Host "ERRO: lex.exe nao encontrado em $lexExe (cargo build -p lexicon-cli)" -ForegroundColor Red
  exit 1
}
Write-Host "lex: $lexExe"

$script:pass = 0
$script:fail = 0
$script:failedNames = @()

function Invoke-LexTimed {
  param([string]$Name, [string[]]$Args, [int]$TimeoutMs, [bool]$ExpectFail = $false)
  $sw = [System.Diagnostics.Stopwatch]::StartNew()
  $p = New-Object System.Diagnostics.Process
  $p.StartInfo.FileName = $lexExe
  # Quote args containing spaces (repo lives under `[ Trabalhos ]`).
  $p.StartInfo.Arguments = (($Args | ForEach-Object { if ($_ -match '\s') { "`"$_`"" } else { $_ } }) -join " ")
  $p.StartInfo.RedirectStandardOutput = $true
  $p.StartInfo.RedirectStandardError = $true
  $p.StartInfo.UseShellExecute = $false
  $p.StartInfo.WorkingDirectory = $repoRoot
  [void]$p.Start()
  $done = $p.WaitForExit($TimeoutMs)
  $code = 99
  if ($done) {
    $code = $p.ExitCode
  } else {
    try { $p.Kill() } catch { }
  }
  $sw.Stop()
  $ok = if ($ExpectFail) { ($done -and $code -ne 0) } else { ($done -and $code -eq 0) }
  $tag = if ($ExpectFail) { "FAIL-EXPECTED" } else { "run" }
  if ($ok) {
    $script:pass++
    Write-Host ("LOAD-PASS {0} [{1}] exit={2} {3}ms" -f $Name, $tag, $code, [math]::Round($sw.Elapsed.TotalMilliseconds))
  } else {
    $script:fail++
    $script:failedNames += $Name
    $why = if (-not $done) { "TIMEOUT>${TimeoutMs}ms" } else { "exit=$code" }
    Write-Host ("LOAD-FAIL {0} [{1}] {2} {3}ms" -f $Name, $tag, $why, [math]::Round($sw.Elapsed.TotalMilliseconds)) -ForegroundColor Red
  }
}

# --- 1. stress files -------------------------------------------------------
$files = Get-ChildItem -LiteralPath (Join-Path $repoRoot "tests/stress") -Filter "load_*.lex" -File | Sort-Object Name
foreach ($f in $files) {
  $expectFail = $f.BaseName.EndsWith("_fail")
  Invoke-LexTimed -Name $f.Name -Args @("run", "--ci", $f.FullName) -TimeoutMs ($TimeoutSec * 1000) -ExpectFail $expectFail
}

# --- 2. parallel storm: 8 concurrent runs, all must exit 0 -----------------
$storm = Join-Path $repoRoot "tests/stress/load_import_storm.lex"
$jobs = @()
for ($i = 0; $i -lt 8; $i++) {
  $p = New-Object System.Diagnostics.Process
  $p.StartInfo.FileName = $lexExe
  $p.StartInfo.Arguments = "run --ci `"$storm`""
  $p.StartInfo.RedirectStandardOutput = $true
  $p.StartInfo.RedirectStandardError = $true
  $p.StartInfo.UseShellExecute = $false
  $p.StartInfo.WorkingDirectory = $repoRoot
  [void]$p.Start()
  $jobs += $p
}
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$allOk = $true
foreach ($p in $jobs) {
  if (-not $p.WaitForExit($TimeoutSec * 1000)) { $allOk = $false; try { $p.Kill() } catch { } }
  elseif ($p.ExitCode -ne 0) { $allOk = $false }
}
$sw.Stop()
if ($allOk) { $script:pass++; Write-Host ("LOAD-PASS parallel-x8 exit=0 {0}ms" -f [math]::Round($sw.Elapsed.TotalMilliseconds)) }
else { $script:fail++; $script:failedNames += "parallel-x8"; Write-Host "LOAD-FAIL parallel-x8" -ForegroundColor Red }

# --- 3. complete hammer: 50 engine calls ------------------------------------
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$completeOk = $true
for ($i = 0; $i -lt 50; $i++) {
  $o = (& $lexExe complete --prefix "strings::" 2>&1 | Out-String)
  if ($LASTEXITCODE -ne 0 -or $o -notmatch "ToUpper") { $completeOk = $false; break }
}
$sw.Stop()
if ($completeOk) { $script:pass++; Write-Host ("LOAD-PASS complete-x50 {0}ms" -f [math]::Round($sw.Elapsed.TotalMilliseconds)) }
else { $script:fail++; $script:failedNames += "complete-x50"; Write-Host "LOAD-FAIL complete-x50" -ForegroundColor Red }

# --- 4. LSP flood: 31 requests, one session, bounded -------------------------
$flood = {
  param($lexExe, $repoRoot)
  $p = New-Object System.Diagnostics.Process
  $p.StartInfo.FileName = $lexExe
  $p.StartInfo.Arguments = "lsp"
  $p.StartInfo.RedirectStandardInput = $true
  $p.StartInfo.RedirectStandardOutput = $true
  $p.StartInfo.UseShellExecute = $false
  $p.StartInfo.WorkingDirectory = $repoRoot
  [void]$p.Start()
  $w = $p.StandardInput
  $r = $p.StandardOutput
  function Send($id, $method, $params) {
    $body = @{ jsonrpc = "2.0"; id = $id; method = $method; params = $params } | ConvertTo-Json -Depth 8 -Compress
    $w.Write("Content-Length: $($body.Length)`r`n`r`n$body")
    $w.Flush()
  }
  function Recv() {
    $len = 0
    while (($line = $r.ReadLine()) -ne $null) {
      if ($line -eq "") { break }
      if ($line -match "Content-Length:\s*(\d+)") { $len = [int]$Matches[1] }
    }
    if ($len -eq 0) { return $null }
    $buf = New-Object char[] $len
    $n = $r.Read($buf, 0, $len)
    return (-join $buf[0..($n - 1)])
  }
  try {
    Send 1 "initialize" @{ capabilities = @{} }
    if (-not (Recv)) { return "no-init" }
    $got = 0
    for ($i = 0; $i -lt 10; $i++) {
      Send (10 + $i) "textDocument/completion" @{ textDocument = @{ uri = "file:///x.lex" }; position = @{ line = 0; character = 7 } }
      Send (100 + $i) "textDocument/hover" @{ textDocument = @{ uri = "file:///x.lex" }; position = @{ line = 0; character = 7 } }
      Send (200 + $i) "textDocument/definition" @{ textDocument = @{ uri = "file:///x.lex" }; position = @{ line = 0; character = 7 } }
    }
    for ($i = 0; $i -lt 30; $i++) {
      if (Recv) { $got++ } else { break }
    }
    try { $w.Close() } catch { }
    $p.WaitForExit(10000) | Out-Null
    return "got=$got/30"
  } finally {
    try { if (-not $p.HasExited) { $p.Kill() } } catch { }
  }
}

$job = Start-Job -ScriptBlock $flood -ArgumentList $lexExe, $repoRoot
if (Wait-Job $job -Timeout 150 | Out-Null) {
  $res = Receive-Job $job
  Remove-Job $job -Force
  if ($res -eq "got=30/30") { $script:pass++; Write-Host "LOAD-PASS lsp-flood got=30/30" }
  else { $script:fail++; $script:failedNames += "lsp-flood"; Write-Host "LOAD-FAIL lsp-flood ($res)" -ForegroundColor Red }
} else {
  Stop-Job $job; Remove-Job $job -Force
  $script:fail++; $script:failedNames += "lsp-flood"
  Write-Host "LOAD-FAIL lsp-flood (job timeout — possible server deadlock)" -ForegroundColor Red
}

# --- summary -----------------------------------------------------------------
Write-Host ""
Write-Host ("Carga: PASS {0}  FAIL {1}" -f $script:pass, $script:fail)
if ($script:failedNames.Count -gt 0) {
  Write-Host ("Falhas: " + ($script:failedNames -join ", ")) -ForegroundColor Red
}
if ($script:fail -gt 0) { exit 1 } else { exit 0 }
