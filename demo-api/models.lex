// models.lex — tipos e funções de domínio (importado por routes/store).
// Valide com: lex vet models.lex

// Exportado com `pub`: visível para outros arquivos via `import app::models`.
pub struct User {
    id: int;
    name: String;
    email: String;
}

// Alias de tipo exportado.
pub type UserId = int;

// Função exportada: monta um rótulo "user:<nome>".
pub fn user_label(name: String) -> String {
    return "user:" + name;
}

// Função privada (sem `pub`): só existe dentro deste arquivo.
fn normalize_email(email: String) -> String {
    return email;
}
