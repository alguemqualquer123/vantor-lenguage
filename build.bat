@echo off
setlocal

echo [1/2] Building Lexicon Language Support Extension...
if exist "themes\lexicon-vscode" (
    cd themes\lexicon-vscode
    call npx @vscode/vsce package --no-dependencies --skip-license --allow-missing-repository
    cd ..\..
) else (
    echo ERRO: Pasta themes\lexicon-vscode nao encontrada.
)

echo.
echo [2/2] Building Lexicon Dark Pro Theme Extension...
if exist "themes\lexicon-dark-pro" (
    cd themes\lexicon-dark-pro
    call npx @vscode/vsce package --no-dependencies --skip-license --allow-missing-repository
    cd ..\..
) else (
    echo ERRO: Pasta themes\lexicon-dark-pro nao encontrada.
)

echo.
echo ==========================================
echo Build concluido! Verifique os arquivos .vsix em:
echo themes/lexicon-vscode/
echo themes/lexicon-dark-pro/
echo ==========================================
pause
