# Bench compare - lex vs go vs ts vs c

Gerado em: 2026-09-28 09:40 UTC | CI=true | 3 iteracoes (media) via Measure-Command

Toolchain: lex lex 0.1.0-alpha (246.6 MB) | go version go1.26.3 windows/amd64 | node v24.14.1 | Version 6.0.3 | gcc.exe (x86_64-win32-seh-rev0, Built by MinGW-Builds projec

Metodologia: build = `lex build [sem --ci: flag inexistente]` vs `go build -o` vs `tsc --target es2022` vs `gcc -O2`; run = `lex run --ci` vs `go run` vs `node <.js>` vs `<bench>_c.exe`. Esperados: fib=75025, loop=500000500000, json contem `count=1000`.

| bench | lang | build ms (media 3x) | run ms (media 3x) | output correto? | nota |
|---|---|---|---|---|---|
| fib | lex | 332.4 | 320.2 | NAO | ecoou "fib(25)" (stub, nao executa) |
| fib | go | 376.9 | 270.1 | sim |  |
| fib | ts | 991.6 | 56.3 | sim |  |
| fib | c | 195.4 | 26.4 | sim |  |
| loop | lex | 333.4 | 317.7 | NAO | ecoou "total" (stub, nao executa) |
| loop | go | 353.8 | 283.2 | sim |  |
| loop | ts | 983.9 | 56 | sim |  |
| loop | c | 133 | 19.1 | sim |  |
| json | lex | 331.2 | 327.6 | NAO | ecoou "Json::parse(s)" (stub, nao executa) |
| json | go | 449.5 | 319.8 | sim |  |
| json | ts | 998.4 | 54.8 | sim |  |
| json | c | 142.4 | 26.3 | sim |  |

_Nota lex: `lex run` e um stub que extrai `println` sem executar o código (ecoa a expressao/variável); o tempo de run mede overhead do stub + spawn do binário. `lex build` roda o pipeline real (lex->parse->typecheck->codegen LLVM IR). `lex build --ci` nao existe no clap - usa-se `lex build` + CI=true._
