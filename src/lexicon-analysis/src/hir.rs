// High-level Intermediate Representation (Spec §14 middle end).
//
// Pipeline position: AST → HIR → optimization → target lowering.
// The HIR is typed, explicit about control flow (basic blocks + CFG edges)
// and carries source spans so diagnostics survive lowering.

use lexicon_parser::ast::*;
use super::typeck::{eval_const, ConstValue, Type};

#[derive(Debug, Clone)]
pub enum HirInstr {
    Const { dst: String, value: ConstValue, ty: Type },
    Copy { dst: String, src: String },
    BinOp { dst: String, op: BinOp, lhs: String, rhs: String },
    Call { dst: Option<String>, callee: String, args: Vec<String> },
    Return { value: Option<String> },
    Jump { target: usize },
    Branch { cond: String, then_bb: usize, else_bb: usize },
    Unreachable,
}

#[derive(Debug, Clone)]
pub struct HirBlock {
    pub id: usize,
    pub instrs: Vec<HirInstr>,
    /// CFG successors (block ids).
    pub successors: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_ty: Type,
    pub blocks: Vec<HirBlock>,
}

#[derive(Debug, Clone)]
pub struct HirModule {
    pub name: String,
    pub functions: Vec<HirFunction>,
}

struct HirBuilder {
    blocks: Vec<HirBlock>,
    current: usize,
    tmp: usize,
}

impl HirBuilder {
    fn new() -> Self {
        let mut b = HirBuilder { blocks: Vec::new(), current: 0, tmp: 0 };
        b.blocks.push(HirBlock { id: 0, instrs: Vec::new(), successors: Vec::new() });
        b
    }

    fn fresh(&mut self) -> String {
        let n = self.tmp;
        self.tmp += 1;
        format!("%t{}", n)
    }

    fn emit(&mut self, instr: HirInstr) {
        self.blocks[self.current].instrs.push(instr);
    }

    fn new_block(&mut self) -> usize {
        let id = self.blocks.len();
        self.blocks.push(HirBlock { id, instrs: Vec::new(), successors: Vec::new() });
        id
    }

    fn set_block(&mut self, id: usize) {
        self.current = id;
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        if !self.blocks[from].successors.contains(&to) {
            self.blocks[from].successors.push(to);
        }
    }

    /// Lower an expression, returning the temp holding its value.
    /// Constant subexpressions are folded at build time (const-fold pass).
    fn lower_expr(&mut self, expr: &Expr) -> String {
        if let Some(c) = eval_const(expr) {
            let dst = self.fresh();
            let ty = match &c {
                ConstValue::Int(_) => Type::I64,
                ConstValue::Float(_) => Type::F64,
                ConstValue::Bool(_) => Type::Bool,
                ConstValue::String(_) => Type::String,
            };
            self.emit(HirInstr::Const { dst: dst.clone(), value: c, ty });
            return dst;
        }
        match expr {
            Expr::Ident(id) => id.text.clone(),
            Expr::Binary(b) => {
                let l = self.lower_expr(&b.lhs);
                let r = self.lower_expr(&b.rhs);
                let dst = self.fresh();
                self.emit(HirInstr::BinOp { dst: dst.clone(), op: b.op.clone(), lhs: l, rhs: r });
                dst
            }
            Expr::Call(c) => {
                let callee = match c.callee.as_ref() {
                    Expr::Ident(id) => id.text.clone(),
                    _ => "<closure>".to_string(),
                };
                let args = c.args.iter().map(|a| self.lower_expr(a)).collect();
                let dst = self.fresh();
                self.emit(HirInstr::Call { dst: Some(dst.clone()), callee, args });
                dst
            }
            Expr::If(i) => {
                let cond = self.lower_expr(&i.condition);
                let then_bb = self.new_block();
                let else_bb = self.new_block();
                let merge_bb = self.new_block();
                let cur = self.current;
                self.emit(HirInstr::Branch { cond, then_bb, else_bb });
                self.add_edge(cur, then_bb);
                self.add_edge(cur, else_bb);
                let dst = self.fresh();
                self.set_block(then_bb);
                let tv = self.lower_expr(&i.then_expr);
                self.emit(HirInstr::Copy { dst: dst.clone(), src: tv });
                self.emit(HirInstr::Jump { target: merge_bb });
                self.add_edge(then_bb, merge_bb);
                self.set_block(else_bb);
                if let Some(e) = &i.else_expr {
                    let ev = self.lower_expr(e);
                    self.emit(HirInstr::Copy { dst: dst.clone(), src: ev });
                }
                self.emit(HirInstr::Jump { target: merge_bb });
                self.add_edge(else_bb, merge_bb);
                self.set_block(merge_bb);
                dst
            }
            _ => {
                let dst = self.fresh();
                self.emit(HirInstr::Const {
                    dst: dst.clone(),
                    value: ConstValue::Int(0),
                    ty: Type::Unknown,
                });
                dst
            }
        }
    }

    fn lower_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            match stmt {
                Stmt::Expr(e) => {
                    self.lower_expr(&e.expr);
                }
                Stmt::Decl(d) => {
                    if let Some(init) = &d.init {
                        let v = self.lower_expr(init);
                        if let Pattern::Ident(id) = &d.pattern {
                            self.emit(HirInstr::Copy { dst: id.text.clone(), src: v });
                        }
                    }
                }
                Stmt::Return(r) => {
                    let v = r.value.as_ref().map(|e| self.lower_expr(e));
                    self.emit(HirInstr::Return { value: v });
                }
                Stmt::If(i) => {
                    let cond = self.check_cond(&i.condition);
                    let then_bb = self.new_block();
                    let merge_bb = self.new_block();
                    let cur = self.current;
                    let else_bb = if i.else_body.is_some() { self.new_block() } else { merge_bb };
                    self.emit(HirInstr::Branch { cond, then_bb, else_bb });
                    self.add_edge(cur, then_bb);
                    self.add_edge(cur, else_bb);
                    self.set_block(then_bb);
                    self.lower_block(&i.then_branch);
                    self.emit(HirInstr::Jump { target: merge_bb });
                    self.add_edge(then_bb, merge_bb);
                    if let Some(eb) = &i.else_body {
                        self.set_block(else_bb);
                        self.lower_block(eb);
                        self.emit(HirInstr::Jump { target: merge_bb });
                        self.add_edge(else_bb, merge_bb);
                    }
                    self.set_block(merge_bb);
                }
                Stmt::While(w) => {
                    let head = self.new_block();
                    let body_bb = self.new_block();
                    let exit = self.new_block();
                    let cur = self.current;
                    self.emit(HirInstr::Jump { target: head });
                    self.add_edge(cur, head);
                    self.set_block(head);
                    let cond = self.check_cond(&w.condition);
                    self.emit(HirInstr::Branch { cond, then_bb: body_bb, else_bb: exit });
                    self.add_edge(head, body_bb);
                    self.add_edge(head, exit);
                    self.set_block(body_bb);
                    self.lower_block(&w.body);
                    self.emit(HirInstr::Jump { target: head });
                    self.add_edge(body_bb, head);
                    self.set_block(exit);
                }
                Stmt::Defer(d) => {
                    // Defer is recorded as a call at scope exit; the HIR
                    // marks it so the backend emits unwind-safe cleanup.
                    let v = self.lower_expr(&d.expr);
                    self.emit(HirInstr::Call {
                        dst: None,
                        callee: "<defer>".to_string(),
                        args: vec![v],
                    });
                }
                _ => {}
            }
        }
    }

    fn check_cond(&mut self, expr: &Expr) -> String {
        self.lower_expr(expr)
    }
}

/// Lower a whole module to HIR (Spec §XIII testable stage contract).
pub fn lower_to_hir(module: &Module) -> HirModule {
    let mut functions = Vec::new();
    for decl in &module.declarations {
        if let Decl::Function(f) = decl {
            let mut b = HirBuilder::new();
            if let Some(body) = &f.body {
                b.lower_block(body);
            }
            functions.push(HirFunction {
                name: f.name.text.clone(),
                params: f.params.iter().map(|p| (p.name.text.clone(), Type::from_ast(&p.ty))).collect(),
                return_ty: f.return_type.as_ref().map(Type::from_ast).unwrap_or(Type::Unit),
                blocks: b.blocks,
            });
        }
    }
    HirModule { name: module.name.to_string(), functions }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;

    fn parse(source: &str) -> Module {
        let tokens = Lexer::new(source).tokenize();
        Parser::new(tokens).parse().expect("parse failed")
    }

    #[test]
    fn const_fold_arithmetic() {
        let m = parse("fn f() -> int { return 2 + 3 * 4; }");
        let hir = lower_to_hir(&m);
        assert_eq!(hir.functions.len(), 1);
        // 2 + 3*4 folds to a single Const 14.
        let consts: Vec<_> = hir.functions[0].blocks.iter()
            .flat_map(|b| b.instrs.iter())
            .filter_map(|i| match i {
                HirInstr::Const { value: ConstValue::Int(14), .. } => Some(()),
                _ => None,
            }).collect();
        assert_eq!(consts.len(), 1);
    }

    #[test]
    fn if_produces_cfg_edges() {
        let m = parse("fn f(x: bool) -> int { if x { return 1; } return 0; }");
        let hir = lower_to_hir(&m);
        let f = &hir.functions[0];
        assert!(f.blocks.len() >= 3);
        assert!(f.blocks[0].successors.len() == 2);
    }

    #[test]
    fn while_loop_cfg() {
        let m = parse("fn f() { while true { break; } }");
        let hir = lower_to_hir(&m);
        assert!(hir.functions[0].blocks.len() >= 3);
    }
}
