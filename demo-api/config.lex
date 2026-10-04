// config.lex — configuração via variáveis de ambiente.
// Valide com: lex vet config.lex
// Em execução, `Env::get` lê o ambiente REAL do processo
// (exportado pelos scripts run-dev.ps1 / run-prod.ps1).

// Lê APP_ENV ("development" ou "production").
pub fn app_env() -> String {
    let env = Env::get("APP_ENV");
    return env;
}

// Lê DATABASE_URL ("sqlite:./dev.db" ou "sqlite:./prod.db").
pub fn db_url() -> String {
    let url = Env::get("DATABASE_URL");
    return url;
}

// Lê LOG_LEVEL ("debug" ou "warn").
pub fn log_level() -> String {
    let level = Env::get("LOG_LEVEL");
    return level;
}
