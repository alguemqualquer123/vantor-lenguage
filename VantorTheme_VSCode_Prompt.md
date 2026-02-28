🎨 Prompt — Criação de Tema Visual Profissional para VSCode

🎯 Objetivo

Crie um tema visual completo e profissional para o Visual Studio Code, com identidade moderna, foco em legibilidade, contraste ideal para longas horas de programação e compatibilidade com linguagens modernas (Java, C#, Rust, TypeScript, etc).

O nome do tema será:

Vantor Dark Pro

O tema deve ser competitivo com temas como:

- Dracula
- One Dark Pro
- GitHub Dark
- Nord
- Monokai Pro

Mas com identidade própria, minimalista e tecnológica.

🧠 Diretrizes de Design

O tema deve:

- Ser predominantemente escuro (Dark Theme)
- Ter contraste equilibrado (não agressivo)
- Reduzir fadiga visual
- Ter distinção clara entre:
  - Keywords
  - Types
  - Strings
  - Functions
  - Variables
  - Comments
- Ter excelente leitura em monitores IPS e OLED
- Ter paleta coerente e profissional
- Funcionar bem com ligatures (Fira Code, JetBrains Mono)

🎨 Paleta de Cores

Definir uma paleta oficial com:

- Background primário
- Background secundário
- Sidebar
- Activity bar
- Status bar
- Selection
- Line highlight
- Cursor
- Error
- Warning
- Info
- Success

Exemplo base (pode melhorar):

- Background: #0F1117
- Sidebar: #151823
- Panel: #1A1D29
- Primary Accent: #5DA9FF
- Secondary Accent: #A277FF
- Success: #4CD97B
- Warning: #FFB454
- Error: #FF5C7A
- Comment: #5C6370
- String: #A3E88D
- Function: #82AAFF
- Keyword: #C792EA
- Type: #FFCB6B

Melhorar e justificar tecnicamente contraste e harmonia.

🧾 Estrutura Técnica do Tema

Gerar:

- Estrutura completa da extensão VSCode
- package.json configurado corretamente
- Arquivo JSON do tema
- Configuração de tokenColors
- Cores da UI (workbench)
- Semantic highlighting configurado
- Ícone e nome do publisher fictício

📂 Estrutura do Projeto

```
vantor-dark-pro/
 ├── package.json
 ├── themes/
 │    └── vantor-dark-pro-color-theme.json
 ├── README.md
 └── icon.png
```

🧩 Token Colors (Syntax Highlighting)

Definir regras para:

- **Keywords**: public, class, async, await, return, if, else, match
- **Types**: int, string, bool, Task, Result
- **Functions**: Métodos declarados, Funções globais
- **Variables**: Variáveis locais, Campos privados, Constantes
- **Strings**
- **Numbers**
- **Comments**: Diferenciar TODO, FIXME, NOTE
- **Annotations / Decorators**
- **Generics**
- **Operators**

🖥️ UI Colors (Workbench)

Definir detalhadamente:

- editor.background
- editor.foreground
- editor.lineHighlightBackground
- editorCursor.foreground
- editor.selectionBackground
- editor.inactiveSelectionBackground
- activityBar.background
- sideBar.background
- sideBar.foreground
- statusBar.background
- statusBar.foreground
- panel.background
- terminal.background
- terminal.foreground

⚙️ Recursos Avançados

O tema deve incluir:

- Semantic Highlighting ativado
- Cores diferentes para:
  - readonly variables
  - parameters
  - class names
  - interfaces
  - enums
- Diferenciação visual clara entre:
  - mutable e immutable
  - async functions
- Compatibilidade com:
  - TypeScript
  - Java
  - C#
  - Rust
  - JSON
  - Markdown

📦 package.json

Gerar um package.json completo contendo:

- name
- displayName
- description
- version
- publisher
- engines.vscode
- contributes.themes
- categories
- keywords

📘 README.md

Gerar README contendo:

- Descrição do tema
- Screenshots (descritos)
- Instalação
- Como publicar na marketplace
- Como testar localmente (vsce package)

🚀 Publicação

Explicar:

- Como instalar vsce
- Como gerar .vsix
- Como publicar na marketplace
- Como versionar corretamente

🎯 Resultado Esperado

A IA deve gerar:

- Código completo do tema
- JSON válido
- Estrutura organizada
- Pronto para publicar
- Design sofisticado
- Justificativa das decisões de design
- Diferencial competitivo frente aos principais temas

🔥 Instrução Final

Gere o projeto completo do tema Vantor Dark Pro pronto para produção e publicação no marketplace do Visual Studio Code, com qualidade profissional e identidade visual marcante.
