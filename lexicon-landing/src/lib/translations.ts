export const translations = {
  en: {
    nav: {
      features: "Features",
      templates: "Templates",
      docs: "Docs",
      install: "Install"
    },
    hero: {
      alpha: "v0.2.0",
      new_era: "Hot Reload + Super Extension",
      title_part1: "CODE",
      title_part2: "WITHOUT LIMITS.",
      description: "Lexicon v0.2.0: a modern language with a hot-reload supervisor, a VS Code super extension, and pre-run diagnostics. Save → auto-restart, catch errors before running, and serve a real HTTP + SQLite demo API.",
      download: "DOWNLOAD SDK",
      repository: "REPOSITORY"
    },
    features: {
      title: "Built for Real Work",
      subtitle: "v0.2.0 capabilities — verified against the actual toolchain",
      hotreload: {
        title: "Hot Reload Supervisor",
        desc: "Save → auto-restart with lex run --watch. 300 ms debounce, .lex-only filter, and crash backoff instead of hot-spinning."
      },
      extension: {
        title: "Super Extension 1.1.x",
        desc: "VS Code super extension: 85+ snippets, 4 themes, contextual autocomplete with auto-import, and vet diagnostics on save."
      },
      diagnostics: {
        title: "Pre-Run Diagnostics",
        desc: "Syntax gate with file:line:col errors before anything runs. lex check and lex vet plus a stable E-code catalog (E0101–E0801)."
      },
      installer: {
        title: "Honest Installer",
        desc: "Silent, idempotent setup — never duplicates PATH entries. Comment-accurate runner: commented-out code never executes."
      }
    },
    templates: {
      title: "Start in Seconds",
      subtitle: "Scaffold instantly, then run the real thing.",
      api: "REST API",
      api_desc: "Scaffold with lex new api --edge. Then run the real demo-api: axum HTTP + SQLite with dev/prod .env flows.",
      plugin: "WASM Plugin",
      plugin_desc: "Scaffold with lex new plugin --target wasm. Snippets and grammar included; check the docs for current target status.",
      service: "gRPC Service",
      service_desc: "Scaffold with lex new service --grpc. Validate every step with lex vet before you run.",
      quickstart_title: "Real quickstart — try it now",
      quickstart_desc: "These commands work against the v0.2.0 toolchain today."
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
      install: "Instalar"
    },
    hero: {
      alpha: "v0.2.0",
      new_era: "Hot Reload + Super Extensão",
      title_part1: "CÓDIGO",
      title_part2: "SEM LIMITES.",
      description: "Lexicon v0.2.0: linguagem moderna com supervisor de hot-reload, super extensão para VS Code e diagnósticos pré-execução. Salvou → reiniciou, erros capturados antes de rodar, e uma demo API real com HTTP + SQLite.",
      download: "BAIXAR SDK",
      repository: "REPOSITÓRIO"
    },
    features: {
      title: "Feito para Trabalho Real",
      subtitle: "Capacidades da v0.2.0 — verificadas contra o toolchain real",
      hotreload: {
        title: "Supervisor Hot Reload",
        desc: "Salvou → reiniciou com lex run --watch. Debounce de 300 ms, filtro só-.lex e backoff contra crash em vez de loop infinito."
      },
      extension: {
        title: "Super Extensão 1.1.x",
        desc: "Super extensão VS Code: 85+ snippets, 4 temas, autocomplete contextual com auto-import e diagnósticos do vet ao salvar."
      },
      diagnostics: {
        title: "Diagnósticos Pré-Execução",
        desc: "Barreira de sintaxe com erros arquivo:linha:col antes de qualquer execução. lex check e lex vet mais catálogo estável de códigos E (E0101–E0801)."
      },
      installer: {
        title: "Instalador Honesto",
        desc: "Setup silencioso e idempotente — nunca duplica entradas do PATH. Runner fiel a comentários: código comentado nunca executa."
      }
    },
    templates: {
      title: "Comece em Segundos",
      subtitle: "Crie o scaffold na hora e rode o que é real.",
      api: "REST API",
      api_desc: "Crie com lex new api --edge. Depois rode a demo-api real: HTTP axum + SQLite com fluxos .env dev/prod.",
      plugin: "WASM Plugin",
      plugin_desc: "Crie com lex new plugin --target wasm. Snippets e gramática inclusos; veja nos docs o status atual do target.",
      service: "gRPC Service",
      service_desc: "Crie com lex new service --grpc. Valide cada passo com lex vet antes de rodar.",
      quickstart_title: "Quickstart real — teste agora",
      quickstart_desc: "Estes comandos funcionam no toolchain v0.2.0 hoje."
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
