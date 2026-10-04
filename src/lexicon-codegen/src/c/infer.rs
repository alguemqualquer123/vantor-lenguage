//! Return-type inference: a fixpoint over the module's functions using an
//! `Unknown`-tolerant type walk (`peek_*`). The strict emission pass repeats
//! the walk with real errors: any `Unknown` that reaches a value context is
//! reported as unsupported and the CLI falls back to the interpreter.

use lexicon_parser::ast::{
    BinOp, Block, Expr, Function, IfStmt, Literal, Pattern, Stmt, UnOp,
};
use std::collections::HashMap;

use super::{unify, Emitter, PeekCtx, Ty};

/// How a statement leaves control: reaches the end of its block (`Fall`),
/// leaves via `return` (`Ret`), or leaves via `break`/`continue` (`Skip`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ender {
    Fall,
    Ret,
    Skip,
}

fn combine(a: Ender, b: Ender) -> Ender {
    match (a, b) {
        (Ender::Fall, _) | (_, Ender::Fall) => Ender::Fall,
        (Ender::Ret, Ender::Ret) => Ender::Ret,
        (Ender::Skip, Ender::Skip) => Ender::Skip,
        // A path that returns and a path that breaks: the statement may do
        // either, so treat it as "may fall through".
        _ => Ender::Fall,
    }
}

impl Emitter {
    pub(crate) fn infer_returns(&mut self) -> Result<(), String> {
        for name in self.fn_order.clone() {
            self.ret_tys.insert(name, Ty::Unknown);
        }
        let mut ann: HashMap<String, Ty> = HashMap::new();
        for name in self.fn_order.clone() {
            let f = self.fns[&name].clone();
            if let Some(t) = &f.return_type {
                let ty = self.ty_of(t).unwrap_or(Ty::Unknown);
                ann.insert(name, ty);
            }
        }
        for _round in 0..8 {
            let mut changed = false;
            for name in self.fn_order.clone() {
                let f = self.fns[&name].clone();
                let body = match &f.body {
                    Some(b) => b.clone(),
                    None => return Err(format!("function `{}` has no body", name)),
                };
                self.scopes.clear();
                self.bound.clear();
                self.shadow_n.clear();
                self.cur_fn = Some(name.clone());
                let mut params = HashMap::new();
                for p in &f.params {
                    let ty = self.ty_of(&p.ty).unwrap_or(Ty::Unknown);
                    params.insert(p.name.text.clone(), (String::new(), ty));
                }
                self.scopes.push(params);
                let mut ctx = PeekCtx { rets: Vec::new() };
                self.scopes.push(HashMap::new());
                let end = self.peek_block(&body, &mut ctx);
                self.scopes.pop();
                let mut t = Ty::Unknown;
                let mut first = true;
                for r in &ctx.rets {
                    if first {
                        t = r.clone();
                        first = false;
                    } else {
                        t = unify(&t, r)?;
                    }
                }
                if first {
                    t = Ty::Unit;
                }
                if end != Ender::Ret {
                    t = unify(&t, &Ty::Unit)
                        .map_err(|_| format!("`{}` may fall through without a value", name))?;
                }
                if let Some(a) = ann.get(&name) {
                    t = unify(&t, a)?;
                }
                if self.ret_tys[&name] != t {
                    self.ret_tys.insert(name.clone(), t);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        for name in self.fn_order.clone() {
            if self.ret_tys[&name] == Ty::Unknown {
                return Err(format!("cannot infer return type of `{}`", name));
            }
        }
        Ok(())
    }

    fn peek_block(&mut self, b: &Block, ctx: &mut PeekCtx) -> Ender {
        let mut end = Ender::Fall;
        for s in &b.statements {
            let e = self.peek_stmt(s, ctx);
            if end == Ender::Fall {
                end = e;
            }
        }
        end
    }

    fn peek_stmt(&mut self, s: &Stmt, ctx: &mut PeekCtx) -> Ender {
        match s {
            Stmt::Expr(e) => {
                self.peek_expr(&e.expr);
                Ender::Fall
            }
            Stmt::Decl(d) => {
                let ty = match &d.init {
                    Some(e) => self.peek_expr(e),
                    None => Ty::Null,
                };
                self.peek_bind(&d.pattern, ty);
                Ender::Fall
            }
            Stmt::If(i) => self.peek_if(i, ctx),
            Stmt::Match(m) => {
                self.peek_expr(&m.expr);
                for arm in &m.arms {
                    self.scopes.push(HashMap::new());
                    if let Some(g) = &arm.guard {
                        self.peek_expr(g);
                    }
                    self.peek_expr(&arm.body);
                    self.scopes.pop();
                }
                Ender::Fall
            }
            Stmt::Switch(sw) => {
                self.peek_expr(&sw.scrutinee);
                let mut all = true;
                for case in &sw.cases {
                    for o in &case.exprs {
                        self.peek_expr(o);
                    }
                    if let Some(g) = &case.guard {
                        self.peek_expr(g);
                    }
                    self.scopes.push(HashMap::new());
                    let e = self.peek_block(&case.body, ctx);
                    self.scopes.pop();
                    if e != Ender::Ret {
                        all = false;
                    }
                }
                match &sw.default {
                    Some(d) => {
                        self.scopes.push(HashMap::new());
                        let e = self.peek_block(d, ctx);
                        self.scopes.pop();
                        if e != Ender::Ret {
                            all = false;
                        }
                    }
                    None => all = false,
                }
                // Every path returning through the cases (`break` included)
                // means the switch itself never reaches the following code.
                if all && !sw.cases.is_empty() {
                    Ender::Ret
                } else {
                    Ender::Fall
                }
            }
            Stmt::For(f) => {
                let it = self.peek_expr(&f.iterable);
                let elem = match &it {
                    Ty::List(t) => (**t).clone(),
                    Ty::Range => Ty::Int,
                    Ty::Str => Ty::Str,
                    Ty::Tuple(ts) => {
                        let mut t = Ty::Unknown;
                        for x in ts {
                            t = unify(&t, x).unwrap_or(Ty::Unknown);
                        }
                        t
                    }
                    _ => Ty::Unknown,
                };
                self.scopes.push(HashMap::new());
                self.peek_bind(&f.pattern, elem);
                self.peek_block(&f.body, ctx);
                self.scopes.pop();
                Ender::Fall
            }
            Stmt::While(w) => {
                self.peek_expr(&w.condition);
                self.scopes.push(HashMap::new());
                self.peek_block(&w.body, ctx);
                self.scopes.pop();
                Ender::Fall
            }
            Stmt::DoWhile(d) => {
                self.peek_expr(&d.condition);
                self.scopes.push(HashMap::new());
                self.peek_block(&d.body, ctx);
                self.scopes.pop();
                Ender::Fall
            }
            Stmt::Loop(l) => {
                self.scopes.push(HashMap::new());
                self.peek_block(&l.body, ctx);
                self.scopes.pop();
                Ender::Fall
            }
            Stmt::Return(r) => {
                let ty = match &r.value {
                    Some(e) => self.peek_expr(e),
                    None => Ty::Unit,
                };
                ctx.rets.push(ty);
                Ender::Ret
            }
            Stmt::Break(_) => Ender::Skip,
            Stmt::Continue(_) => Ender::Skip,
            Stmt::Defer(d) => {
                self.peek_expr(&d.expr);
                Ender::Fall
            }
            Stmt::AsyncBlock(a) => {
                self.peek_expr(&a.body);
                Ender::Fall
            }
            Stmt::Block(b) => {
                self.scopes.push(HashMap::new());
                let e = self.peek_block(b, ctx);
                self.scopes.pop();
                e
            }
        }
    }

    fn peek_if(&mut self, i: &IfStmt, ctx: &mut PeekCtx) -> Ender {
        self.peek_expr(&i.condition);
        self.scopes.push(HashMap::new());
        let then_e = self.peek_block(&i.then_branch, ctx);
        self.scopes.pop();
        let else_e = if let Some(eb) = &i.else_branch {
            self.peek_if(eb, ctx)
        } else if let Some(eb) = &i.else_body {
            self.scopes.push(HashMap::new());
            let e = self.peek_block(eb, ctx);
            self.scopes.pop();
            e
        } else {
            Ender::Fall
        };
        combine(then_e, else_e)
    }

    fn peek_bind(&mut self, p: &Pattern, ty: Ty) {
        match p {
            Pattern::Ident(id) => {
                if id.text != "_" {
                    if let Some(top) = self.scopes.last_mut() {
                        top.insert(id.text.clone(), (String::new(), ty));
                    }
                    self.bound.insert(id.text.clone());
                }
            }
            Pattern::Wildcard(_) => {}
            Pattern::Tuple(parts) => {
                match &ty {
                    Ty::Tuple(ts) if ts.len() == parts.len() => {
                        for (p, t) in parts.iter().zip(ts.iter()) {
                            self.peek_bind(p, t.clone());
                        }
                    }
                    Ty::List(t) => {
                        for p in parts {
                            self.peek_bind(p, (**t).clone());
                        }
                    }
                    _ => {
                        for p in parts {
                            self.peek_bind(p, Ty::Unknown);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn peek_expr(&mut self, e: &Expr) -> Ty {
        match e {
            Expr::Literal(l) => match l {
                Literal::Bool(_) => Ty::Bool,
                Literal::Int(_, _) => Ty::Int,
                Literal::Float(_, _) => Ty::Float,
                Literal::String(_) => Ty::Str,
                Literal::Char(_) => Ty::Char,
                Literal::Unit => Ty::Unit,
                Literal::Null => Ty::Null,
                Literal::Array(es) => {
                    let mut t = Ty::Unknown;
                    for x in es {
                        let xt = self.peek_expr(x);
                        t = unify(&t, &xt).unwrap_or(Ty::Unknown);
                    }
                    Ty::List(Box::new(t))
                }
                Literal::Tuple(es) => Ty::Tuple(es.iter().map(|x| self.peek_expr(x)).collect()),
            },
            Expr::Ident(id) => {
                if let Some((_, t)) = self.lookup(&id.text) {
                    return t;
                }
                if let Some(t) = self.global_tys.get(&id.text) {
                    return t.clone();
                }
                Ty::Unknown
            }
            Expr::InterpolatedString(parts) => {
                for p in parts {
                    if let lexicon_parser::InterpolatedPart::Expr(x) = p {
                        self.peek_expr(x);
                    }
                }
                Ty::Str
            }
            Expr::Unary(u) => {
                let t = self.peek_expr(&u.operand);
                match u.op {
                    UnOp::Neg => t,
                    UnOp::Not => Ty::Bool,
                    UnOp::BitNot => {
                        if t == Ty::Int {
                            Ty::Int
                        } else {
                            Ty::Unknown
                        }
                    }
                    _ => Ty::Unknown,
                }
            }
            Expr::Binary(b) => self.peek_binary(b),
            Expr::Call(c) => {
                self.peek_args(&c.args);
                match &*c.callee {
                    Expr::Ident(id) => match id.text.as_str() {
                        "print" | "println" | "assert" => Ty::Unit,
                        "panic" => Ty::Panic,
                        "inspect" => Ty::Str,
                        "typeOf" | "typeof" | "type" => Ty::Str,
                        other => self.ret_tys.get(other).cloned().unwrap_or(Ty::Unknown),
                    },
                    Expr::FieldAccess(fa) => self.peek_module_call(fa),
                    _ => Ty::Unknown,
                }
            }
            Expr::MethodCall(m) => {
                self.peek_args(&m.args);
                self.peek_method(&m.object, &m.method.text)
            }
            Expr::Index(ix) => {
                let ot = self.peek_expr(&ix.object);
                let it = self.peek_expr(&ix.index);
                match &ot {
                    Ty::List(t) => (**t).clone(),
                    Ty::Str => {
                        if it == Ty::Int || it == Ty::Unknown {
                            Ty::Str
                        } else {
                            Ty::Unknown
                        }
                    }
                    Ty::Tuple(ts) => match &*ix.index {
                        Expr::Literal(Literal::Int(n, _)) => ts
                            .get(*n as usize)
                            .cloned()
                            .unwrap_or(Ty::Unknown),
                        _ => Ty::Unknown,
                    },
                    _ => Ty::Unknown,
                }
            }
            Expr::FieldAccess(fa) => {
                let ot = self.peek_expr(&fa.object);
                match &ot {
                    Ty::Struct(name) => self
                        .structs
                        .get(name)
                        .and_then(|s| s.find(&fa.field.text).map(|i| s.fields[i].1.clone()))
                        .unwrap_or(Ty::Unknown),
                    Ty::Tuple(ts) => fa
                        .field
                        .text
                        .parse::<usize>()
                        .ok()
                        .and_then(|i| ts.get(i).cloned())
                        .unwrap_or(Ty::Unknown),
                    _ => Ty::Unknown,
                }
            }
            Expr::OptionalFieldAccess(fa) => {
                let ot = self.peek_expr(&fa.object);
                if ot == Ty::Null {
                    return Ty::Null;
                }
                match &ot {
                    Ty::Struct(name) => self
                        .structs
                        .get(name)
                        .and_then(|s| s.find(&fa.field.text).map(|i| s.fields[i].1.clone()))
                        .unwrap_or(Ty::Unknown),
                    _ => Ty::Unknown,
                }
            }
            Expr::Cast(c) => {
                let src = self.peek_expr(&c.expr);
                match self.ty_of(&c.ty) {
                    Ok(t) => t,
                    Err(_) => src,
                }
            }
            Expr::If(i) => {
                self.peek_expr(&i.condition);
                let a = self.peek_expr(&i.then_expr);
                match &i.else_expr {
                    Some(b) => {
                        let bt = self.peek_expr(b);
                        unify(&a, &bt).unwrap_or(Ty::Unknown)
                    }
                    None => unify(&a, &Ty::Unit).unwrap_or(Ty::Unknown),
                }
            }
            Expr::Match(m) => {
                self.peek_expr(&m.expr);
                let mut t = Ty::Unknown;
                for arm in &m.arms {
                    if let Some(g) = &arm.guard {
                        self.peek_expr(g);
                    }
                    let bt = self.peek_expr(&arm.body);
                    t = unify(&t, &bt).unwrap_or(Ty::Unknown);
                }
                t
            }
            Expr::NullCoalesce(a, b) => {
                let lt = self.peek_expr(a);
                let rt = self.peek_expr(b);
                if lt == Ty::Null {
                    rt
                } else if rt == Ty::Null || lt == rt {
                    lt
                } else {
                    Ty::Unknown
                }
            }
            Expr::Range(a, b, _) => {
                self.peek_expr(a);
                self.peek_expr(b);
                Ty::Range
            }
            Expr::Await(a) => self.peek_expr(&a.expr),
            Expr::New(n) => {
                let name = n.ty.clone();
                self.peek_args(&n.args);
                match &name {
                    lexicon_parser::Type::Path(p) if p.segments.len() == 1 => {
                        let seg = &p.segments[0].text;
                        if self.struct_decls.contains_key(seg) {
                            Ty::Struct(seg.clone())
                        } else {
                            Ty::Unknown
                        }
                    }
                    _ => Ty::Unknown,
                }
            }
            Expr::UnsafeBlock(b, _) => {
                let mut ctx = PeekCtx { rets: Vec::new() };
                self.scopes.push(HashMap::new());
                self.peek_block(b, &mut ctx);
                self.scopes.pop();
                Ty::Unit
            }
            _ => Ty::Unknown,
        }
    }

    fn peek_args(&mut self, args: &[Expr]) {
        for a in args {
            self.peek_expr(a);
        }
    }

    fn peek_module_call(&mut self, fa: &lexicon_parser::FieldAccessExpr) -> Ty {
        if let Expr::Ident(obj) = &*fa.object {
            if obj.text.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                && self.lookup(&obj.text).is_none()
                && !self.global_tys.contains_key(&obj.text)
            {
                if obj.text == "Console" {
                    return Ty::Unit;
                }
            }
        }
        Ty::Unknown
    }

    fn peek_method(&mut self, obj: &Expr, method: &str) -> Ty {
        match method {
            "len" | "length" | "count" | "size" => Ty::Int,
            "isEmpty" | "is_empty" => Ty::Bool,
            "push" => Ty::Unit,
            "pop" => Ty::Unknown,
            "contains" => Ty::Bool,
            "to_string" | "toString" => Ty::Str,
            "char_at" | "charAt" => Ty::Char,
            _ => {
                if let Expr::Ident(o) = obj {
                    if o.text.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                        && self.lookup(&o.text).is_none()
                        && !self.global_tys.contains_key(&o.text)
                    {
                        if o.text == "Console" {
                            return Ty::Unit;
                        }
                    }
                }
                Ty::Unknown
            }
        }
    }

    fn peek_binary(&mut self, b: &lexicon_parser::BinaryExpr) -> Ty {
        let lt = self.peek_expr(&b.lhs);
        let rt = self.peek_expr(&b.rhs);
        match b.op {
            BinOp::Add => {
                if lt == Ty::Str && rt.is_num() {
                    Ty::Str
                } else if lt.is_num() && rt == Ty::Str {
                    Ty::Str
                } else if lt == Ty::Str && rt == Ty::Str {
                    Ty::Str
                } else if lt.is_num() && rt.is_num() {
                    if lt == Ty::Float || rt == Ty::Float {
                        Ty::Float
                    } else {
                        Ty::Int
                    }
                } else {
                    Ty::Unknown
                }
            }
            BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                if lt.is_num() && rt.is_num() {
                    if lt == Ty::Float || rt == Ty::Float {
                        Ty::Float
                    } else {
                        Ty::Int
                    }
                } else {
                    Ty::Unknown
                }
            }
            BinOp::EqEq | BinOp::Neq | BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => {
                Ty::Bool
            }
            BinOp::AndAnd | BinOp::AmpAmp | BinOp::OrOr | BinOp::PipePipe => Ty::Bool,
            BinOp::Shl | BinOp::Shr | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
                if lt == Ty::Int && rt == Ty::Int {
                    Ty::Int
                } else {
                    Ty::Unknown
                }
            }
            BinOp::AddEq => rt,
            BinOp::SubEq | BinOp::MulEq | BinOp::DivEq | BinOp::ModEq => {
                if lt.is_num() && rt.is_num() {
                    if lt == Ty::Float || rt == Ty::Float {
                        Ty::Float
                    } else {
                        Ty::Int
                    }
                } else {
                    Ty::Unknown
                }
            }
            BinOp::Pipe => match &*b.rhs {
                Expr::Ident(id) => self.ret_tys.get(&id.text).cloned().unwrap_or(Ty::Unknown),
                Expr::Call(c) => {
                    if let Expr::Ident(id) = &*c.callee {
                        self.ret_tys.get(&id.text).cloned().unwrap_or(Ty::Unknown)
                    } else {
                        Ty::Unknown
                    }
                }
                _ => Ty::Unknown,
            },
        }
    }
}

/// Unused import guard: `Function` is referenced through `self.fns`.
#[allow(dead_code)]
fn _assert_used(_: &Function) {}
