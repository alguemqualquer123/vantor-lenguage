//! Generic lifetime / borrow analysis (Spec §6 / §44).
//!
//! `LoanChecker` is deliberately AST-agnostic: `lexicon-utils` must NOT
//! depend on `lexicon-parser` (dependency cycle), so this module tracks
//! *named* loans (`&str` variable names) with lexical scopes and the
//! typeck (owned by another agent) wires AST statements into it, e.g.:
//!
//! ```ignore
//! // typeck sketch (lives outside lexicon-utils):
//! checker.enter_scope();                       // `{`
//! checker.declare("x");                        // `let x = ...`
//! checker.loan_mut("x")?;                      // `let r = &mut x`
//! checker.use_name("x")?;                      // `print(x)` — errors while mutably loaned
//! checker.move_name("x")?;                     // `consume(x)` — errors while loaned
//! checker.exit_scope();                        // `}` — loans/scoped names expire
//! ```
//!
//! Rules enforced (Rust-style diagnostics):
//! - second `loan_mut` while mutably loaned → `E0305`
//! - `loan_mut` while shared loans live (and vice versa) → `E0305`
//! - `use_name` of a moved value → `E0382` (use-after-move)
//! - `use_name` while mutably loaned → `E0305` (use-while-mutably-loaned)
//! - `move_name` of a moved value → `E0382`
//! - `move_name` while any loan lives → `E0305`
//!
//! Scope semantics: `enter_scope` pushes a lexical level; `exit_scope`
//! ends every loan taken at the exiting depth and drops every name first
//! declared at that depth. Loans taken in an outer scope survive an inner
//! `exit_scope` (they remain live until their own scope exits). There is
//! no explicit `unloan`: expiry is purely scope-driven, which keeps the
//! "concurrent contract honest" — the typeck decides region granularity.
//!
//! Ownership: `LoanChecker` owns all analysis state; callers hold no
//! borrows across calls (all APIs take `&str` + `&mut self`).
//! Thread-safety: `!Sync` by convention — run one checker per typeck
//! invocation on a single thread; share across threads only behind a lock.
//! Complexity: all ops are O(1) amortised (`HashMap` lookup).

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct VarState {
    declared_depth: usize,
    mut_loan_depth: Option<usize>,
    shared_loan_depths: Vec<usize>,
    moved: bool,
}

impl VarState {
    fn new(declared_depth: usize) -> Self {
        VarState {
            declared_depth,
            mut_loan_depth: None,
            shared_loan_depths: Vec::new(),
            moved: false,
        }
    }

    fn is_mutably_loaned(&self) -> bool {
        self.mut_loan_depth.is_some()
    }

    fn shared_count(&self) -> usize {
        self.shared_loan_depths.len()
    }

    fn is_loaned(&self) -> bool {
        self.is_mutably_loaned() || !self.shared_loan_depths.is_empty()
    }
}

/// Scope-aware borrow checker over named variables.
///
/// See the [module](self) docs for the typeck wiring contract.
#[derive(Debug, Default)]
pub struct LoanChecker {
    vars: HashMap<String, VarState>,
    /// Names first declared at each scope depth; `scopes.len() - 1` is the
    /// current depth. Index 0 is the root scope.
    scopes: Vec<HashSet<String>>,
}

impl LoanChecker {
    pub fn new() -> Self {
        LoanChecker {
            vars: HashMap::new(),
            scopes: vec![HashSet::new()],
        }
    }

    fn depth(&self) -> usize {
        self.scopes.len() - 1
    }

    /// Push a new lexical scope (e.g. a block, loop body, function body).
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    /// Pop the current scope: loans taken at the exiting depth end and
    /// names declared there go out of scope. Popping the root scope is a
    /// no-op (keeps the checker reusable across top-level items).
    pub fn exit_scope(&mut self) {
        if self.scopes.len() <= 1 {
            return;
        }
        let exiting = self.scopes.len() - 1;
        let names = self.scopes.pop().expect("scope stack underflow");
        // End loans taken at the exiting depth.
        for var in self.vars.values_mut() {
            if var.mut_loan_depth == Some(exiting) {
                var.mut_loan_depth = None;
            }
            var.shared_loan_depths.retain(|d| *d != exiting);
        }
        // Drop names declared in the exiting scope.
        for name in names {
            self.vars.remove(&name);
        }
    }

    /// Declare `name` in the current scope. Idempotent within a scope.
    pub fn declare(&mut self, name: &str) {
        let depth = self.depth();
        if !self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), VarState::new(depth));
            self.scopes[depth].insert(name.to_string());
        }
    }

    fn ensure(&mut self, name: &str) -> &mut VarState {
        let depth = self.depth();
        if !self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), VarState::new(depth));
            self.scopes[depth].insert(name.to_string());
        }
        self.vars.get_mut(name).expect("just inserted")
    }

    /// Take a mutable loan (`&mut name`). O(1).
    pub fn loan_mut(&mut self, name: &str) -> Result<(), String> {
        let depth = self.depth();
        let st = self.ensure(name);
        if st.moved {
            return Err(format!("E0382: use of moved value `{}`", name));
        }
        if st.is_mutably_loaned() {
            return Err(format!(
                "E0305: second mutable loan of `{}` while first loan is live",
                name
            ));
        }
        if st.shared_count() > 0 {
            return Err(format!(
                "E0305: cannot mutably loan `{}` while {} shared loan(s) are live",
                name,
                st.shared_count()
            ));
        }
        st.mut_loan_depth = Some(depth);
        Ok(())
    }

    /// Take a shared loan (`&name`). O(1). Multiple shared loans coexist.
    pub fn loan_shared(&mut self, name: &str) -> Result<(), String> {
        let depth = self.depth();
        let st = self.ensure(name);
        if st.moved {
            return Err(format!("E0382: use of moved value `{}`", name));
        }
        if st.is_mutably_loaned() {
            return Err(format!(
                "E0305: cannot shared-loan `{}` while mutably loaned",
                name
            ));
        }
        st.shared_loan_depths.push(depth);
        Ok(())
    }

    /// Read/use `name` (e.g. `print(x)`). O(1).
    pub fn use_name(&self, name: &str) -> Result<(), String> {
        match self.vars.get(name) {
            None => Ok(()), // Unknown names are outside the checker (e.g. globals).
            Some(st) if st.moved => Err(format!("E0382: use of moved value `{}`", name)),
            Some(st) if st.is_mutably_loaned() => Err(format!(
                "E0305: use of `{}` while mutably loaned",
                name
            )),
            Some(_) => Ok(()),
        }
    }

    /// Move `name` (e.g. `consume(x)`). O(1).
    pub fn move_name(&mut self, name: &str) -> Result<(), String> {
        let st = self.ensure(name);
        if st.moved {
            return Err(format!("E0382: use of moved value `{}` (already moved)", name));
        }
        if st.is_loaned() {
            return Err(format!(
                "E0305: cannot move `{}` while loaned",
                name
            ));
        }
        st.moved = true;
        Ok(())
    }

    /// Introspection for the typeck / tests: is `name` moved?
    pub fn is_moved(&self, name: &str) -> bool {
        self.vars.get(name).map(|s| s.moved).unwrap_or(false)
    }

    /// Introspection: is `name` currently mutably loaned?
    pub fn is_mutably_loaned(&self, name: &str) -> bool {
        self.vars.get(name).map(|s| s.is_mutably_loaned()).unwrap_or(false)
    }

    /// Introspection: how many shared loans are live on `name`?
    pub fn shared_count(&self, name: &str) -> usize {
        self.vars.get(name).map(|s| s.shared_count()).unwrap_or(0)
    }

    /// Names currently tracked (declared and still in scope).
    pub fn tracked_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.vars.keys().cloned().collect();
        names.sort();
        names
    }
}

// ---------------------------------------------------------------------------
// Part X — ownership / borrow checker public surface (Spec §6 + §44).
//
// `BorrowState` is the canonical checker name used by the typeck and
// tooling; it is the same scope-aware engine as `LoanChecker` (no
// second implementation to drift). The rules below are the documented
// ownership contract; violations carry stable codes so diagnostics,
// lints and IDE hints stay versioned (Spec §70).
// ---------------------------------------------------------------------------

/// Canonical borrow-checker state (alias of [`LoanChecker`]).
pub type BorrowState = LoanChecker;

/// Documented ownership/borrow rules (stable, user-facing).
pub fn borrow_rules_doc() -> &'static str {
    "ownership/borrow rules: (1) each value has one owner; moves transfer ownership \
     (use-after-move is E0382); (2) at most one live `&mut` loan per value (second \
     `&mut` is E0305); (3) `&mut` excludes `&` and vice versa (E0305); (4) multiple \
     `&` loans may coexist; (5) use-while-mutably-loaned is E0305; (6) moves while \
     loaned are E0305; (7) loans end when their scope exits (scope-driven expiry, \
     no explicit unloan); (8) unknown/globals are outside the checker."
}

/// Ownership rule violated by a raw checker message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorrowRule {
    /// Second `&mut` / `&`+`&mut` mix / use-while-loaned / move-while-loaned.
    ExclusiveLoan,
    /// Use or move of a moved value.
    UseAfterMove,
}

impl BorrowRule {
    pub fn code(self) -> &'static str {
        match self {
            BorrowRule::ExclusiveLoan => "E0305",
            BorrowRule::UseAfterMove => "E0382",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            BorrowRule::ExclusiveLoan => "exclusive-loan violation (aliasing or use/move while loaned)",
            BorrowRule::UseAfterMove => "use-after-move (ownership already transferred)",
        }
    }
}

/// Programmatic borrow diagnostic for tooling/IDE sinks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorrowDiagnostic {
    pub code: &'static str,
    pub rule: BorrowRule,
    pub variable: String,
    pub message: String,
}

impl BorrowDiagnostic {
    pub fn render(&self) -> String {
        format!("[{}] {}: {}", self.code, self.variable, self.message)
    }
}

/// Classify a raw [`LoanChecker`] error into a [`BorrowDiagnostic`].
/// Returns `None` when `err` carries no variable name (never for
/// checker-produced messages, which always name the variable).
pub fn diagnose_borrow_error(err: &str) -> Option<BorrowDiagnostic> {
    let rule = if err.contains("E0382") {
        BorrowRule::UseAfterMove
    } else if err.contains("E0305") {
        BorrowRule::ExclusiveLoan
    } else {
        return None;
    };
    // Checker messages always format the variable as `` `name` ``.
    let variable = {
        let mut name = String::new();
        let mut chars = err.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '`' {
                for c2 in chars.by_ref() {
                    if c2 == '`' {
                        break;
                    }
                    name.push(c2);
                }
                break;
            }
        }
        name
    };
    if variable.is_empty() {
        return None;
    }
    Some(BorrowDiagnostic {
        code: rule.code(),
        rule,
        variable,
        message: err.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_mut_loan_rejected() {
        let mut ck = LoanChecker::new();
        ck.declare("x");
        ck.loan_mut("x").unwrap();
        let err = ck.loan_mut("x").unwrap_err();
        assert!(err.contains("E0305"), "got: {}", err);
        assert!(err.contains("x"));
    }

    #[test]
    fn mut_then_shared_rejected_and_vice_versa() {
        let mut ck = LoanChecker::new();
        ck.declare("a");
        ck.loan_mut("a").unwrap();
        let err = ck.loan_shared("a").unwrap_err();
        assert!(err.contains("E0305"), "got: {}", err);

        let mut ck2 = LoanChecker::new();
        ck2.declare("b");
        ck2.loan_shared("b").unwrap();
        let err2 = ck2.loan_mut("b").unwrap_err();
        assert!(err2.contains("E0305"), "got: {}", err2);
    }

    #[test]
    fn shared_loans_coexist_and_allow_use() {
        let mut ck = LoanChecker::new();
        ck.declare("v");
        ck.loan_shared("v").unwrap();
        ck.loan_shared("v").unwrap();
        assert_eq!(ck.shared_count("v"), 2);
        ck.use_name("v").unwrap();
    }

    #[test]
    fn use_while_mutably_loaned_rejected() {
        let mut ck = LoanChecker::new();
        ck.declare("x");
        ck.loan_mut("x").unwrap();
        let err = ck.use_name("x").unwrap_err();
        assert!(err.contains("E0305"), "got: {}", err);
    }

    #[test]
    fn move_then_use_is_use_after_move() {
        let mut ck = LoanChecker::new();
        ck.declare("x");
        ck.move_name("x").unwrap();
        assert!(ck.is_moved("x"));
        let err = ck.use_name("x").unwrap_err();
        assert!(err.contains("E0382"), "got: {}", err);
        let err2 = ck.move_name("x").unwrap_err();
        assert!(err2.contains("E0382"), "got: {}", err2);
        // Loans after move are also rejected as moved.
        let err3 = ck.loan_mut("x").unwrap_err();
        assert!(err3.contains("E0382"), "got: {}", err3);
    }

    #[test]
    fn move_while_loaned_rejected() {
        let mut ck = LoanChecker::new();
        ck.declare("x");
        ck.loan_shared("x").unwrap();
        let err = ck.move_name("x").unwrap_err();
        assert!(err.contains("E0305"), "got: {}", err);

        let mut ck2 = LoanChecker::new();
        ck2.declare("y");
        ck2.loan_mut("y").unwrap();
        let err2 = ck2.move_name("y").unwrap_err();
        assert!(err2.contains("E0305"), "got: {}", err2);
    }

    #[test]
    fn inner_scope_exit_ends_inner_loans_only() {
        let mut ck = LoanChecker::new();
        ck.declare("x");
        ck.enter_scope();
        ck.loan_mut("x").unwrap();
        assert!(ck.is_mutably_loaned("x"));
        ck.exit_scope();
        // Inner loan expired; outer `x` survives and can be loaned again.
        assert!(!ck.is_mutably_loaned("x"));
        ck.loan_mut("x").unwrap();
        ck.use_name("x").unwrap_err();
    }

    #[test]
    fn names_declared_in_inner_scope_drop_on_exit() {
        let mut ck = LoanChecker::new();
        ck.enter_scope();
        ck.declare("tmp");
        ck.loan_shared("tmp").unwrap();
        ck.exit_scope();
        // `tmp` is gone; a fresh `tmp` starts unloaned and unmoved.
        assert_eq!(ck.shared_count("tmp"), 0);
        assert!(!ck.is_moved("tmp"));
        ck.use_name("tmp").unwrap();
        ck.move_name("tmp").unwrap();
    }

    #[test]
    fn outer_loan_survives_inner_exit() {
        let mut ck = LoanChecker::new();
        ck.declare("x");
        ck.loan_shared("x").unwrap();
        ck.enter_scope();
        ck.declare("y");
        ck.exit_scope();
        assert_eq!(ck.shared_count("x"), 1);
        let err = ck.loan_mut("x").unwrap_err();
        assert!(err.contains("E0305"), "got: {}", err);
    }

    #[test]
    fn borrow_state_alias_and_diagnostics() {
        let mut st: BorrowState = BorrowState::new();
        st.declare("x");
        st.move_name("x").unwrap();
        let err = st.use_name("x").unwrap_err();
        let d = diagnose_borrow_error(&err).expect("diagnostic");
        assert_eq!(d.code, "E0382");
        assert_eq!(d.rule, BorrowRule::UseAfterMove);
        assert_eq!(d.variable, "x");
        assert!(d.render().contains("E0382"));

        let mut st2 = BorrowState::new();
        st2.declare("y");
        st2.loan_mut("y").unwrap();
        let err2 = st2.loan_mut("y").unwrap_err();
        let d2 = diagnose_borrow_error(&err2).unwrap();
        assert_eq!(d2.code, "E0305");
        assert_eq!(d2.rule, BorrowRule::ExclusiveLoan);
        assert!(borrow_rules_doc().contains("E0305"));
        assert!(diagnose_borrow_error("no code here").is_none());
        assert_eq!(st2.tracked_names(), vec!["y".to_string()]);
    }
}
