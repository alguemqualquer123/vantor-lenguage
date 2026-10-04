Boa — a extensão já tem base sólida (gramática, 85 snippets, 3 temas, 14 comandos, hover/completion/definition/symbols, vet-on-save). Aqui vai uma lista de melhorias possíveis, organizadas por impacto:
IntelliSense (maior valor)
1. Signature help — ao digitar Http::serve( mostrar os parâmetros (address: String) como hint inline
2. Completion de membros via . — hoje cobre Modulo::; faltam métodos de instância (minhaLista. → push/pop/get/map…)
3. Rename symbol (F2) — renomear função/variável no arquivo com preview
4. Find all references — listar onde um símbolo é usado no workspace
5. Workspace symbol search (Ctrl+T) — buscar struct User em todos os .lex do projeto
6. Inlay hints — mostrar tipos inferidos (let x = 5; → int apagadinho ao lado)
7. Semantic highlighting — o tema já tem "semanticHighlighting": true; falta o provider que colore variáveis mutáveis, parâmetros e tipos com mais precisão que regex
8. Auto-import ao aceitar completion normal — hoje só em Mod::; estender para quando você completa serve solto virar Http::serve + import
Diagnóstico e qualidade
 9. Squiggles precisos — hoje o vet-on-save marca a linha aproximada; mapear arquivo:linha:coluna e códigos E0301/E0204 para ranges exatos com codes clicáveis
10. Quick fixes (Ctrl+.) — ex.: "adicionar import ausente", "trocar == true por direto", "adicionar _ em match não-exaustivo"
11. Status bar rica — $(check) lex vet: limpo / $(error) 3 erros clicável que abre o Problems
12. Problem Matcher — lex test/vet com regex de file:line:col para Tasks mostrarem erro navegável
13. Folding ranges — dobrar fn/struct/class/match via provider (hoje só #region)
Produtividade
14. Snippets contextuais — test só sugerir dentro de fn, @Get só no topo; snippets de Db com migration completa
15. File templates — lexicon.newProject gerar main.lex + .env.dev/.prod + run-dev.ps1 (estilo demo-api)
16. Debugger real (Debug Adapter) — breakpoints/step/locals via DAP ligando no lex debug (hoje só abre terminal)
17. Test Explorer — descobrir @Test e mostrar ▶/✅ na sidebar de testes
18. Task provider dinâmico — tasks lex: run/build/test auto-detectadas por arquivo .lex aberto
19. Formatting provider — Shift+Alt+F chamando lex fmt de verdade (hoje só comando no terminal)
20. Color picker em strings de cor — setColor("#FFF") com preview (via DocumentColorProvider)
Temas e visual
21. Ícones por símbolo — Outline mostrando fn/struct/enum com ícones distintos (vem do DocumentSymbolProvider, quase de graça)
22. Bracket pair colorization específica — guias coloridos para match {} aninhados
23. Tema high-contrast — variante acessível (WCAG) dos 3 temas
24. Semantic token legend — registrar tokenTypes custom (lexFunction, lexDecorator) para controle fino
Ecossistema
25. Walkthrough (Get Started) — guia interativo embutido: instala lex, cria projeto, roda demo-api
26. Settings UI completa — descrições + markdownDescription com exemplos em todas as configs
27. Telemetry opcional — contar comandos usados para priorizar evolução (opt-in, privacidade explícita)
28. i18n pt-BR/en — package.nls.json com as strings da extensão traduzidas
29. Extension Pack — lexicon-pack: super + tema dark-pro + snippets extras num único install
30. Web extension — compilar para vscode.dev (sem child_process; rodar lex via terminal link + playground)