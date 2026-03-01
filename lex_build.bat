@echo off
setlocal
set PROJECT_ROOT=C:\Users\Admin\Desktop\asax\vantor-lenguage
cd /d %PROJECT_ROOT%
cargo run --package lexicon-cli -- %*
endlocal
