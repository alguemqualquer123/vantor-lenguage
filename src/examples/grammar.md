---
version: 0.1
source_of_truth:
  - src/lexicon-lexer/src/tokens.rs
  - src/lexicon-lexer/src/tokenizer.rs
  - src/lexicon-parser/src/parser.rs
  - src/lexicon-parser/src/ast/mod.rs
---

# Lexicon Grammar v0.1 (tooling / highlighting reference)

This file is the formal grammar artifact (Spec §1). It documents what the
lexer actually emits and what `parser.rs` actually accepts. Anything marked
**reserved** is lexed but has no parser production yet — highlight it, do not
assume it parses. Do not invent constructs outside this file.

## 1. Token classes

### 1.1 Keywords

From `Token::keyword` (case-sensitive; `Some`/`None` are capitalized):

```
module import pub private protected class struct enum trait interface type
fn async return defer let var mut const static if else match switch case
default do for while loop break continue try catch throw finally await
macro task channel actor self super where as is lambda new in unsafe
extern inline native panic recover true false null extends implements
effect constructor dynamic function spawn select with derive Some None
yield ref
```

`try/catch/throw/finally`, `task/channel/actor`, `spawn/select/with`,
`yield/ref`, `macro/derive`, `extern/native`, `effect/dynamic/function`,
`panic/recover`, `private/protected`, `const/static` (as decl modifiers) are
**reserved**: lexed, but `parser.rs` has no statement/expression production
for them (see §6).

`@` maps to `Token::At` in `Token::keyword`, but the tokenizer has no `'@'`
arm, so a literal `@` currently lexes as `Error`. In practice attributes use
the `#[...]` form. `@name(...)` is parsed by `parse_attribute` but unreachable
from the lexer today.

### 1.2 Literals

| Class | Lexer form | AST |
|---|---|---|
| Decimal int | `[0-9][0-9_]*` e.g. `123`, `1_000` | `Literal::Int(i64)` |
| Hex int | `0x`/`0X` + `[0-9a-fA-F_]+` e.g. `0x1A` | `IntLit("0x1A")` → `Literal::Int` via `str::parse`, which fails on `0x` and falls back to `0` (known limitation) |
| Binary int | `0b`/`0B` + `[01_]+` e.g. `0b101` | Same fallback limitation as hex |
| Float | digits with `.` and/or `e/E` exponent, `_` allowed, e.g. `1.23`, `1.23e-4` | `FloatLit` → `Literal::Float(f64)` (fallback `0.0`) |
| String | `"..."` with escapes `\n \t \r \\ \" \'`; `{name}` inside triggers interpolation heuristic → `Expr::InterpolatedString` | `Literal::String` or `InterpolatedString` |
| Char | `'c'` (exactly one char + closing `'`) | `CharLit(char)`; unterminated → `Error` |
| Bool | `true` / `false` | `Literal::Bool` |
| Null | `null` | `Literal::Null` |

No octal literal. No suffix lexing (`i32`/`f64` suffixes exist only as
`IntSuffix`/`FloatSuffix` AST data, never produced by the tokenizer).
Array/tuple literals exist as `Literal::Array/Tuple` in the AST but have no
parser production.

### 1.3 Operators (lexed)

```
== != <= >= && || => -> :: ?? ?. !! ++ -- += -= *= /= %=
<< >> + - * / % ^ ~ ! & | ? : . .. ..= , ; : # ▷
```

`▷` lexes directly as `PipeRArrow`. `|>` also lexes as `PipeRArrow`
(`scan_pipe`). `...` (variadic) lexes as `DotDot` + `Dot`.

**Actually parsed as operators** (see §4 precedence):

```
as  =  ? :  ??  ||  &&  == !=  < <= > >=  + -  * / %
! - *(deref) await (unary)  () call  . ?. ::  |...| lambda
```

Everything else (`++ -- += -= *= /= %= << >> & | ^ ~`) is lexed but has no
binary-expression production. `=` is parsed as assignment but desugared to
`Binary(AddEq)` as a placeholder. `|>`/`▷` (`PipeRArrow`) is lexed and
`parse_pipe` exists, but `parse_pipe` is never called (dead code), so the pipe
operator does not parse today.

### 1.4 Delimiters

```
( )  [ ]  { }  ,  ;  :  ::  .  ?  #  ->  =>  :
```

`;` is optional after most statements (`expect(Semi).ok()`). Field separators
in `struct` accept `;` or `,`, optional before `}`.

### 1.5 Comments / whitespace / identifiers

- `//...` → `Comment`, `/*...*/` → `BlockComment` (unterminated → `Error`).
- Whitespace `space \t \n \r` skipped.
- Identifiers: `[A-Za-z_][A-Za-z0-9_]*` minus keywords.
- Anything else → `Error("Unexpected character: …")`.

## 2. Modules and imports (EBNF-ish)

```ebnf
module    := ["module" path ";"] import* declaration* EOF ;
path      := ident ("::" ident)* ;
import    := "import" path ["as" ident] ";" ;
```

Module header defaults to `main` when absent. Imports use `::` paths with an
optional `as` alias.

## 3. Declarations

```ebnf
declaration := attr* (
                 function | class | struct | enum
               | trait | interface | type_alias | global_var
               ) ;
attr        := "#[" ident ["(" attr_arg ("," attr_arg)* ")"] "]" ;
(* `@name(...)` exists in parse_attribute but is lexer-unreachable; use `#[...]` *)
attr_arg    := cfg_pred | expression ;
cfg_pred    := ident ["=" (string | ident)] ;   (* only inside #[cfg(...)] *)

function    := ["pub"] ["async"] "fn" ident [generics]
               "(" [param ("," param)*] ")" [where_clause]
               ["->" type] ("{" block "}" | ";") ;
param       := ident ":" type ["..."] ;   (* trailing "..." = variadic, must be last *)
generics    := "<" ident [":" type ("," type)*] ("," ...)* ">" ;
where_clause:= "where" type ":" type (("&" | ",") type)* ;

class       := ["pub"] "class" ident [generics]
               ["extends" type] ["implements" type ("," type)*]
               "{" (method | constructor | field)* "}" ;
constructor := "constructor" "(" [param ...] ")" block ;
field       := ["pub"] ["mut"] ident ":" type [";" | ","] ;

struct      := ["pub"] "struct" ident [generics] "{" field* "}" ;
enum        := "enum" ident [generics] "{" [ident ("," ident)* ","] "}" ;
(* enum variants are unit-only today; payload types are not parsed *)

trait       := ["pub"] "trait" ident [generics] "{" method_sig* "}" ;
interface   := ["pub"] "interface" ident [generics] "{" method_sig* "}" ;
method_sig  := ["pub"] ["async"] "fn" ident "(" [param ...] ")" ["->" type] ";" ;

type_alias  := ["pub"] "type" ident [generics] "=" type ";" ;
global_var  := ["pub"] ["static"] ["mut"] ident [":" type] ["=" expression] ";" ;
```

Notes:

- Declaration-level modifiers skipped for dispatch: `pub private protected
  async static const`. Method/field modifiers also accept `mut`.
- `Effect` exists in the AST but has no parser production (reserved).
- `Function` has `preconditions/postconditions` fields but the parser never
  fills them (always empty).
- `#[cfg(...)]` is the only special attribute: `#[cfg(feature = "net")]`,
  `#[cfg(test)]`, `#[cfg(target = "...")]`. Stored as string-literal args
  (`feature=net`) for compile-time evaluation.
- Known limitation: `#[inline]` / `@inline` do not round-trip today because
  `inline` lexes as `Token::Inline` (keyword) while `parse_hash_attribute` /
  `parse_attribute` expect `Token::Ident`. The HIR inliner (`inline_calls` +
  `inline_marks_from_module` in `lexicon-codegen`) accepts the AST once the
  parser allows keywords as attribute names.

## 4. Statements

```ebnf
statement := if_stmt | switch_stmt | do_while | for_stmt | while_stmt
           | loop_stmt | match_stmt | return_stmt | break_stmt | cont_stmt
           | defer_stmt | let_stmt | expr_stmt ;
if_stmt   := "if" expression block ["else" block] ;
(* no `else if` production; nest `else { if ... }` instead *)
switch_stmt := "switch" expression "{"
               ("case" expression ("," expression)* ":" (block | statement*)
                | "default" ":" (block | statement*))* "}" ;
do_while  := "do" block "while" expression [";"] ;   (* Spec §3 SHOULD *)
for_stmt  := "for" pattern "in" expression block ;
while_stmt:= "while" expression block ;
loop_stmt := "loop" block ;
match_stmt:= "match" expression "{"
               (pattern ["if" expression] "=>" (expression | "return" [expression]
                | "break" | "continue") ","?)* "}" ;
return_stmt := "return" [expression] [";"] ;
break_stmt  := "break" [ident] [";"] ;    (* optional label *)
cont_stmt   := "continue" [ident] [";"] ;
defer_stmt  := "defer" expression [";"] ;
let_stmt    := ("let" | "var") ["mut"] ident [":" type] ["=" expression] [";"] ;
expr_stmt   := expression [";"] ;
```

`Stmt::AsyncBlock` and `Stmt::Block` exist in the AST but have no statement
production. `try/catch`, `throw`, `async` blocks as statements are reserved.

## 5. Expressions (precedence, lowest → highest)

```ebnf
expression  := assignment ;
assignment  := ternary ["=" assignment] | ternary "as" type ;
ternary     := null_coalesce ["?" expression ":" ternary] ;
null_coalesce := logical_or ("??" logical_or)* ;
logical_or  := logical_and ("||" logical_and)* ;
logical_and := equality ("&&" equality)* ;
equality    := comparison (("==" | "!=") comparison)* ;
comparison  := term (("<" | "<=" | ">" | ">=") term)* ;
term        := factor (("+" | "-") factor)* ;
factor      := unary (("*" | "/" | "%") unary)* ;
unary       := ("!" | "-" | "*" | "await" unary) | call_member ;
call_member := primary (call | method | field | null_field | path | lambda)* ;
call        := "(" [expression ("," expression)*] ")" ;
method      := ("." | "?." ) ident ["<" type ("," type)* ">"]
               "(" [expression ("," expression)*] ")" ;
field       := ("." | "?.") ident ;
path        := "::" ident ;
lambda      := "|" [param ("," param)*] "|" expression ;
primary     := "true" | "false" | "null" | int | float | string | ident
             | "(" expression ")" | struct_lit | "unsafe" block ;
struct_lit  := ident "{" [ident [":" expression] ("," ...)*] "}" ;
(* shorthand `User { id }` means `User { id: id }`; empty `Name {}` allowed *)
```

Additional truths:

- `unsafe { ... }` block expressions are parsed (Spec §32).
- Interpolated strings split on `{...}` and re-parse the inside as an
  expression.
- `Expr::Index`, `Try`, `Spawn`, `Select`, `With`, `MacroCall`,
  destructuring, ranges are AST-only (reserved).

## 6. Types and patterns

```ebnf
type    := "(" [type ("," type)*] ")"            (* tuple *)
         | path ["<" type ("," type)* ">"] ["?"] ; (* path / generic / nullable *)
pattern := "_" | ident | literal
         | "(" [pattern ("," pattern)*] ")"
         | "[" [pattern ("," pattern)*] "]"
         | "{" ident ":" pattern ("," ...)* "}"
         | ident "(" [pattern ("," pattern)*] ")"  (* enum variant *)
         | ident "@" pattern ;                     (* named *)
```

`Array/Slice/Map/Function/Reference/Pointer/Dynamic` exist in `ast::Type` but
only `Path`, `Generic`, `Nullable`, and `Tuple` are produced by `parse_type`.

## 7. Reserved (lexed, not parsed) — for highlighters

`try catch throw finally task channel actor spawn select with yield ref macro
derive extern native effect dynamic function panic recover private protected`.
Highlight as keywords; do not offer completions/snippet bodies that assume
parser support.

## 8. Tooling notes

- Grammar version is `0.1` (header block). Bump only with a parser change.
- `switch`, `do-while`, `unsafe` blocks, and `#[cfg(...)]` are implemented —
  highlight and complete them (unlike the reserved list above).
- Variadic params use trailing `...` (`name: T...`); callers pass trailing
  args; `typeck` enforces last-position.
- `match` arms accept tail expressions plus `return`/`break`/`continue`
  (desugared to unit) and optional `if` guards.
