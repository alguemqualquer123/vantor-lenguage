//! Gerenciador de pacotes do Lex (`lex mod …`) — Spec §12/§82, estilo Go.
//!
//! Fontes suportadas:
//! - `github:owner/repo[@tag|@branch|#sha]` (shorthand → github.com)
//! - `gitlab:owner/repo[...]` (shorthand → gitlab.com)
//! - URLs git diretas (`https://…`, `git@…`)
//! - pastas locais (`path:../dir` ou caminho existente)
//!
//! Modelo (primeiros anos do Go, sem servidor central):
//! - Cache global `~/.lexicon/pkg/<owner>_<repo>@<ref>`
//! - Materialização no projeto em `lex_packages/<dono>_<repo>/…` (a partir
//!   de `src/` ou da raiz com `lib/`)
//! - `lexicon.lock` fixa (fonte, ref resolvida, commit, data) → builds
//!   reproduzíveis; `lex mod install` restaura do lock.
//!
//! O loader de módulos do interpretador (`find_module_file` em `interp.rs`)
//! já resolve `import a::b` dentro de `lex_packages/`, então nada mais
//! precisa mudar para que os pacotes baixados funcionem.

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Serialize, Deserialize};

const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_YELLOW: &str = "\x1b[33m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

/// Uma dependência do `lexicon.toml` (`[dependencies]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dep {
    /// Fonte declarada: `github:o/r@v`, `path:../x`, URL git.
    pub source: String,
}

/// Entrada do `lexicon.lock` (fonte + ref resolvida).
#[derive(Debug, Clone, Default)]
pub struct Locked {
    pub source: String,
    pub resolved: String,
    pub commit: Option<String>,
    pub fetched_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Manifesto (`lexicon.toml`) — parse/serialize mínimos por linha
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct Manifest {
    name: String,
    version: String,
    deps: Vec<(String, Dep)>,
}

fn manifest_path() -> PathBuf {
    PathBuf::from("lexicon.toml")
}

fn read_manifest() -> Result<Manifest> {
    let p = manifest_path();
    if !p.exists() {
        anyhow::bail!("no lexicon.toml here — run `lex mod init <nome>` first");
    }
    let text = std::fs::read_to_string(&p)?;
    Ok(parse_manifest(&text))
}

fn parse_manifest(text: &str) -> Manifest {
    let mut m = Manifest::default();
    let mut in_deps = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = line.eq_ignore_ascii_case("[dependencies]");
            continue;
        }
        if in_deps {
            if let Some((k, v)) = line.split_once('=') {
                let key = k.trim().trim_matches('"').to_string();
                let raw = v.trim().trim_matches('"').to_string();
                if !key.is_empty() {
                    m.deps.push((key, Dep { source: raw }));
                }
            }
        } else if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let val = v.trim().trim_matches('"');
            if key == "name" {
                m.name = val.to_string();
            } else if key == "version" {
                m.version = val.to_string();
            }
        }
    }
    m
}

fn write_manifest(m: &Manifest) -> Result<()> {
    let mut out = String::new();
    out.push_str(&format!("[project]\nname = \"{}\"\nversion = \"{}\"\n\n", m.name, m.version));
    if m.deps.is_empty() {
        out.push_str("[dependencies]\n");
    } else {
        out.push_str("[dependencies]\n");
        for (k, d) in &m.deps {
            out.push_str(&format!("{} = \"{}\"\n", k, d.source));
        }
    }
    std::fs::write(manifest_path(), out)?;
    Ok(())
}

fn lock_path() -> PathBuf {
    PathBuf::from("lexicon.lock")
}

/// Lock serializado à mão (sem dep `toml`): seções `[nome]` com strings.
fn read_lock() -> Result<std::collections::BTreeMap<String, Locked>> {
    let p = lock_path();
    if !p.exists() {
        return Ok(Default::default());
    }
    let text = std::fs::read_to_string(&p)?;
    let mut map: std::collections::BTreeMap<String, Locked> = Default::default();
    let mut cur: Option<String> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let k = line[1..line.len() - 1].trim().to_string();
            cur = Some(k.clone());
            map.insert(k, Locked::default());
            continue;
        }
        if let (Some(k), Some((raw_key, raw_val))) = (cur.as_ref(), line.split_once('=')) {
            let val = raw_val.trim().trim_matches('"').to_string();
            if let Some(entry) = map.get_mut(k) {
                match raw_key.trim() {
                    "source" => entry.source = val,
                    "resolved" => entry.resolved = val,
                    "commit" => entry.commit = Some(val),
                    "fetched_at" => entry.fetched_at = Some(val),
                    _ => {}
                }
            }
        }
    }
    Ok(map)
}

fn write_lock(map: &std::collections::BTreeMap<String, Locked>) -> Result<()> {
    let mut body = String::from("# Gerado por `lex mod`. NÃO edite à mão — builds reproduzíveis.\n");
    for (name, l) in map {
        body.push_str(&format!("\n[{}]\n", name));
        body.push_str(&format!("source = {:?}\n", l.source));
        body.push_str(&format!("resolved = {:?}\n", l.resolved));
        if let Some(c) = &l.commit {
            body.push_str(&format!("commit = {:?}\n", c));
        }
        if let Some(t) = &l.fetched_at {
            body.push_str(&format!("fetched_at = {:?}\n", t));
        }
    }
    std::fs::write(lock_path(), body)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Fontes de pacote
// ---------------------------------------------------------------------------

/// Uma fonte resolvida de dependência.
#[derive(Debug, Clone)]
enum Src {
    Git { url: String, r#ref: String },
    Path(PathBuf),
}

fn parse_source(src: &str) -> Src {
    let s = src.trim();
    // Shorthands: github:owner/repo[@ref|#sha], gitlab:...
    for (prefix, host) in [("github:", "github.com"), ("gitlab:", "gitlab.com")] {
        if let Some(rest) = s.strip_prefix(prefix) {
            let (repo, r#ref) = split_ref(rest);
            return Src::Git {
                url: format!("https://{}/{}", host, repo),
                r#ref,
            };
        }
    }
    // URL git direta.
    if s.starts_with("https://")
        || s.starts_with("http://")
        || s.starts_with("git@")
        || s.starts_with("file://")
    {
        let (url, r#ref) = split_ref(s);
        return Src::Git { url, r#ref };
    }
    // Pasta local (`path:` explícito ou caminho existente).
    let p = s.strip_prefix("path:").unwrap_or(s);
    Src::Path(PathBuf::from(p))
}

fn split_ref(rest: &str) -> (String, String) {
    match rest.rfind(['@', '#']) {
        Some(i) if i > 0 => (rest[..i].to_string(), rest[i + 1..].to_string()),
        _ => (rest.to_string(), "HEAD".to_string()),
    }
}

/// Nome de pacote a partir da fonte: `owner_repo` (git) ou o nome da pasta.
fn pkg_name(src: &Src) -> String {
    match src {
        Src::Git { url, .. } => {
            let clean = url.trim_end_matches(".git");
            let tail = clean.rsplit('/').next().unwrap_or("pkg");
            let owner = clean
                .rsplit('/')
                .nth(1)
                .unwrap_or("local")
                .replace('.', "-");
            format!("{}_{}", owner, tail)
        }
        Src::Path(p) => p
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "local".to_string()),
    }
}

fn pkg_cache_root() -> PathBuf {
    dirs_home().join(".lexicon").join("pkg")
}

fn dirs_home() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn now_stamp() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", d)
}

// ---------------------------------------------------------------------------
// Fetch / materialização
// ---------------------------------------------------------------------------

/// Clona/atualiza o cache global e devolve (dir_cacheado, commit).
fn fetch_git(url: &str, r#ref: &str) -> Result<(PathBuf, Option<String>)> {
    let clean = url.trim_end_matches(".git").to_string();
    let tail = clean.rsplit('/').next().unwrap_or("repo").to_string();
    let owner = clean.rsplit('/').nth(1).unwrap_or("local").replace('.', "-");
    let cache = pkg_cache_root().join(format!("{}_{}@{}", owner, tail, r#ref));

    if !cache.join(".git").exists() {
        std::fs::create_dir_all(&cache)?;
        let st = Command::new("git")
            .args(["clone", "--quiet"])
            .arg(url)
            .arg(&cache)
            .status();
        if !st.map(|s| s.success()).unwrap_or(false) {
            let _ = std::fs::remove_dir_all(&cache);
            anyhow::bail!("git clone {} falhou (git instalado? rede ok?)", url);
        }
    }
    let run_git = |args: &[&str]| -> Result<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&cache)
            .args(args)
            .output()?;
        if !out.status.success() {
            anyhow::bail!(
                "git {} falhou: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    run_git(&["fetch", "--quiet", "origin"])?;
    run_git(&["checkout", "--quiet", r#ref])?;
    run_git(&["pull", "--quiet", "--ff-only"])?;
    let commit = run_git(&["rev-parse", "HEAD"]).ok();
    Ok((cache, commit))
}

/// Resolve fontes locais: absolutas passam direto; relativas juntam ao cwd;
/// no Windows, caminhos estilo Git Bash/MSYS (`/tmp/...`, `/c/...`) são
/// traduzidos para o caminho Windows correspondente.
fn resolve_local(p: &Path) -> PathBuf {
    if p.is_absolute() {
        return p.to_path_buf();
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    #[cfg(windows)]
    let s = p.to_string_lossy().into_owned();
    #[cfg(windows)]
    if s.starts_with('/') {
        let rest = s.trim_start_matches('/');
        // `/tmp/x` → `%TEMP%\x` (Git Bash/MSYS2 mapeia /tmp para o temp).
        if rest == "tmp" {
            return std::env::temp_dir();
        }
        if let Some(sub) = rest.strip_prefix("tmp/") {
            return std::env::temp_dir().join(sub);
        }
        // `/c/Users/...` → `C:/Users/...` (convenção de drive MSYS).
        let mut parts = rest.splitn(2, '/');
        if let Some(d) = parts.next() {
            if d.len() == 1 && d.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
                let base = format!("{}:/", d.to_ascii_uppercase());
                return match parts.next() {
                    Some(rem) => PathBuf::from(&base).join(rem),
                    None => PathBuf::from(base),
                };
            }
        }
        // Fallback: mesmo drive do cwd (root-relative).
        if let Some(drive) = cwd.to_str().and_then(|c| c.get(..2)) {
            return PathBuf::from(format!("{}{}", drive, s));
        }
    }
    cwd.join(p)
}

/// Materializa um pacote no projeto: `lex_packages/<nome>/…` com `src/`
/// ou `lib/` na raiz do pacote. Devolve quantos arquivos `.lex` entraram.
fn materialize(src: &Src, cache_dir: Option<&Path>) -> Result<(String, usize)> {
    let name = pkg_name(src);
    let dest = PathBuf::from("lex_packages").join(&name);
    let origin: PathBuf = match (src, cache_dir) {
        (Src::Git { .. }, Some(c)) => c.to_path_buf(),
        (Src::Path(p), _) => resolve_local(p),
        _ => anyhow::bail!("cache do pacote indisponível"),
    };
    if !origin.is_dir() {
        anyhow::bail!("pacote de origem não encontrado: {}", origin.display());
    }

    // Cabeçalho com origem (para debug/navegação no IDE).
    std::fs::create_dir_all(&dest)?;
    std::fs::write(
        dest.join("lex-pkg.toml"),
        format!("# pacote materializado de {:?}\n", origin.display()),
    )?;

    let mut count = 0usize;
    let mut copy_tree = |from_root: &Path| {
        let mut stack = vec![from_root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else {
                continue;
            };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    let fname = e.file_name().to_string_lossy().into_owned();
                    if fname != ".git" {
                        stack.push(p);
                    }
                    continue;
                }
                if p.extension().map(|x| x == "lex").unwrap_or(false) {
                    let rel = p.strip_prefix(from_root).unwrap_or(&p);
                    // Convenção: código em `src/` do pacote vira a raiz do
                    // pacote materializado (`lex_packages/<pkg>/x.lex`),
                    // então `import <pkg>::x;` funciona direto.
                    let rel: PathBuf = {
                        let mut it = rel.components();
                        if it.next().map(|c| c.as_os_str() == "src").unwrap_or(false)
                        {
                            it.collect()
                        } else {
                            rel.to_path_buf()
                        }
                    };
                    let to = dest.join(rel);
                    if let Some(parent) = to.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if std::fs::copy(&p, &to).is_ok() {
                        count += 1;
                    }
                }
            }
        }
    };
    // Projeto-lib pode expor código em `src/`, `lib/` ou na raiz — `src/`
    // é achatado (convenção: `import <pkg>::x` resolve em `lex_packages/<pkg>/x.lex`).
    copy_tree(&origin);
    if count == 0 {
        anyhow::bail!(
            "nenhum arquivo .lex em {} (o pacote expõe código em src/ ou lib/?)",
            origin.display()
        );
    }
    Ok((name, count))
}

// ---------------------------------------------------------------------------
// Comandos
// ---------------------------------------------------------------------------

/// `lex mod <subcomando>` — roteador chamado do main.rs.
pub fn cmd(args: Vec<String>) -> Result<()> {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("help");
    let rest = &args[1.min(args.len())..];
    match sub {
        "init" => init(rest.first().map(|s| s.as_str())),
        "add" => add(rest.first().map(|s| s.as_str())),
        "install" => install(),
        "remove" | "rm" => remove(rest.first().map(|s| s.as_str())),
        "tidy" => tidy(),
        "list" => list(),
        "graph" => graph(),
        "verify" => verify(rest.first().map(|s| s.as_str())),
        _ => help(),
    }
}

fn help() -> Result<()> {
    println!("{}lex mod — gerenciador de pacotes{}", COLOR_CYAN, RESET);
    println!("  init <nome>              cria lexicon.toml");
    println!("  add <fonte>              adiciona dependência e instala");
    println!("  install                  restaura tudo do lexicon.lock");
    println!("  remove <nome>            remove dependência");
    println!("  tidy                     sincroniza toml/lock/lex_packages");
    println!("  list                     dependências declaradas");
    println!("  graph                    árvore de imports do projeto");
    println!("  verify [arquivo]         checa se imports resolvem");
    println!();
    println!("Fontes: github:owner/repo[@tag|@branch|#sha] · gitlab:… · https://… · path:../dir");
    println!("Pacotes entram em lex_packages/ · versões fixadas em lexicon.lock");
    Ok(())
}

fn init(name: Option<&str>) -> Result<()> {
    let name = name.unwrap_or("meu-projeto").trim_matches('"').to_string();
    if manifest_path().exists() {
        println!("{}mod init: lexicon.toml já existe{}", COLOR_YELLOW, RESET);
        return Ok(());
    }
    write_manifest(&Manifest {
        name: name.clone(),
        version: "0.1.0".into(),
        deps: Vec::new(),
    })?;
    std::fs::create_dir_all("src")?;
    if !PathBuf::from("src/main.lex").exists() {
        std::fs::write(
            PathBuf::from("src/main.lex"),
            "pub fn main() -> void {\n    Console::log(\"olá, \" + \"Lexicon!\");\n}\n",
        )?;
    }
    println!("{}init:{} pacote {} v0.1.0 criado (src/main.lex)", COLOR_GREEN, RESET, name);
    Ok(())
}

fn add(source: Option<&str>) -> Result<()> {
    let Some(source) = source else {
        anyhow::bail!("uso: lex mod add <github:owner/repo|path:../dir>");
    };
    let src = parse_source(source);
    let name = pkg_name(&src);

    // Fetch + materializa já na adição (falha cedo).
    let (commit, files) = match &src {
        Src::Git { url, r#ref } => {
            let (cache, commit) = fetch_git(url, r#ref)?;
            let (_n, files) = materialize(&src, Some(&cache))?;
            (commit, files)
        }
        Src::Path(_) => {
            let (_n, files) = materialize(&src, None)?;
            (None, files)
        }
    };

    let mut m = read_manifest()?;
    if !m.deps.iter().any(|(k, _)| *k == name) {
        m.deps.push((name.clone(), Dep { source: source.to_string() }));
        write_manifest(&m)?;
    }

    let mut lock = read_lock()?;
    lock.insert(
        name.clone(),
        Locked {
            source: source.to_string(),
            resolved: match &src {
                Src::Git { url, r#ref } => format!("{}@{}", url, r#ref),
                Src::Path(p) => p.display().to_string(),
            },
            commit,
            fetched_at: Some(now_stamp()),
        },
    );
    write_lock(&lock)?;

    println!(
        "{}add:{} {} ({} arquivo(s) .lex → lex_packages/{})",
        COLOR_GREEN, RESET, name, files, name
    );
    Ok(())
}

fn install() -> Result<()> {
    let lock = read_lock()?;
    if lock.is_empty() {
        let m = read_manifest()?;
        if m.deps.is_empty() {
            println!("{}install: sem dependências{}", COLOR_YELLOW, RESET);
            return Ok(());
        }
        // Sem lock: deriva das fontes do toml (compatibilidade).
        let mut new_lock = lock;
        for (name, dep) in &m.deps {
            let src = parse_source(&dep.source);
            let (commit, files) = match &src {
                Src::Git { url, r#ref } => {
                    let (cache, c) = fetch_git(url, r#ref)?;
                    let (_n, files) = materialize(&src, Some(&cache))?;
                    (c, files)
                }
                Src::Path(_) => {
                    let (_n, files) = materialize(&src, None)?;
                    (None, files)
                }
            };
            println!("{}install:{} {} ({} arquivo(s))", COLOR_GREEN, RESET, name, files);
            new_lock.insert(
                name.clone(),
                Locked {
                    source: dep.source.clone(),
                    resolved: match &src {
                        Src::Git { url, r#ref } => format!("{}@{}", url, r#ref),
                        Src::Path(p) => p.display().to_string(),
                    },
                    commit,
                    fetched_at: Some(now_stamp()),
                },
            );
        }
        write_lock(&new_lock)?;
        return Ok(());
    }
    let mut total = 0usize;
    for (name, l) in &lock {
        let src = parse_source(&l.source);
        let files = match &src {
            Src::Git { url, r#ref } => {
                let (cache, _c) = fetch_git(url, r#ref)?;
                materialize(&src, Some(&cache))?.1
            }
            Src::Path(_) => materialize(&src, None)?.1,
        };
        total += files;
        println!("{}install:{} {} ({} arquivo(s))", COLOR_GREEN, RESET, name, files);
    }
    println!(
        "{}install: {} pacote(s), {} arquivo(s){}",
        COLOR_GREEN, lock.len(), total, RESET
    );
    Ok(())
}

fn remove(name: Option<&str>) -> Result<()> {
    let Some(name) = name else {
        anyhow::bail!("uso: lex mod remove <nome-do-pacote>");
    };
    let mut m = read_manifest()?;
    let before = m.deps.len();
    m.deps.retain(|(k, _)| k != name);
    if m.deps.len() == before {
        println!(
            "{}remove: `{}` não está no lexicon.toml{}",
            COLOR_YELLOW, name, RESET
        );
        return Ok(());
    }
    write_manifest(&m)?;
    let mut lock = read_lock()?;
    lock.remove(name);
    write_lock(&lock)?;
    let dir = PathBuf::from("lex_packages").join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    println!("{}remove:{} {} removido", COLOR_GREEN, RESET, name);
    Ok(())
}

/// `tidy`: remove pacotes não-declarados do toml e garante lock+materialização.
fn tidy() -> Result<()> {
    let m = read_manifest()?;
    let mut lock = read_lock()?;

    // 1) Lock entries sem dependência declarada → fora (lock e disco).
    let declared: std::collections::HashSet<&String> = m.deps.iter().map(|(k, _)| k).collect();
    let stale: Vec<String> = lock
        .keys()
        .filter(|k| !declared.contains(*k))
        .cloned()
        .collect();
    for k in &stale {
        lock.remove(k);
        let dir = PathBuf::from("lex_packages").join(k);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        println!("{}tidy:{} removido {}", COLOR_YELLOW, k, RESET);
    }
    // Diretórios órfãos no disco (nunca declarados / apagados à mão).
    if let Ok(rd) = std::fs::read_dir("lex_packages") {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if !declared.contains(&name) && e.path().is_dir() {
                let _ = std::fs::remove_dir_all(e.path());
                println!("{}tidy:{} removido lex_packages/{}", COLOR_YELLOW, name, RESET);
            }
        }
    }

    // 2) Dependência declarada sem lock → baixa e registra.
    for (name, dep) in &m.deps {
        if !lock.contains_key(name) {
            let src = parse_source(&dep.source);
            let (commit, files) = match &src {
                Src::Git { url, r#ref } => {
                    let (cache, c) = fetch_git(url, r#ref)?;
                    let (_n, files) = materialize(&src, Some(&cache))?;
                    (c, files)
                }
                Src::Path(_) => {
                    let (_n, files) = materialize(&src, None)?;
                    (None, files)
                }
            };
            println!("{}tidy:{} adicionado {} ({} arquivo(s))", COLOR_GREEN, RESET, name, files);
            lock.insert(
                name.clone(),
                Locked {
                    source: dep.source.clone(),
                    resolved: match &src {
                        Src::Git { url, r#ref } => format!("{}@{}", url, r#ref),
                        Src::Path(p) => p.display().to_string(),
                    },
                    commit,
                    fetched_at: Some(now_stamp()),
                },
            );
        }
    }
    write_lock(&lock)?;
    println!("{}tidy: {} dependência(s) OK{}", COLOR_GREEN, m.deps.len(), RESET);
    Ok(())
}

fn list() -> Result<()> {
    let m = read_manifest()?;
    let lock = read_lock()?;
    if m.deps.is_empty() {
        println!("{}list: nenhuma dependência{}", COLOR_YELLOW, RESET);
        return Ok(());
    }
    for (name, dep) in &m.deps {
        let commit = lock.get(name).and_then(|l| l.commit.as_ref()).map(|c| {
            let short: String = c.chars().take(9).collect();
            format!(" @ {}", short)
        }).unwrap_or_default();
        println!("  {} {} ({}){}", COLOR_CYAN, name, dep.source, commit);
    }
    Ok(())
}

/// `graph`: lista imports de `src/**/*.lex` marcando os que vêm de pacotes.
fn graph() -> Result<()> {
    let pkgs = PathBuf::from("lex_packages");
    let mut imports: std::collections::BTreeSet<String> = Default::default();
    let mut stack = vec![PathBuf::from("src"), PathBuf::from(".")];
    let mut seen = std::collections::HashSet::new();
    while let Some(dir) = stack.pop() {
        if !seen.insert(dir.clone()) {
            continue;
        }
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                let fname = e.file_name().to_string_lossy().into_owned();
                if fname != ".git" && fname != "lex_packages" && fname != "target" && fname != "build" {
                    stack.push(p);
                }
                continue;
            }
            if p.extension().map(|x| x == "lex").unwrap_or(false) {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    for line in lexicon_lexer::strip_comments(&text).lines() {
                        let line = line.trim();
                        if let Some(rest) = line.strip_prefix("import ") {
                            imports.insert(rest.trim_end_matches(';').trim().to_string());
                        }
                    }
                }
            }
        }
    }
    println!("{}graph:{} imports do projeto (* = pacote lex_packages)", COLOR_CYAN, RESET);
    for imp in &imports {
        let first = imp.split("::").next().unwrap_or("");
        let tag = if pkgs.join(first).is_dir() { " *" } else { "" };
        println!("  {}{}", imp, tag);
    }
    Ok(())
}

/// `verify [arquivo]` — reimplementado sobre o pkg: mantém o contrato antigo.
fn verify(file: Option<&str>) -> Result<()> {
    // Delega ao verificador original (interp::find_module_file), que agora
    // conhece lex_packages/.
    crate::compiler::mod_verify(file.map(|p| p.to_string()))
}

// ---------------------------------------------------------------------------
// Testes (funções puras — o fluxo completo roda no e2e)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_source_github_shorthand_with_ref() {
        let Src::Git { url, r#ref } = parse_source("github:owner/repo@v1.2.0") else {
            panic!("esperava fonte git");
        };
        assert_eq!(url, "https://github.com/owner/repo");
        assert_eq!(r#ref, "v1.2.0");
    }

    #[test]
    fn parse_source_github_shorthand_with_commit() {
        let Src::Git { url, r#ref } = parse_source("github:owner/repo#a1b2c3d") else {
            panic!("esperava fonte git");
        };
        assert_eq!(url, "https://github.com/owner/repo");
        assert_eq!(r#ref, "a1b2c3d");
    }

    #[test]
    fn parse_source_bare_github_defaults_to_head() {
        let Src::Git { url, r#ref } = parse_source("github:lexicon-lang/std") else {
            panic!("esperava fonte git");
        };
        assert_eq!(url, "https://github.com/lexicon-lang/std");
        assert_eq!(r#ref, "HEAD");
    }

    #[test]
    fn parse_source_https_url_keeps_ref_split() {
        let Src::Git { url, r#ref } = parse_source("https://example.com/x.git@main") else {
            panic!("esperava fonte git");
        };
        assert_eq!(url, "https://example.com/x.git");
        assert_eq!(r#ref, "main");
    }

    #[test]
    fn parse_source_path_prefix() {
        let Src::Path(p) = parse_source("path:../mylib") else {
            panic!("esperava fonte path");
        };
        assert_eq!(p, PathBuf::from("../mylib"));
    }

    #[test]
    fn split_ref_at_sha_separator() {
        let (u, r) = split_ref("owner/repo#deadbeef");
        assert_eq!(u, "owner/repo");
        assert_eq!(r, "deadbeef");
    }

    #[test]
    fn manifest_parses_project_and_deps() {
        let m = parse_manifest(
            "[project]\nname = \"demo\"\nversion = \"0.2.0\"\n\n[dependencies]\nmylib = \"github:o/r@v1\"\nlocal = \"path:../l\"\n",
        );
        assert_eq!(m.name, "demo");
        assert_eq!(m.version, "0.2.0");
        assert_eq!(m.deps.len(), 2);
        assert_eq!(m.deps[0].0, "mylib");
        assert_eq!(m.deps[0].1.source, "github:o/r@v1");
        assert_eq!(m.deps[1].1.source, "path:../l");
    }

    #[test]
    fn pkg_name_uses_owner_and_repo() {
        let src = parse_source("github:acme/widgets");
        assert_eq!(pkg_name(&src), "acme_widgets");
    }
}
