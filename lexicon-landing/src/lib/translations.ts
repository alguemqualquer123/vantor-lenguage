export const translations = {
  en: {
    nav: {
      features: "Features",
      templates: "Templates",
      docs: "Docs",
      download: "Download"
    },
    hero: {
      version: "v0.3.5",
      tagline: "One binary · 89 std packages · real GUI",
      title_part1: "CODE",
      title_part2: "WITHOUT LIMITS.",
      description: "Lexicon is a language with its own toolchain in a single 9,4 MB binary: run, hot reload, check, vet, fmt, a package manager, a wgpu GUI engine and 89 standard-library packages written in Lex itself. Download the SDK, or just the language — it installs the SDK for you.",
      download_sdk: "DOWNLOAD THE SDK",
      download_lang: "JUST THE LANGUAGE",
      repository: "REPOSITORY"
    },
    features: {
      title: "Built for Real Work",
      subtitle: "v0.3.5 capabilities — verified against the actual toolchain",
      hotreload: {
        title: "Hot Reload Supervisor",
        desc: "Save → auto-restart with lex run --watch. 300 ms debounce, .lex-only filter, and crash backoff instead of hot-spinning."
      },
      extension: {
        title: "Super Extension 1.2.0",
        desc: "VS Code extension: 117 snippets, 4 themes + icon theme, contextual autocomplete with auto-import, and vet diagnostics on save."
      },
      diagnostics: {
        title: "Pre-Run Diagnostics",
        desc: "Syntax gate with file:line:col errors before anything runs. lex check and lex vet plus a stable E-code catalog (E0101–E0801)."
      },
      engine: {
        title: "GUI Engine on wgpu",
        desc: "The same binary draws windows: Window / Canvas / Input on Vulkan, DX12, Metal, OpenGL or WebGPU. lex new -t gui, or run the Pong demo."
      }
    },
    download: {
      title: "DOWNLOAD",
      subtitle: "Two flavours, same toolchain",
      from_github: "on GitHub",
      button: "DOWNLOAD",
      sdk: {
        title: "Full SDK",
        blurb: "Everything, unpacked and ready: the lex binary, 38 launcher shims, 89 std packages, 19 runnable examples, 5 templates, docs and scripts.",
        size_zip: "4,4 MB zip",
        size_installed: "9,7 MB installed"
      },
      lang: {
        title: "Just the language",
        blurb: "A single lex.exe. The first run registers itself on PATH and writes the same SDK into ~/.lexicon/sdk — same result, one smaller download.",
        size_zip: "4,3 MB zip",
        size_installed: "9,4 MB installed"
      },
      contains_sdk: {
        title: "What the SDK zip contains",
        items: [
          "bin/lex.exe + 38 shims (lex-run, lex-check, lex-mod, …)",
          "lib/std — 89 packages written in Lex",
          "examples — 19 programs that actually run",
          "templates — default · api · plugin · service · gui",
          "docs + scripts + LICENSE-MIT + VERSION",
          "check it: lex sdk verify"
        ]
      },
      contains_lang: {
        title: "What one binary does",
        items: [
          "~9,4 MB, one file, no runtime to install",
          "first run: PATH entry + SDK export (silent, idempotent)",
          "lex run · check · vet · fmt · mod · sdk · gui · lsp · dap",
          "uninstall: lex uninstall"
        ]
      },
      installer_title: "Or let a script do it",
      installer_windows: "Windows (PowerShell)",
      installer_posix: "Linux and macOS",
      installer_note: "install.sh looks for a compiled artifact for your platform in the release; when there isn't one it builds from source with cargo and runs the same lex install.",
      checksums: "SHA-256 checksums",
      extension: "Super extension (.vsix)",
      all_releases: "All releases and assets",
      manual: {
        title: "Manual install",
        steps: [
          "Unzip wherever you like",
          "Run lex.exe install",
          "Open a NEW terminal (PATH is persisted per session)",
          "Check with lex version"
        ]
      },
      os_title: "Platforms",
      os_note: "The release carries pre-built Windows x64 binaries; on other platforms install.sh compiles from source in ~2 min.",
      no_admin: "No admin rights needed",
      source_title: "Build from source",
      source_note: "Rust toolchain + the dist profile (opt-level z + fat LTO) — the same one we ship."
    },
    templates: {
      title: "Start in Seconds",
      subtitle: "Scaffold instantly, then run the real thing.",
      api: "REST API",
      api_desc: "Scaffold with lex new api. Then run the real demo-api: axum HTTP + SQLite with dev/prod .env flows.",
      gui: "GUI / Game",
      gui_desc: "lex new my-window -t gui gives you a real wgpu window: loop, input, Canvas drawing. Pong is already in examples/.",
      plugin: "WASM Plugin",
      plugin_desc: "Scaffold with lex new plugin --target wasm. Snippets and grammar included; check the docs for current target status.",
      service: "gRPC Service",
      service_desc: "Scaffold with lex new service --grpc. Validate every step with lex vet before you run.",
      quickstart_title: "Real quickstart — try it now",
      quickstart_desc: "These commands work against the v0.3.5 toolchain today."
    },
    docs: {
      title: "LEXICON DOCS",
      search_placeholder: "What are you looking for?",
      portal: "Portal",
      github: "GitHub",
      loading: "Loading documentation...",
      no_results: "No results",
      error_loading: "Error: Could not load this documentation file.",
      error_generic: "Error: An error occurred while fetching the documentation.",
      menu: {
        user_guide: "User Guide",
        manual: "User Manual",
        installation: "Installation and Setup",
        tech_docs: "Technical Documentation",
        architecture: "Architecture and APIs",
        troubleshooting: "Troubleshooting"
      }
    }
  },
  pt: {
    nav: {
      features: "Recursos",
      templates: "Templates",
      docs: "Docs",
      download: "Baixar"
    },
    hero: {
      version: "v0.3.5",
      tagline: "Um binário · 89 pacotes std · GUI de verdade",
      title_part1: "CÓDIGO",
      title_part2: "SEM LIMITES.",
      description: "Lexicon é uma linguagem com toolchain próprio num binário de 9,4 MB: run, hot reload, check, vet, fmt, gerenciador de pacotes, motor gráfico wgpu e 89 pacotes de biblioteca padrão escritos em Lex. Baixe o SDK completo, ou só a linguagem — ela instala o SDK sozinha.",
      download_sdk: "BAIXAR O SDK",
      download_lang: "SÓ A LINGUAGEM",
      repository: "REPOSITÓRIO"
    },
    features: {
      title: "Feito para Trabalho Real",
      subtitle: "Capacidades da v0.3.5 — verificadas contra o toolchain real",
      hotreload: {
        title: "Supervisor Hot Reload",
        desc: "Salvou → reiniciou com lex run --watch. Debounce de 300 ms, filtro só-.lex e backoff contra crash em vez de loop infinito."
      },
      extension: {
        title: "Super Extensão 1.2.0",
        desc: "Extensão VS Code: 117 snippets, 4 temas + icon theme, autocomplete contextual com auto-import e diagnósticos do vet ao salvar."
      },
      diagnostics: {
        title: "Diagnósticos Pré-Execução",
        desc: "Barreira de sintaxe com erros arquivo:linha:col antes de qualquer execução. lex check e lex vet mais catálogo estável de códigos E (E0101–E0801)."
      },
      engine: {
        title: "Motor gráfico em wgpu",
        desc: "O mesmo binário abre janelas: Window / Canvas / Input sobre Vulkan, DX12, Metal, OpenGL ou WebGPU. lex new -t gui ou rode o Pong dos exemplos."
      }
    },
    download: {
      title: "DOWNLOAD",
      subtitle: "Dois sabores, o mesmo toolchain",
      from_github: "no GitHub",
      button: "BAIXAR",
      sdk: {
        title: "SDK completo",
        blurb: "Tudo já extraído e pronto: o binário lex, 38 shims de launcher, 89 pacotes std, 19 exemplos que rodam, 5 templates, docs e scripts.",
        size_zip: "4,4 MB no zip",
        size_installed: "9,7 MB instalado"
      },
      lang: {
        title: "Só a linguagem",
        blurb: "Um único lex.exe. Na primeira execução ele se registra no PATH e escreve o mesmo SDK em ~/.lexicon/sdk — mesmo resultado, download menor.",
        size_zip: "4,3 MB no zip",
        size_installed: "9,4 MB instalado"
      },
      contains_sdk: {
        title: "O que tem no zip do SDK",
        items: [
          "bin/lex.exe + 38 shims (lex-run, lex-check, lex-mod, …)",
          "lib/std — 89 pacotes escritos em Lex",
          "examples — 19 programas que rodam de verdade",
          "templates — default · api · plugin · service · gui",
          "docs + scripts + LICENSE-MIT + VERSION",
          "confira com: lex sdk verify"
        ]
      },
      contains_lang: {
        title: "O que um binário só faz",
        items: [
          "~9,4 MB, um arquivo, sem runtime para instalar",
          "primeiro run: entrada no PATH + exportação do SDK (silencioso, idempotente)",
          "lex run · check · vet · fmt · mod · sdk · gui · lsp · dap",
          "remover: lex uninstall"
        ]
      },
      installer_title: "Ou deixe um script fazer isso",
      installer_windows: "Windows (PowerShell)",
      installer_posix: "Linux e macOS",
      installer_note: "O install.sh procura no release um artefato compilado para a sua plataforma; se não houver, ele compila do fonte com cargo e chama o mesmo lex install.",
      checksums: "Checksums SHA-256",
      extension: "Super extensão (.vsix)",
      all_releases: "Todas as releases e assets",
      manual: {
        title: "Instalação manual",
        steps: [
          "Extraia o zip onde quiser",
          "Rode lex.exe install",
          "Abra um terminal NOVO (o PATH é persistido por sessão)",
          "Confira com lex version"
        ]
      },
      os_title: "Plataformas",
      os_note: "A release traz binários prontos para Windows x64; nas outras plataformas o install.sh compila do fonte em ~2 min.",
      no_admin: "Não precisa de admin",
      source_title: "Compilar do fonte",
      source_note: "Toolchain Rust + o perfil dist (opt-level z + LTO gorda) — o mesmo que a gente publica."
    },
    templates: {
      title: "Comece em Segundos",
      subtitle: "Crie o scaffold na hora e rode o que é real.",
      api: "REST API",
      api_desc: "Crie com lex new api. Depois rode a demo-api real: HTTP axum + SQLite com fluxos .env dev/prod.",
      gui: "GUI / Jogo",
      gui_desc: "lex new minha-janela -t gui abre uma janela wgpu de verdade: loop, input e desenho com Canvas. O Pong já está em examples/.",
      plugin: "WASM Plugin",
      plugin_desc: "Crie com lex new plugin --target wasm. Snippets e gramática inclusos; veja nos docs o status atual do target.",
      service: "gRPC Service",
      service_desc: "Crie com lex new service --grpc. Valide cada passo com lex vet antes de rodar.",
      quickstart_title: "Quickstart real — teste agora",
      quickstart_desc: "Estes comandos funcionam no toolchain v0.3.5 hoje."
    },
    docs: {
      title: "LEXICON DOCS",
      search_placeholder: "O que você está procurando?",
      portal: "Portal",
      github: "GitHub",
      loading: "Carregando documentação...",
      no_results: "Nenhum resultado",
      error_loading: "Erro: Não foi possível carregar este arquivo de documentação.",
      error_generic: "Erro: Ocorreu um erro ao buscar a documentação.",
      menu: {
        user_guide: "Guia do Usuário",
        manual: "Manual do Usuário",
        installation: "Instalação e Setup",
        tech_docs: "Documentação Técnica",
        architecture: "Arquitetura e APIs",
        troubleshooting: "Troubleshooting"
      }
    }
  }
};

export type Lang = 'en' | 'pt';
