@echo off
setlocal

echo [1/2] Building Lexicon Language Support Extension...
cd lexicon-vscode
call npx @vscode/vsce package --no-dependencies --skip-license --allow-missing-repository
cd ..\..

echo.
echo [2/2] Building Lexicon Dark Pro Theme Extension...
cd lexicon-dark-pro
call npx @vscode/vsce package --no-dependencies --skip-license --allow-missing-repository
cd ..\..

echo.
echo ==========================================
echo Build concluído com sucesso!
echo Arquivos .vsix gerados nas pastas themes/lexicon-vscode e themes/lexicon-dark-pro
echo ==========================================
pause
