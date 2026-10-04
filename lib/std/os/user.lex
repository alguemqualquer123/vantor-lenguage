// Lexicon Standard Library — os/user.
// Go-parity user lookup (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 3).
// Sourced from the environment (`USERNAME`/`USER`, `USERPROFILE`/`HOME`);
// domain fields stay empty outside OS account APIs (documented).
// Import as `import std::os::user;`.

/// A user account (Go's `user.User`, portable subset).
pub struct User {
    uid: String,
    gid: String,
    username: String,
    name: String,
    home: String,
}

/// Current user (Go's `user.Current`).
pub fn Current() -> User {
    let username = Env::get("USERNAME");
    if username == "NOT_FOUND" {
        username = Env::get("USER");
    }
    if username == "NOT_FOUND" {
        username = "";
    }
    return User { uid: "", gid: "", username: username, name: "", home: File::home_dir() };
}

/// Looks up `name` (Go's `user.Lookup`): hits only when it matches the
// current user (no account database in the tree-walker).
pub fn Lookup(name: String) -> Dynamic {
    let me = Current();
    if me.username != "" && me.username == name {
        return [me, true];
    }
    return [User { uid: "", gid: "", username: "", name: "", home: "" }, false];
}
