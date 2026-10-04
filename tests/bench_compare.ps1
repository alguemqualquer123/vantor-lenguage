<#
.SYNOPSIS
  Compara build + run dos benchmarks bench_{fib,loop,json} em 4 linguagens
  (lex, go, ts, c). Roda cada comando 3x via Measure-Command, imprime tabela
  markdown e salva tests/bench_results.md.
.NOTES
  - Fixa CI=true para o lex nunca bloquear em "Press Enter".
  - `lex build` NAO possui flag --ci no clap (só run/bench tem); o harness
    detecta isso 1x (probe) e usa `lex build <file>` + CI=true.
  - `lex run` e um stub de interpretacao (extrai prints, nao executa o código),
    por isso a coluna "output OK?" tende a falhar p/ lex com a expressao ecoada.
#>
$ErrorActionPreference = 'Continue'
$env:CI = 'true'

$Root = Split-Path $PSScriptRoot -Parent
$SrcDir = Join-Path $Root 'src\examples\tests'
$ResultsMd = Join-Path $PSScriptRoot 'bench_results.md'
$Tmp = Join-Path ([System.IO.Path]::GetTempPath()) 'lexbench'
New-Item -ItemType Directory -Path $Tmp -Force | Out-Null

# --- resolve toolchains ---
$LexExe = Join-Path $Root 'target\debug\lex.exe'
if (-not (Test-Path -LiteralPath $LexExe)) { $LexExe = 'lex' }
$TscCmd = (Get-Command tsc -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)
if (-not $TscCmd) { $TscCmd = 'tsc' }

$Iterations = 3
$Benches = @(
    @{ Name = 'fib';  Expected = '75025' },
    @{ Name = 'loop'; Expected = '500000500000' },
    @{ Name = 'json'; Expected = 'count=1000' }
)
$Langs = @('lex', 'go', 'ts', 'c')

function Get-LexBuildFlag {
    # Probe 1x: `lex build --ci` existe? (no clap atual, NAO existe)
    $fib = Join-Path $SrcDir 'bench_fib.lex'
    $out = & $LexExe build --ci $fib 2>&1 | ForEach-Object { "$_" }
    $joined = ($out -join "`n")
    if ($joined -match 'unexpected argument') { return @() }
    return @('--ci')
}
$LexBuildFlag = Get-LexBuildFlag

function Get-BuildCommand($lang, $bench) {
    $lexFile = Join-Path $SrcDir "bench_$bench.lex"
    $goFile  = Join-Path $SrcDir "bench_$bench.go"
    $tsFile  = Join-Path $SrcDir "bench_$bench.ts"
    $cFile   = Join-Path $SrcDir "bench_$bench.c"
    switch ($lang) {
        'lex' { return @{ Exe = $LexExe; Args = @('build') + $LexBuildFlag + @($lexFile); Cwd = $Root } }
        'go'  { return @{ Exe = 'go'; Args = @('build', '-o', (Join-Path $Tmp "bench_${bench}_go.exe"), $goFile); Cwd = $Root } }
        'ts'  { return @{ Exe = $TscCmd; Args = @($tsFile, '--target', 'es2022', '--module', 'commonjs', '--outDir', (Join-Path $Tmp "ts_$bench")); Cwd = $Root } }
        'c'   { return @{ Exe = 'gcc'; Args = @('-O2', '-o', (Join-Path $Tmp "bench_${bench}_c.exe"), $cFile); Cwd = $Root } }
    }
}

function Get-RunCommand($lang, $bench) {
    $lexFile = Join-Path $SrcDir "bench_$bench.lex"
    $goFile  = Join-Path $SrcDir "bench_$bench.go"
    switch ($lang) {
        'lex' { return @{ Exe = $LexExe; Args = @('run', '--ci', $lexFile); Cwd = $Root } }
        'go'  { return @{ Exe = 'go'; Args = @('run', $goFile); Cwd = $Root } }
        'ts'  { return @{ Exe = 'node'; Args = @((Join-Path $Tmp "ts_$bench\bench_$bench.js")); Cwd = $Root } }
        'c'   { return @{ Exe = (Join-Path $Tmp "bench_${bench}_c.exe"); Args = @(); Cwd = $Root } }
    }
}

function Invoke-Timed($cmd, [ref]$captured) {
    # Roda 1x via Measure-Command; captura stdout+stderr como string[].
    $box = @{ Out = @(); Code = 0 }
    $ms = (Measure-Command {
        Push-Location -LiteralPath $cmd.Cwd
        try {
            $exe = $cmd.Exe
            $argList = @($cmd.Args)
            $o = & $exe @argList 2>&1 | ForEach-Object { "$_" }
            $box.Out = @($o)
            $box.Code = $LASTEXITCODE
        } catch {
            $box.Out = @("EXCEPTION: $($_.Exception.Message)")
            $box.Code = -1
        } finally {
            Pop-Location
        }
    }).TotalMilliseconds
    $captured.Value = $box
    return $ms
}

function Clean-Output($lines) {
    $ansi = [char]27 + '\[[0-9;?]*[a-zA-Z]'
    $clean = @()
    foreach ($l in $lines) {
        $t = $l -replace $ansi, ''
        $t = $t.Trim()
        if ($t -eq '') { continue }
        if ($t -like '*Lexicon added to system PATH*') { continue }
        if ($t -like 'Compiling and running*') { continue }
        if ($t -match '^-+$') { continue }
        if ($t -like '*CI mode*') { continue }
        if ($t -like 'Run time:*' -or $t -like 'Build time:*') { continue }
        if ($t -match '^\[\d{4}-') { continue }  # linhas de log env_logger
        if ($t -like 'Building (*' -or $t -like 'Building Lexicon*') { continue }
        if ($t -like 'Compiling...*' -or $t -like 'Pipeline de compila*') { continue }
        if ($t -like 'Build successful*') { continue }
        $clean += $t
    }
    return ($clean -join "`n")
}

$rows = @()
foreach ($b in $Benches) {
    foreach ($lang in $Langs) {
        Write-Host "[$($b.Name)/$lang] build x$Iterations + run x$Iterations ..."
        $buildCmd = Get-BuildCommand $lang $b.Name
        $runCmd = Get-RunCommand $lang $b.Name

        $buildTimes = @(); $buildCode = 0
        for ($i = 1; $i -le $Iterations; $i++) {
            $cap = $null
            $ms = Invoke-Timed $buildCmd ([ref]$cap)
            $buildTimes += $ms
            if ($cap.Code -ne 0 -and $cap.Code -ne $null) { $buildCode = $cap.Code }
        }
        $buildAvg = [math]::Round(($buildTimes | Measure-Object -Average).Average, 1)

        $runTimes = @(); $lastOut = ''; $runCode = 0; $rawFirst = ''
        for ($i = 1; $i -le $Iterations; $i++) {
            $cap = $null
            $ms = Invoke-Timed $runCmd ([ref]$cap)
            $runTimes += $ms
            $runCode = $cap.Code
            $lastOut = Clean-Output $cap.Out
            if ($lastOut -ne '') { $rawFirst = ($lastOut -split "`n" | Select-Object -First 1) }
        }
        $runAvg = [math]::Round(($runTimes | Measure-Object -Average).Average, 1)

        $ok = $lastOut.Contains($b.Expected)
        $okStr = if ($ok) { 'sim' } else { 'NAO' }
        $note = ''
        if ($lang -eq 'lex' -and -not $ok) { $note = "ecoou `"$rawFirst`" (stub, nao executa)" }
        elseif ($runCode -ne 0) { $note = "exit=$runCode" }
        elseif ($buildCode -ne 0) { $note = "build exit=$buildCode" }
        if ($lang -eq 'lex' -and $LexBuildFlag.Count -eq 0 -and $note -eq '') { $note = 'build sem --ci (flag inexistente)' }

        $rows += [pscustomobject]@{
            Bench = $b.Name; Lang = $lang
            BuildMs = $buildAvg; RunMs = $runAvg
            OutputOk = $okStr; Note = $note
        }
    }
}

# --- markdown ---
$now = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm UTC')
$lexVer = ((& $LexExe --version 2>&1 | ForEach-Object { "$_" } | Where-Object { $_ -notlike '*system PATH*' } | Select-Object -Last 1) -join ' ')
$goVer = ((go version 2>&1 | ForEach-Object { "$_" }) -join ' ')
$nodeVer = ((node --version 2>&1 | ForEach-Object { "$_" }) -join ' ')
$tscVer = ((& $TscCmd --version 2>&1 | ForEach-Object { "$_" }) -join ' ')
$gccFull = ((gcc --version 2>&1 | Select-Object -First 1 | ForEach-Object { "$_" }) -join ' ').Trim()
$gccShort = if ($gccFull.Length -gt 60) { $gccFull.Substring(0, 60) } else { $gccFull }
$lexSize = if (Test-Path -LiteralPath $LexExe) { [math]::Round((Get-Item -LiteralPath $LexExe).Length / 1MB, 1) } else { '?' }

$md = @()
$md += '# Bench compare - lex vs go vs ts vs c'
$md += ''
$md += "Gerado em: $now | CI=true | 3 iteracoes (media) via Measure-Command"
$md += ''
$md += "Toolchain: lex $lexVer ($lexSize MB) | $goVer | node $nodeVer | $tscVer | $gccShort"
$md += ''
$md += 'Metodologia: build = `lex build [sem --ci: flag inexistente]` vs `go build -o` vs `tsc --target es2022` vs `gcc -O2`; run = `lex run --ci` vs `go run` vs `node <.js>` vs `<bench>_c.exe`. Esperados: fib=75025, loop=500000500000, json contem `count=1000`.'
$md += ''
$md += '| bench | lang | build ms (media 3x) | run ms (media 3x) | output correto? | nota |'
$md += '|---|---|---|---|---|---|'
foreach ($r in $rows) {
    $md += "| $($r.Bench) | $($r.Lang) | $($r.BuildMs) | $($r.RunMs) | $($r.OutputOk) | $($r.Note) |"
}
$md += ''
$md += '_Nota lex: `lex run` e um stub que extrai `println` sem executar o código (ecoa a expressao/variável); o tempo de run mede overhead do stub + spawn do binário. `lex build` roda o pipeline real (lex->parse->typecheck->codegen LLVM IR). `lex build --ci` nao existe no clap - usa-se `lex build` + CI=true._'
$mdText = ($md -join "`n") + "`n"
[System.IO.File]::WriteAllText($ResultsMd, $mdText, [System.Text.Encoding]::UTF8)

Write-Host ''
Write-Host $mdText
Write-Host "Salvo em: $ResultsMd"
