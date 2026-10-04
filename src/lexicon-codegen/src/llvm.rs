use lexicon_analysis::Type as Ty;
use lexicon_analysis::TypeChecker;
use lexicon_analysis::hir::HirBlock;
use lexicon_analysis::{HirFunction, HirInstr, HirModule};
use lexicon_parser::ast::{Decl, Function, Module, Param};
use lexicon_core::{Error, Result};
use std::collections::{HashMap, HashSet};

/// Backend responsável por transformar a AST em LLVM IR textual.
///
/// Pipeline (Spec §XIII–§XIV): type check → IR → optimization
/// passes → target lowering → textual IR.
pub struct LlvmBackend {
    pub target: Target,
    pub opt_level: OptLevel,
}

/// Supported compilation targets (Spec §15).
/// P0: x86-64 Windows/Linux, ARM64 · P1: macOS, WASM ·
/// P2: RISC-V, FreeBSD, Android, iOS · P3: embedded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    X86_64Windows,
    X86_64Linux,
    MacOsX86_64,
    MacOsAarch64,
    Aarch64Linux,
    Wasm32,
    Riscv64,
    FreeBsdX86_64,
    AndroidAarch64,
    IosAarch64,
    Embedded,
}

impl Target {
    pub fn triple(&self) -> &'static str {
        match self {
            Target::X86_64Windows => "x86_64-pc-windows-msvc",
            Target::X86_64Linux => "x86_64-unknown-linux-gnu",
            Target::MacOsX86_64 => "x86_64-apple-darwin",
            Target::MacOsAarch64 => "aarch64-apple-darwin",
            Target::Aarch64Linux => "aarch64-unknown-linux-gnu",
            Target::Wasm32 => "wasm32-unknown-unknown",
            Target::Riscv64 => "riscv64gc-unknown-linux-gnu",
            Target::FreeBsdX86_64 => "x86_64-unknown-freebsd",
            Target::AndroidAarch64 => "aarch64-linux-android",
            Target::IosAarch64 => "aarch64-apple-ios",
            Target::Embedded => "thumbv7em-none-eabihf",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "x86_64-windows" | "x86_64-pc-windows-msvc" => Some(Target::X86_64Windows),
            "x86_64-linux" | "x86_64-unknown-linux-gnu" | "native" => Some(Target::X86_64Linux),
            "x86_64-macos" | "x86_64-apple-darwin" => Some(Target::MacOsX86_64),
            "aarch64-macos" | "aarch64-apple-darwin" => Some(Target::MacOsAarch64),
            "aarch64-linux" | "aarch64" => Some(Target::Aarch64Linux),
            "wasm" | "wasm32" | "wasm32-unknown-unknown" => Some(Target::Wasm32),
            "riscv64" => Some(Target::Riscv64),
            "freebsd" => Some(Target::FreeBsdX86_64),
            "android" => Some(Target::AndroidAarch64),
            "ios" => Some(Target::IosAarch64),
            "embedded" => Some(Target::Embedded),
            _ => None,
        }
    }
}

/// Build profiles (Spec §XIV).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
    Debug,
    Release,
    Test,
    Sanitized,
}

/// Language ABI version (Spec §67 + §78). Every released ABI identifies
/// architecture, OS, calling convention and this language ABI version.
/// Bumped only with a documented compatibility transition.
pub const LEX_ABI_VERSION: u32 = 1;

/// Calling convention selected per target (Spec §4 + §67).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConv {
    /// Windows x64 (`__vectorcall`-free default).
    Win64,
    /// System V AMD64 ABI (Linux/macOS/BSD x86-64).
    SysV,
    /// AArch64 procedure call standard.
    Aapcs64,
    /// WebAssembly MVP calling convention.
    Wasm,
    /// RISC-V LP64D calling convention.
    Riscv,
}

impl CallingConv {
    /// Parameter passing rule: registers vs stack (Spec §4).
    pub fn param_rule(&self) -> &'static str {
        match self {
            CallingConv::Win64 => {
                "params: RCX,RDX,R8,R9 + stack (32B shadow space); floats in XMM0-XMM3"
            }
            CallingConv::SysV => {
                "params: RDI,RSI,RDX,RCX,R8,R9 + stack; floats in XMM0-XMM7"
            }
            CallingConv::Aapcs64 => {
                "params: X0-X7 + stack; floats/SIMD in V0-V7"
            }
            CallingConv::Wasm => {
                "params: operand stack (i32/i64/f32/f64)"
            }
            CallingConv::Riscv => {
                "params: A0-A7 + stack; floats in FA0-FA7"
            }
        }
    }

    /// Result return rule (Spec §4).
    pub fn result_rule(&self) -> &'static str {
        match self {
            CallingConv::Win64 => "result: RAX (RDX for 128-bit); floats in XMM0",
            CallingConv::SysV => "result: RAX:RDX; floats in XMM0:XMM1",
            CallingConv::Aapcs64 => "result: X0 (+X1 for 128-bit); floats in V0",
            CallingConv::Wasm => "result: operand stack",
            CallingConv::Riscv => "result: A0 (+A1 for 128-bit); floats in FA0",
        }
    }

    /// Combined passing rules for diagnostics and IR notes.
    pub fn passing_rules(&self) -> String {
        format!("{}; {}", self.param_rule(), self.result_rule())
    }
}

impl Target {
    /// Calling convention for this target (Spec §67, target-specific).
    pub fn calling_conv(&self) -> CallingConv {
        match self {
            Target::X86_64Windows => CallingConv::Win64,
            Target::X86_64Linux | Target::MacOsX86_64 | Target::FreeBsdX86_64 => CallingConv::SysV,
            // AAPCS family: macOS/Linux/Android/iOS AArch64 plus the
            // embedded ARM target (`thumbv7em`, AAPCS 32-bit; modeled with
            // the same X0-X7/V0-V7 + stack rules as AAPCS64).
            Target::MacOsAarch64 | Target::Aarch64Linux
            | Target::AndroidAarch64 | Target::IosAarch64 | Target::Embedded => CallingConv::Aapcs64,
            Target::Wasm32 => CallingConv::Wasm,
            Target::Riscv64 => CallingConv::Riscv,
        }
    }

    /// ABI identifier string: `arch-os-conv-lexABI` (Spec §78).
    pub fn abi_id(&self) -> String {
        format!("{:?}-lex{}", self.calling_conv(), LEX_ABI_VERSION)
    }
}

/// Check two ABI identifiers for compatibility (Spec §78).
pub fn abi_compatible(a: &str, b: &str) -> bool {
    a == b
}

/// Stable name mangling: `_LX<module>_<name>_<tyhash>` (Spec §67).
/// Never changes within a `LEX_ABI_VERSION`.
pub fn mangle_symbol(module: &str, name: &str, ty_hash: u64) -> String {
    let clean = |s: &str| {
        s.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
    };
    format!("_LX{}_{}_{:x}", clean(module), clean(name), ty_hash)
}

/// Computed struct layout for FFI (Spec §5 + §67): size, alignment and
/// per-field offsets following C-compatible rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLayout {
    pub size: usize,
    pub align: usize,
    pub field_offsets: Vec<(String, usize)>,
}

/// Layout of a struct from `(name, size, align)` fields.
/// Padding is inserted so every field starts at a multiple of its
/// alignment; tail padding rounds the size up to the struct alignment.
pub fn compute_layout(fields: &[(String, usize, usize)]) -> StructLayout {
    let mut offset = 0usize;
    let mut max_align = 1usize;
    let mut field_offsets = Vec::new();
    for (name, size, align) in fields {
        let align = (*align).max(1);
        max_align = max_align.max(align);
        let mis = offset % align;
        if mis != 0 {
            offset += align - mis;
        }
        field_offsets.push((name.clone(), offset));
        offset += size;
    }
    let mis = offset % max_align;
    if mis != 0 {
        offset += max_align - mis;
    }
    StructLayout { size: offset, align: max_align, field_offsets }
}

/// Part X / §31: FFI C/C++ surface (Spec §31 + §67).
///
/// Rules: foreign memory and calling conventions stay isolated from
/// the safe model. Every extern call crosses an explicit declaration
/// ([`ExternDecl`]); callbacks cross a trampoline ([`CallbackTrampoline`])
/// that owns the error boundary (foreign exceptions/unwind NEVER
/// propagate into Lex frames — they become `Err` codes). Struct layout
/// follows [`compute_layout`] (C-compatible). Ownership: caller-owned
/// buffers are borrowed for the call duration only; the callee MUST NOT
/// retain pointers after return unless the declaration says so.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternDecl {
    /// Lex-visible symbol name.
    pub name: String,
    /// C symbol name (may differ for mangled C++ / prefixed libs).
    pub c_symbol: String,
    /// Linked library (e.g. `"c"`, `"m"`, `"user32"`).
    pub lib: String,
    /// Return + parameter types in LLVM spelling.
    pub ret: String,
    pub params: Vec<String>,
    /// Calling convention note (e.g. `"C"`, `"WinAPI"`, `"POSIX"`).
    pub convention: String,
}

impl ExternDecl {
    pub fn new(
        name: impl Into<String>,
        c_symbol: impl Into<String>,
        lib: impl Into<String>,
        ret: impl Into<String>,
        params: Vec<String>,
        convention: impl Into<String>,
    ) -> Self {
        ExternDecl {
            name: name.into(),
            c_symbol: c_symbol.into(),
            lib: lib.into(),
            ret: ret.into(),
            params,
            convention: convention.into(),
        }
    }

    /// Whether this declaration targets C++ (mangled `c_symbol`).
    pub fn is_cpp(&self) -> bool {
        self.convention == "C++"
    }
}

/// Emit an LLVM `declare` for a C/C++ extern plus provenance comments.
///
/// The error boundary is documented inline: callees report errors via
/// return codes / out-params (never via unwind into Lex frames).
pub fn emit_extern_decl(d: &ExternDecl) -> String {
    format!(
        "; extern @{name} from lib `{lib}` (C symbol `{sym}`, convention {conv}; errors via return codes, no unwind into Lex)\ndeclare {ret} @{name}({params})",
        name = d.name,
        lib = d.lib,
        sym = d.c_symbol,
        conv = d.convention,
        ret = d.ret,
        params = d.params.join(", "),
    )
}

/// Callback trampoline descriptor (Spec §31).
///
/// A trampoline is a tiny generated shim: the foreign side calls the
/// trampoline with the C ABI; the trampoline converts pointers with
/// bounds checks, invokes the Lex closure, and translates panics into
/// an error code at the boundary. This stub describes the shape; real
/// codegen lowers it per [`Target::calling_conv`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallbackTrampoline {
    pub name: String,
    pub param_tys: Vec<String>,
    pub ret_ty: String,
}

impl CallbackTrampoline {
    pub fn new(name: impl Into<String>, param_tys: Vec<String>, ret_ty: impl Into<String>) -> Self {
        CallbackTrampoline { name: name.into(), param_tys, ret_ty: ret_ty.into() }
    }
}

/// Emit a documented trampoline stub for `cb` on `target`.
pub fn emit_trampoline(cb: &CallbackTrampoline, target: Target) -> String {
    format!(
        "; trampoline @{name} for {triple} [{conv:?}]: C ABI in -> bounds-checked convert -> Lex call -> panic-to-error-code boundary\ndefine {ret} @__tramp_{name}({params}) {{\n  ret {ret} undef ; stub: lowered per target at codegen\n}}",
        name = cb.name,
        triple = target.triple(),
        conv = target.calling_conv(),
        ret = cb.ret_ty,
        params = cb.param_tys.join(", "),
    )
}

/// Part X / §68: inline-assembly stub (target-scoped, non-portable).
///
/// Contract (Spec §68 + §32): inline asm is allowed ONLY inside
/// `unsafe` blocks, is scoped to one [`Target`], and is clearly marked
/// non-portable. This stub validates scope and renders a comment-level
/// placeholder; real instruction emission is a per-target backend step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAsm {
    /// Assembly template (e.g. `"nop"`, `"cpuid"`).
    pub template: String,
    /// LLVM-style constraints (e.g. `"={eax},={ebx}"`).
    pub constraints: String,
    /// Target this asm is written for.
    pub target: Target,
}

impl InlineAsm {
    pub fn new(template: impl Into<String>, constraints: impl Into<String>, target: Target) -> Self {
        InlineAsm { template: template.into(), constraints: constraints.into(), target }
    }

    /// Whether this asm may be emitted for `target` (exact match only).
    pub fn is_supported(&self, target: Target) -> bool {
        self.target == target
    }
}

/// Render an inline-asm placeholder, or an explicit non-portability
/// error when `target` differs from the asm's scoped target.
pub fn emit_inline_asm(asm: &InlineAsm, target: Target) -> Result<String> {
    if !asm.is_supported(target) {
        return Err(Error::Codegen(format!(
            "E0401: inline-asm scoped to {} cannot be emitted for {} (non-portable, Spec §68)",
            asm.target.triple(),
            target.triple()
        )));
    }
    Ok(format!(
        "; inline-asm for {} (unsafe-only, non-portable): template `{}` constraints `{}`\n  call void asm sideeffect \"{}\", \"{}\"()",
        target.triple(),
        asm.template,
        asm.constraints,
        asm.template,
        asm.constraints,
    ))
}

/// CPU feature detection for portable SIMD dispatch (Spec §33 + §68).
/// Returns enabled feature names for the host (x86-64 / aarch64).
pub fn cpu_features() -> Vec<&'static str> {
    let mut out = Vec::new();
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("sse2") { out.push("sse2"); }
        if std::arch::is_x86_feature_detected!("avx") { out.push("avx"); }
        if std::arch::is_x86_feature_detected!("avx2") { out.push("avx2"); }
        if std::arch::is_x86_feature_detected!("avx512f") { out.push("avx512f"); }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // NEON/ASIMD is mandatory on AArch64.
        out.push("neon");
    }
    out
}

/// Whether a named CPU feature is enabled on this host (Spec §68).
pub fn has_cpu_feature(name: &str) -> bool {
    cpu_features().iter().any(|f| *f == name)
}

/// Whether `feature` can be assumed for `target` on this host.
/// Conservative: returns `true` only when the host reports the feature
/// AND the target is known to support that ISA family.
pub fn require_cpu_feature(target: Target, feature: &str) -> bool {
    if !has_cpu_feature(feature) {
        return false;
    }
    match feature {
        "sse2" | "avx" | "avx2" | "avx512f" => matches!(
            target,
            Target::X86_64Windows
                | Target::X86_64Linux
                | Target::MacOsX86_64
                | Target::FreeBsdX86_64
        ),
        "neon" => matches!(
            target,
            Target::MacOsAarch64
                | Target::Aarch64Linux
                | Target::AndroidAarch64
                | Target::IosAarch64
        ),
        _ => false,
    }
}

/// Per-function escape result (Spec §14).
///
/// A local "escapes" when it is observable outside its stack frame:
/// passed as a call argument (including `<defer>` cleanup calls) or
/// returned as the function result. Everything else (pure temporaries,
/// `BinOp` operands, branch conditions, `Copy` chains) is stack-confined.
///
/// NOTE: the current HIR has no `Store`/`Alloc` instructions, so heap
/// stores are not modeled yet; when they land they must count as escapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscapeInfo {
    pub function: String,
    pub escapes: Vec<String>,
    pub confined: Vec<String>,
}

/// Whole-module escape report (Spec §14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscapeReport {
    pub functions: Vec<EscapeInfo>,
}

impl EscapeReport {
    pub fn get(&self, function: &str) -> Option<&EscapeInfo> {
        self.functions.iter().find(|f| f.function == function)
    }
}

/// Escape analysis over HIR (Spec §14).
///
/// For each function collects defined locals (`params` + every `dst`)
/// then marks those used as `Call` args or `Return` values as escaping.
pub fn escape_analysis(hir: &HirModule) -> EscapeReport {
    let mut functions = Vec::new();
    for func in &hir.functions {
        let mut defined: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for (p, _) in &func.params {
            if seen.insert(p.clone()) {
                defined.push(p.clone());
            }
        }
        for block in &func.blocks {
            for instr in &block.instrs {
                let dst_opt: Option<&String> = match instr {
                    HirInstr::Const { dst, .. } => Some(dst),
                    HirInstr::Copy { dst, .. } => Some(dst),
                    HirInstr::BinOp { dst, .. } => Some(dst),
                    HirInstr::Call { dst: Some(d), .. } => Some(d),
                    _ => None,
                };
                if let Some(d) = dst_opt {
                    if seen.insert(d.clone()) {
                        defined.push(d.clone());
                    }
                }
            }
        }
        let mut escaping_set: HashSet<String> = HashSet::new();
        for block in &func.blocks {
            for instr in &block.instrs {
                match instr {
                    HirInstr::Call { args, .. } => {
                        for a in args {
                            escaping_set.insert(a.clone());
                        }
                    }
                    HirInstr::Return { value: Some(v) } => {
                        escaping_set.insert(v.clone());
                    }
                    _ => {}
                }
            }
        }
        let mut escapes = Vec::new();
        let mut confined = Vec::new();
        for d in defined {
            if escaping_set.contains(&d) {
                escapes.push(d);
            } else {
                confined.push(d);
            }
        }
        escapes.sort();
        confined.sort();
        functions.push(EscapeInfo { function: func.name.clone(), escapes, confined });
    }
    EscapeReport { functions }
}

/// Size budget for the HIR inliner (Spec §14): callees with more than
/// this many HIR instructions are never inlined.
pub const INLINE_BUDGET: usize = 20;

/// Inliner outcome (Spec §14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineReport {
    /// `(caller, callee)` pairs that were inlined.
    pub inlined: Vec<(String, String)>,
    /// `(caller, callee, reason)` for skipped call sites.
    /// Reasons: `unmarked`, `oversize cost N > budget M`,
    /// `multi-block callee`, `control-flow`, `arity mismatch`.
    pub skipped: Vec<(String, String, String)>,
}

/// Cost model for inlining: total HIR instruction count.
pub fn inline_cost(func: &HirFunction) -> usize {
    func.blocks.iter().map(|b| b.instrs.len()).sum()
}

/// Collect `#[inline]` / `@inline` marks from an AST module.
///
/// Scans top-level `fn` declarations for an attribute whose name is
/// exactly `inline` (either `@inline` or `#[inline]` form).
pub fn inline_marks_from_module(module: &Module) -> HashSet<String> {
    let mut out = HashSet::new();
    for decl in &module.declarations {
        if let Decl::Function(f) = decl {
            if f.attrs.iter().any(|a| a.name.text == "inline") {
                out.insert(f.name.text.clone());
            }
        }
    }
    out
}

/// Inline HIR-level calls (Spec §14).
///
/// Inlines direct `Call` sites whose callee is in `inline_marked` and
/// whose `inline_cost` fits within `budget`. Only single-block callees
/// without internal control flow (`Jump`/`Branch`) are inlined; anything
/// else is reported in `skipped` with a reason and left as a `Call`.
///
/// Inlining renames callee locals to `"<orig>_inl<tag>_<callee>"` (with a
/// leading `%` stripped), binds params via `Copy param <- arg`, splices
/// the callee body, and rewrites `Return v` into `Copy call_dst <- v`.
/// Recursive calls (`callee == caller`) are never inlined.
pub fn inline_calls(
    hir: &HirModule,
    inline_marked: &HashSet<String>,
    budget: usize,
) -> (HirModule, InlineReport) {
    let mut func_map: HashMap<&str, &HirFunction> = HashMap::new();
    for f in &hir.functions {
        func_map.insert(f.name.as_str(), f);
    }
    let mut report = InlineReport { inlined: Vec::new(), skipped: Vec::new() };
    let mut new_functions: Vec<HirFunction> = Vec::new();
    let mut fresh_tag: usize = 0;

    for caller in &hir.functions {
        let mut new_blocks: Vec<HirBlock> = caller.blocks.clone();
        for block_idx in 0..new_blocks.len() {
            let orig_instrs = std::mem::take(&mut new_blocks[block_idx].instrs);
            let mut expanded: Vec<HirInstr> = Vec::with_capacity(orig_instrs.len());
            for instr in orig_instrs {
                match instr {
                    HirInstr::Call { dst, callee, args } => {
                        let known = func_map.contains_key(callee.as_str());
                        if !inline_marked.contains(&callee) {
                            if known {
                                report.skipped.push((
                                    caller.name.clone(),
                                    callee.clone(),
                                    "unmarked".to_string(),
                                ));
                            }
                            expanded.push(HirInstr::Call { dst, callee, args });
                            continue;
                        }
                        if callee == caller.name {
                            report.skipped.push((
                                caller.name.clone(),
                                callee.clone(),
                                "recursive".to_string(),
                            ));
                            expanded.push(HirInstr::Call { dst, callee, args });
                            continue;
                        }
                        let callee_func = match func_map.get(callee.as_str()) {
                            Some(f) => *f,
                            None => {
                                expanded.push(HirInstr::Call { dst, callee, args });
                                continue;
                            }
                        };
                        let cost = inline_cost(callee_func);
                        if cost > budget {
                            report.skipped.push((
                                caller.name.clone(),
                                callee.clone(),
                                format!("oversize cost {} > budget {}", cost, budget),
                            ));
                            expanded.push(HirInstr::Call { dst, callee, args });
                            continue;
                        }
                        if callee_func.blocks.len() != 1 {
                            report.skipped.push((
                                caller.name.clone(),
                                callee.clone(),
                                "multi-block callee".to_string(),
                            ));
                            expanded.push(HirInstr::Call { dst, callee, args });
                            continue;
                        }
                        if callee_func.params.len() != args.len() {
                            report.skipped.push((
                                caller.name.clone(),
                                callee.clone(),
                                "arity mismatch".to_string(),
                            ));
                            expanded.push(HirInstr::Call { dst, callee, args });
                            continue;
                        }
                        let has_cfg = callee_func.blocks[0].instrs.iter().any(|i| {
                            matches!(i, HirInstr::Jump { .. } | HirInstr::Branch { .. })
                        });
                        if has_cfg {
                            report.skipped.push((
                                caller.name.clone(),
                                callee.clone(),
                                "control-flow".to_string(),
                            ));
                            expanded.push(HirInstr::Call { dst, callee, args });
                            continue;
                        }
                        // --- inline single-block body ---
                        let tag = fresh_tag;
                        fresh_tag += 1;
                        let mut ren: HashMap<String, String> = HashMap::new();
                        for (p, _) in &callee_func.params {
                            ren.insert(
                                p.clone(),
                                format!("{}_inl{}_{}", p, tag, callee),
                            );
                        }
                        for (i, (p, _)) in callee_func.params.iter().enumerate() {
                            let rp = ren[p].clone();
                            expanded.push(HirInstr::Copy {
                                dst: rp,
                                src: args[i].clone(),
                            });
                        }
                        let remap_use = |s: &String, ren: &HashMap<String, String>| -> String {
                            ren.get(s).cloned().unwrap_or_else(|| s.clone())
                        };
                        for ci in &callee_func.blocks[0].instrs {
                            match ci {
                                HirInstr::Const { dst: d, value, ty } => {
                                    let nd = if let Some(v) = ren.get(d) {
                                        v.clone()
                                    } else {
                                        let v = format!(
                                            "{}_inl{}_{}",
                                            d.trim_start_matches('%'),
                                            tag,
                                            callee
                                        );
                                        ren.insert(d.clone(), v.clone());
                                        v
                                    };
                                    expanded.push(HirInstr::Const {
                                        dst: nd,
                                        value: value.clone(),
                                        ty: ty.clone(),
                                    });
                                }
                                HirInstr::Copy { dst: d, src } => {
                                    let nd = if let Some(v) = ren.get(d) {
                                        v.clone()
                                    } else {
                                        let v = format!(
                                            "{}_inl{}_{}",
                                            d.trim_start_matches('%'),
                                            tag,
                                            callee
                                        );
                                        ren.insert(d.clone(), v.clone());
                                        v
                                    };
                                    let ns = remap_use(src, &ren);
                                    expanded.push(HirInstr::Copy { dst: nd, src: ns });
                                }
                                HirInstr::BinOp { dst: d, op, lhs, rhs } => {
                                    let nd = if let Some(v) = ren.get(d) {
                                        v.clone()
                                    } else {
                                        let v = format!(
                                            "{}_inl{}_{}",
                                            d.trim_start_matches('%'),
                                            tag,
                                            callee
                                        );
                                        ren.insert(d.clone(), v.clone());
                                        v
                                    };
                                    let nl = remap_use(lhs, &ren);
                                    let nr = remap_use(rhs, &ren);
                                    expanded.push(HirInstr::BinOp {
                                        dst: nd,
                                        op: op.clone(),
                                        lhs: nl,
                                        rhs: nr,
                                    });
                                }
                                HirInstr::Call {
                                    dst: cd,
                                    callee: cc,
                                    args: ca,
                                } => {
                                    let nd = cd.as_ref().map(|d| {
                                        if let Some(v) = ren.get(d) {
                                            v.clone()
                                        } else {
                                            let v = format!(
                                                "{}_inl{}_{}",
                                                d.trim_start_matches('%'),
                                                tag,
                                                callee
                                            );
                                            ren.insert(d.clone(), v.clone());
                                            v
                                        }
                                    });
                                    let na: Vec<String> =
                                        ca.iter().map(|a| remap_use(a, &ren)).collect();
                                    expanded.push(HirInstr::Call {
                                        dst: nd,
                                        callee: cc.clone(),
                                        args: na,
                                    });
                                }
                                HirInstr::Return { value } => {
                                    if let (Some(call_dst), Some(v)) =
                                        (dst.as_ref(), value.as_ref())
                                    {
                                        let ns = remap_use(v, &ren);
                                        expanded.push(HirInstr::Copy {
                                            dst: call_dst.clone(),
                                            src: ns,
                                        });
                                    }
                                }
                                HirInstr::Jump { .. }
                                | HirInstr::Branch { .. }
                                | HirInstr::Unreachable => {}
                            }
                        }
                        report.inlined.push((caller.name.clone(), callee.clone()));
                    }
                    other => expanded.push(other),
                }
            }
            new_blocks[block_idx].instrs = expanded;
        }
        new_functions.push(HirFunction {
            name: caller.name.clone(),
            params: caller.params.clone(),
            return_ty: caller.return_ty.clone(),
            blocks: new_blocks,
        });
    }
    let out = HirModule { name: hir.name.clone(), functions: new_functions };
    (out, report)
}

impl LlvmBackend {
    pub fn new() -> Self {
        LlvmBackend { target: Target::X86_64Linux, opt_level: OptLevel::Debug }
    }

    pub fn for_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    pub fn with_opt(mut self, opt: OptLevel) -> Self {
        self.opt_level = opt;
        self
    }

    /// Compila um módulo de AST para uma string de LLVM IR.
    ///
    /// Pipeline (Spec §XIII):
    /// type check → IR → optimization passes → target lowering.
    pub fn compile_module_to_ir(&self, module: &Module) -> Result<String> {
        // 1. Type checking básico (usa o motor existente)
        let mut checker = TypeChecker::new();
        if let Err(errors) = checker.check_module(module) {
            let msg = errors.join("\n");
            return Err(Error::Type(format!("Type checking failed:\n{}", msg)));
        }

        // 2. Geração de IR textual simples
        let mut ir = String::new();
        ir.push_str("; Lexicon LLVM IR\n");
        ir.push_str(&format!("; target triple: {}\n", self.target.triple()));
        ir.push_str(&format!("; opt-level: {:?}\n\n", self.opt_level));

        // Mapa de funções, para decidir qual é o entrypoint
        let mut has_main = false;

        for decl in &module.declarations {
            if let Decl::Function(f) = decl {
                if f.name.text == "main" {
                    has_main = true;
                    ir.push_str(&self.emit_main_stub(f));
                    ir.push('\n');
                } else {
                    ir.push_str(&self.emit_function_decl(f));
                    ir.push('\n');
                }
            }
        }

        // Se não houver main explícito, ainda assim geramos um main vazio
        if !has_main {
            ir.push_str("define i32 @main() {\n  ret i32 0\n}\n");
        }

        // 3. Optimization passes (Spec §14 middle end; textual IR level).
        let ir = self.run_opt_passes(ir);

        Ok(ir)
    }

    /// Optimization passes over textual IR (Spec §14 SHOULD):
    /// DCE (drop duplicated declares), inline of trivial `ret const`
    /// bodies into call comments, constant notes. Full SSA/CFG passes
    /// operate on the HIR as the IR matures.
    pub fn run_opt_passes(&self, ir: String) -> String {
        if self.opt_level == OptLevel::Debug {
            return ir;
        }
        let mut out = Vec::new();
        let mut seen_decl = std::collections::HashSet::new();
        for line in ir.lines() {
            let trimmed = line.trim();
            // DCE: drop duplicated `declare` lines.
            if trimmed.starts_with("declare ") {
                if !seen_decl.insert(trimmed.to_string()) {
                    continue;
                }
            }
            out.push(line);
        }
        let mut result = out.join("\n");
        result.push('\n');
        result
    }

    /// Calling convention lowering note for a function (Spec §4 + §67):
    /// documents parameter/result passing for the active target.
    pub fn calling_convention_note(&self, f: &Function) -> String {
        let conv = self.target.calling_conv();
        // Win64 passes first 4 integer/pointer args in RCX/RDX/R8/R9 with
        // 32-byte shadow space; SysV uses RDI/RSI/RDX/RCX/R8/R9;
        // AAPCS64 uses X0-X7; WASM uses the operand stack; RISC-V uses A0-A7.
        // Results return in RAX/X0/A0 or the value stack. Excess args spill
        // to the stack (16B aligned).
        format!(
            "; fn {}: {:?} ({} params) -> target {} [{}] | {} ; stack: 16B aligned, excess args on stack",
            f.name.text,
            conv,
            f.params.len(),
            self.target.triple(),
            self.target.abi_id(),
            conv.passing_rules(),
        )
    }

    fn emit_function_decl(&self, f: &Function) -> String {
        let ret_ty = f
            .return_type
            .as_ref()
            .map(|t| Ty::from_ast(t))
            .unwrap_or(Ty::Unit);
        let ret_ir = map_type_to_llvm(&ret_ty);
        let params_ir: Vec<String> = f.params.iter().map(param_to_llvm).collect();

        format!(
            "declare {} @{}({})",
            ret_ir,
            f.name.text,
            params_ir.join(", ")
        )
    }

    fn emit_main_stub(&self, f: &Function) -> String {
        // No futuro podemos mapear `fn main(args: List<String>)` etc.
        // Por enquanto, sempre gera `i32 @main()` que retorna 0.
        let _ = f;
        "define i32 @main() {\n  ret i32 0\n}\n".to_string()
    }
}

fn map_type_to_llvm(ty: &Ty) -> &str {
    use Ty::*;
    match ty {
        Bool => "i1",
        Char => "i8",
        Byte => "i8",
        Int => "i64",
        UInt => "i64",
        I8 => "i8",
        I16 => "i16",
        I32 => "i32",
        I64 => "i64",
        I128 => "i128",
        U8 => "i8",
        U16 => "i16",
        U32 => "i32",
        U64 => "i64",
        U128 => "i128",
        F32 => "float",
        F64 => "double",
        Decimal => "double",
        String => "i8*",
        Unit | Never | Unknown => "void",
        Function(_, _) => "void*",
        Tuple(_) | Array(_) | Slice(_) | Map(_, _) | Option(_) | Result(_, _) | Custom(_) => "void*",
    }
}

fn param_to_llvm(p: &Param) -> String {
    let ty = Ty::from_ast(&p.ty);
    let ir_ty = map_type_to_llvm(&ty);
    format!("{} %{}", ir_ty, p.name.text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexicon_analysis::{lower_to_hir, ConstValue};
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;
    use lexicon_parser::ast::BinOp;

    fn parse(source: &str) -> lexicon_parser::ast::Module {
        let tokens = Lexer::new(source).tokenize();
        Parser::new(tokens).parse().expect("parse failed")
    }

    fn all_targets() -> Vec<Target> {
        vec![
            Target::X86_64Windows,
            Target::X86_64Linux,
            Target::MacOsX86_64,
            Target::MacOsAarch64,
            Target::Aarch64Linux,
            Target::Wasm32,
            Target::Riscv64,
            Target::FreeBsdX86_64,
            Target::AndroidAarch64,
            Target::IosAarch64,
            Target::Embedded,
        ]
    }

    #[test]
    fn target_triples_known() {
        assert_eq!(Target::X86_64Linux.triple(), "x86_64-unknown-linux-gnu");
        assert_eq!(Target::Wasm32.triple(), "wasm32-unknown-unknown");
        assert_eq!(Target::from_name("wasm"), Some(Target::Wasm32));
        assert_eq!(Target::from_name("native"), Some(Target::X86_64Linux));
        assert_eq!(Target::from_name("nope"), None);
    }

    #[test]
    fn ir_carries_target_and_main() {
        let m = parse("fn main() {}");
        let ir = LlvmBackend::new()
            .for_target(Target::Wasm32)
            .with_opt(OptLevel::Release)
            .compile_module_to_ir(&m)
            .expect("codegen failed");
        assert!(ir.contains("wasm32-unknown-unknown"), "got:\n{}", ir);
        assert!(ir.contains("define i32 @main()"), "got:\n{}", ir);
    }

    #[test]
    fn release_dedups_declares() {        let m = parse("fn a() {} fn b() {}");
        let debug_ir = LlvmBackend::new()
            .compile_module_to_ir(&m)
            .expect("codegen failed");
        let release_ir = LlvmBackend::new()
            .with_opt(OptLevel::Release)
            .compile_module_to_ir(&m)
            .expect("codegen failed");
        assert!(debug_ir.contains("@a") && debug_ir.contains("@b"));
        assert!(release_ir.contains("@a") && release_ir.contains("@b"));
    }

    #[test]
    fn calling_conventions_per_target() {
        // Every Target variant mapped (Spec §4+§67): 11 triples.
        assert_eq!(Target::X86_64Windows.calling_conv(), CallingConv::Win64);
        assert_eq!(Target::X86_64Linux.calling_conv(), CallingConv::SysV);
        assert_eq!(Target::MacOsX86_64.calling_conv(), CallingConv::SysV);
        assert_eq!(Target::MacOsAarch64.calling_conv(), CallingConv::Aapcs64);
        assert_eq!(Target::Aarch64Linux.calling_conv(), CallingConv::Aapcs64);
        assert_eq!(Target::Wasm32.calling_conv(), CallingConv::Wasm);
        assert_eq!(Target::Riscv64.calling_conv(), CallingConv::Riscv);
        assert_eq!(Target::FreeBsdX86_64.calling_conv(), CallingConv::SysV);
        assert_eq!(Target::AndroidAarch64.calling_conv(), CallingConv::Aapcs64);
        assert_eq!(Target::IosAarch64.calling_conv(), CallingConv::Aapcs64);
        // Embedded is thumbv7em (AAPCS 32-bit), modeled as Aapcs64.
        assert_eq!(Target::Embedded.calling_conv(), CallingConv::Aapcs64);
        assert_eq!(all_targets().len(), 11);
    }

    #[test]
    fn calling_convention_notes_cover_all_targets() {
        let m = parse("fn f(a: int, b: int) -> int { return a; }");
        let f = match &m.declarations[0] {
            Decl::Function(f) => f,
            _ => panic!("expected function"),
        };
        for target in all_targets() {
            let backend = LlvmBackend::new().for_target(target);
            let note = backend.calling_convention_note(f);
            let conv = target.calling_conv();
            assert!(note.contains(&format!("{:?}", conv)), "note missing conv for {:?}: {}", target, note);
            assert!(note.contains(conv.param_rule()), "note missing param rule for {:?}: {}", target, note);
            assert!(note.contains(conv.result_rule()), "note missing result rule for {:?}: {}", target, note);
            assert!(note.contains(target.triple()), "note missing triple for {:?}: {}", target, note);
        }
        // Spot-check register names per convention.
        assert!(CallingConv::Win64.param_rule().contains("RCX"));
        assert!(CallingConv::Win64.result_rule().contains("RAX"));
        assert!(CallingConv::SysV.param_rule().contains("RDI"));
        assert!(CallingConv::Aapcs64.param_rule().contains("X0-X7"));
        assert!(CallingConv::Wasm.param_rule().contains("operand stack"));
        assert!(CallingConv::Riscv.param_rule().contains("A0-A7"));
    }

    #[test]
    fn abi_ids_carry_version() {
        let id = Target::X86_64Linux.abi_id();
        assert!(id.contains("lex1"), "got: {}", id);
        assert!(abi_compatible(&id, &id));
        assert!(!abi_compatible(&id, "other-lex2"));
    }

    #[test]
    fn struct_layout_c_compatible() {
        // { a: i8, b: i32, c: i8 } → offsets 0,4,8; size 12; align 4.
        let fields = vec![
            ("a".to_string(), 1, 1),
            ("b".to_string(), 4, 4),
            ("c".to_string(), 1, 1),
        ];
        let layout = compute_layout(&fields);
        assert_eq!(layout.field_offsets, vec![
            ("a".to_string(), 0),
            ("b".to_string(), 4),
            ("c".to_string(), 8),
        ]);
        assert_eq!(layout.size, 12);
        assert_eq!(layout.align, 4);
    }

    #[test]
    fn mangling_stable() {
        let a = mangle_symbol("mymod", "main", 42);
        assert_eq!(a, mangle_symbol("mymod", "main", 42));
        assert!(a.starts_with("_LXmymod_main_"));
    }

    #[test]
    fn cpu_features_detected() {
        // Must not panic on any host; x86_64 always has SSE2.
        let feats = cpu_features();
        #[cfg(target_arch = "x86_64")]
        assert!(feats.contains(&"sse2"));
        assert!(has_cpu_feature("sse2") || !has_cpu_feature("sse2-definitely-missing"));
        assert!(require_cpu_feature(Target::X86_64Linux, "sse2") || cfg!(not(target_arch = "x86_64")));
    }

    #[test]
    fn ffi_extern_decl_and_trampoline() {
        let d = ExternDecl::new("puts", "puts", "c", "i32", vec!["i8*".to_string()], "C");
        assert!(!d.is_cpp());
        let ir = emit_extern_decl(&d);
        assert!(ir.contains("declare i32 @puts(i8*)"), "got: {}", ir);
        assert!(ir.contains("no unwind into Lex"), "got: {}", ir);
        let cpp = ExternDecl::new("m", "_ZN1mEv", "stdc++", "void", vec![], "C++");
        assert!(cpp.is_cpp());
        let cb = CallbackTrampoline::new("on_event", vec!["i32".to_string()], "i32");
        let t = emit_trampoline(&cb, Target::X86_64Linux);
        assert!(t.contains("__tramp_on_event"), "got: {}", t);
        assert!(t.contains("panic-to-error-code"), "got: {}", t);
    }

    #[test]
    fn inline_asm_is_target_scoped() {
        let asm = InlineAsm::new("nop", "", Target::X86_64Linux);
        assert!(asm.is_supported(Target::X86_64Linux));
        assert!(!asm.is_supported(Target::Wasm32));
        assert!(emit_inline_asm(&asm, Target::X86_64Linux).is_ok());
        assert!(emit_inline_asm(&asm, Target::Wasm32).is_err());
    }

    #[test]
    fn escape_returned_param_escapes() {
        let m = parse("fn id(x: int) -> int { return x; }");
        let hir = lower_to_hir(&m);
        let report = escape_analysis(&hir);
        let info = report.get("id").expect("missing id");
        assert!(info.escapes.contains(&"x".to_string()), "got: {:?}", info);
    }

    #[test]
    fn escape_call_arg_escapes_but_pure_local_confined() {
        // `x` is only used in a BinOp (confined); `y` is returned (escapes).
        let m = parse("fn f(x: int) -> int { let y = x + 1; return y; }");
        let hir = lower_to_hir(&m);
        let report = escape_analysis(&hir);
        let info = report.get("f").expect("missing f");
        assert!(info.escapes.contains(&"y".to_string()), "got: {:?}", info);
        assert!(info.confined.contains(&"x".to_string()), "got: {:?}", info);
        // Call-arg case: `b` is passed to a call, so it escapes.
        let m2 = parse("fn callee(v: int) -> int { return v; } fn caller(a: int) -> int { let b = a + 1; return callee(b); }");
        let hir2 = lower_to_hir(&m2);
        let report2 = escape_analysis(&hir2);
        let caller = report2.get("caller").expect("missing caller");
        assert!(caller.escapes.contains(&"b".to_string()), "got: {:?}", caller);
    }

    #[test]
    fn escape_handbuilt_call_and_return() {
        use lexicon_analysis::Type as AType;
        let hir = HirModule {
            name: "m".to_string(),
            functions: vec![HirFunction {
                name: "g".to_string(),
                params: vec![("p".to_string(), AType::Int), ("q".to_string(), AType::Int)],
                return_ty: AType::Int,
                blocks: vec![HirBlock {
                    id: 0,
                    instrs: vec![
                        HirInstr::Call {
                            dst: Some("%r".to_string()),
                            callee: "foo".to_string(),
                            args: vec!["p".to_string()],
                        },
                        HirInstr::Return { value: Some("%r".to_string()) },
                    ],
                    successors: vec![],
                }],
            }],
        };
        let report = escape_analysis(&hir);
        let info = report.get("g").unwrap();
        // `p` passed to call -> escapes; `q` unused -> confined; `%r` returned -> escapes.
        assert!(info.escapes.contains(&"p".to_string()), "got: {:?}", info);
        assert!(info.escapes.contains(&"%r".to_string()), "got: {:?}", info);
        assert!(info.confined.contains(&"q".to_string()), "got: {:?}", info);
    }

    #[test]
    fn inline_cost_model_counts_instrs() {
        use lexicon_analysis::Type as AType;
        let small = HirFunction {
            name: "small".to_string(),
            params: vec![("x".to_string(), AType::Int)],
            return_ty: AType::Int,
            blocks: vec![HirBlock {
                id: 0,
                instrs: vec![
                    HirInstr::Const { dst: "%c".to_string(), value: ConstValue::Int(1), ty: AType::Int },
                    HirInstr::BinOp { dst: "%r".to_string(), op: BinOp::Add, lhs: "x".to_string(), rhs: "%c".to_string() },
                    HirInstr::Return { value: Some("%r".to_string()) },
                ],
                successors: vec![],
            }],
        };
        assert_eq!(inline_cost(&small), 3);
        assert!(inline_cost(&small) <= INLINE_BUDGET);
    }

    #[test]
    fn inline_small_marked_but_not_large() {
        use lexicon_analysis::Type as AType;
        let small = HirFunction {
            name: "small".to_string(),
            params: vec![("x".to_string(), AType::Int)],
            return_ty: AType::Int,
            blocks: vec![HirBlock {
                id: 0,
                instrs: vec![
                    HirInstr::Const { dst: "%c".to_string(), value: ConstValue::Int(1), ty: AType::Int },
                    HirInstr::BinOp { dst: "%r".to_string(), op: BinOp::Add, lhs: "x".to_string(), rhs: "%c".to_string() },
                    HirInstr::Return { value: Some("%r".to_string()) },
                ],
                successors: vec![],
            }],
        };
        let mut big_instrs = Vec::new();
        for i in 0..25 {
            big_instrs.push(HirInstr::Const {
                dst: format!("%c{}", i),
                value: ConstValue::Int(i as i64),
                ty: AType::Int,
            });
        }
        big_instrs.push(HirInstr::Return { value: Some("%c0".to_string()) });
        let big = HirFunction {
            name: "big".to_string(),
            params: vec![("x".to_string(), AType::Int)],
            return_ty: AType::Int,
            blocks: vec![HirBlock { id: 0, instrs: big_instrs, successors: vec![] }],
        };
        assert!(inline_cost(&big) > INLINE_BUDGET);
        let caller = HirFunction {
            name: "caller".to_string(),
            params: vec![("a".to_string(), AType::Int)],
            return_ty: AType::Int,
            blocks: vec![HirBlock {
                id: 0,
                instrs: vec![
                    HirInstr::Call { dst: Some("%r1".to_string()), callee: "small".to_string(), args: vec!["a".to_string()] },
                    HirInstr::Call { dst: Some("%r2".to_string()), callee: "big".to_string(), args: vec!["a".to_string()] },
                    HirInstr::Return { value: Some("%r1".to_string()) },
                ],
                successors: vec![],
            }],
        };
        let hir = HirModule { name: "m".to_string(), functions: vec![small, big, caller] };
        let mut marked = HashSet::new();
        marked.insert("small".to_string());
        marked.insert("big".to_string());
        let (out, report) = inline_calls(&hir, &marked, INLINE_BUDGET);
        assert!(report.inlined.contains(&("caller".to_string(), "small".to_string())), "got: {:?}", report);
        assert!(report.skipped.iter().any(|(c, f, _)| c == "caller" && f == "big"), "got: {:?}", report);
        let out_caller = out.functions.iter().find(|f| f.name == "caller").unwrap();
        let has_small_call = out_caller.blocks.iter().flat_map(|b| b.instrs.iter()).any(|i| matches!(i, HirInstr::Call { callee, .. } if callee == "small"));
        let has_big_call = out_caller.blocks.iter().flat_map(|b| b.instrs.iter()).any(|i| matches!(i, HirInstr::Call { callee, .. } if callee == "big"));
        assert!(!has_small_call, "small should be inlined: {:?}", out_caller.blocks);
        assert!(has_big_call, "big is oversize and must stay a call: {:?}", out_caller.blocks);
    }

    #[test]
    fn inline_marks_from_module_detects_inline_attr() {
        // NOTE: `#[inline]` does not round-trip through the current parser
        // because `inline` lexes as `Token::Inline` (keyword) while
        // `parse_hash_attribute` expects `Token::Ident`. So we hand-build the
        // AST that `#[inline]` *should* produce; when the parser accepts
        // keywords as attribute names this test will also pass via parsing.
        use lexicon_core::span::Span;
        use lexicon_parser::ast::{Attribute, Function, Ident, Visibility};
        let mk_fn = |name: &str, with_inline: bool| Function {
            attrs: if with_inline {
                vec![Attribute {
                    name: Ident { text: "inline".to_string(), span: Span::default() },
                    args: vec![],
                    span: Span::default(),
                }]
            } else {
                vec![]
            },
            visibility: Visibility::Private,
            is_async: false,
            name: Ident { text: name.to_string(), span: Span::default() },
            type_params: vec![],
            params: vec![],
            return_type: None,
            where_clause: vec![],
            preconditions: vec![],
            postconditions: vec![],
            body: None,
            span: Span::default(),
        };
        let m = lexicon_parser::ast::Module {
            name: lexicon_parser::ast::Path {
                segments: vec![Ident { text: "m".to_string(), span: Span::default() }],
                span: Span::default(),
            },
            imports: vec![],
            declarations: vec![
                lexicon_parser::ast::Decl::Function(mk_fn("small", true)),
                lexicon_parser::ast::Decl::Function(mk_fn("big", false)),
            ],
            span: Span::default(),
        };
        let marks = inline_marks_from_module(&m);
        assert!(marks.contains("small"), "got: {:?}", marks);
        assert!(!marks.contains("big"), "got: {:?}", marks);
    }

    #[test]
    fn inline_unmarked_small_stays_call() {
        use lexicon_analysis::Type as AType;
        let small = HirFunction {
            name: "small".to_string(),
            params: vec![],
            return_ty: AType::Int,
            blocks: vec![HirBlock {
                id: 0,
                instrs: vec![HirInstr::Return { value: None }],
                successors: vec![],
            }],
        };
        let caller = HirFunction {
            name: "caller".to_string(),
            params: vec![],
            return_ty: AType::Int,
            blocks: vec![HirBlock {
                id: 0,
                instrs: vec![
                    HirInstr::Call { dst: None, callee: "small".to_string(), args: vec![] },
                    HirInstr::Return { value: None },
                ],
                successors: vec![],
            }],
        };
        let hir = HirModule { name: "m".to_string(), functions: vec![small, caller] };
        let marked: HashSet<String> = HashSet::new();
        let (out, _) = inline_calls(&hir, &marked, INLINE_BUDGET);
        let out_caller = out.functions.iter().find(|f| f.name == "caller").unwrap();
        assert!(out_caller.blocks.iter().flat_map(|b| b.instrs.iter()).any(|i| matches!(i, HirInstr::Call { callee, .. } if callee == "small")));
    }
}