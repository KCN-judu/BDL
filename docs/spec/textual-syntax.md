---
kind: specification
area: textual
status: current
---

# BDL textual syntax (v0.1 core, v0.2 project items)

Normative for the textual authoring surface and for `crates/bdl-syntax`. Nothing
is implemented in the parser that is not described here; when the two disagree,
this document is wrong or the parser is, and the fix is recorded in both.

The textual syntax is **another authoring surface over the same semantic model**
(`docs/architecture/overview.md`, ADR-0001). It is not a second language: every
construct here lowers to the surface model that the canvas edits, and from there
elaborates through the same pipeline (`docs/architecture/compiler-pipeline.md`).
The parser answers _what did the user write?_; meaning is decided later.

Design lineage: Haskell contributes the declaration discipline (a type signature
first, a definition second, both naming the same thing); Rust and MoonBit
contribute the structural syntax (calls with parentheses, braces, `match`,
constructors, generics, patterns). Deliberately absent: layout rule, significant
whitespace, juxtaposition application, user-defined operators and fixities,
multiple function equations.

---

## 1. A source file

```bdl
// Lamp behaviour

concept Tilt : Angle
concept Brightness : Scalar
concept Held : Bool

mapping dimByTilt : Tilt -> Brightness
dimByTilt(tilt) =
  clamp(tilt / (90 deg), 0, 1)

mapping chooseBrightness : Held -> Tilt -> Brightness
chooseBrightness(held, tilt) =
  if held then
    dimByTilt(tilt)
  else
    0

enum LampMode {
  Off,
  Automatic,
  Manual(Brightness),
}

mapping resolve : LampMode -> Tilt -> Brightness
resolve(mode, tilt) =
  match mode {
    Off => 0,
    Automatic => dimByTilt(tilt),
    Manual(value) => value,
  }
```

A file is a sequence of items. Items begin with a keyword (`concept`, `mapping`,
`enum`) and end where the next item's keyword or the end of the file begins.
Newlines never carry meaning.

---

## 2. Lexical grammar

Source is UTF-8. Every byte of the source belongs to exactly one token; the
token sequence concatenated is the source (the tree is lossless).

```ebnf
Whitespace   ::= ( " " | "\t" | "\r" | "\n" | "\f" )+
LineComment  ::= "//" ( any char except "\n" )*
BlockComment ::= "/*" ( any char )* "*/"          (* not nested; see §2.4 *)

Ident        ::= [A-Za-z_] [A-Za-z0-9_]*          (* except "_" and keywords *)
Underscore   ::= "_"
Number       ::= Digits ( "." Digits )? Exponent?
              |  "." Digits Exponent?
Digits       ::= [0-9]+
Exponent     ::= ( "e" | "E" ) ( "+" | "-" )? Digits
```

### 2.1 Keywords

Reserved and lexed as keyword tokens (never identifiers):

| keyword            | used by                               |
| ------------------ | ------------------------------------- |
| `concept`          | concept declarations                  |
| `mapping`          | mapping declarations                  |
| `enum`             | enum declarations                     |
| `match`            | match expressions                     |
| `let`              | let statements                        |
| `if` `then` `else` | if expressions                        |
| `true` `false`     | boolean literals and literal patterns |
| `in`               | membership: `x in [a, b]` (§15)       |
| `ordered`          | `ordered concept …` (§15)             |

**Future-reserved** — lexed as keyword tokens today so that no v0.1 program uses
them as names, but with no grammar production. Writing one where a name is
expected reports _`context` is reserved for a future version of BDL_:

`context` · `output` · `clock` · `component` · `require`

### 2.2 Punctuation and operators

Longest match wins; `->`, `=>`, `==`, `!=`, `<=`, `>=`, `&&`, `||`, `..`, `??`
are single tokens, so `a->b` lexes as `a` `->` `b` and `a- >b` as `a` `-` `>`
`b`.

| token       | kind                                          | token           | kind                              |
| ----------- | --------------------------------------------- | --------------- | --------------------------------- |
| `(` `)`     | `LParen` `RParen`                             | `+` `-` `*` `/` | `Plus` `Minus` `Star` `Slash`     |
| `{` `}`     | `LBrace` `RBrace`                             | `!`             | `Bang`                            |
| `[` `]`     | `LBracket` `RBracket` (collections, §15)      |                 |                                   |
| `<` `>`     | `Lt` `Gt` (comparison _and_ generic brackets) | `<=` `>=`       | `Le` `Ge`                         |
| `:` `,` `;` | `Colon` `Comma` `Semi`                        | `==` `!=`       | `EqEq` `Ne`                       |
| `=`         | `Eq`                                          | `&&` `\|\|`     | `AndAnd` `OrOr`                   |
| `->`        | `Arrow`                                       | `=>`            | `FatArrow`                        |
| `?`         | `Question` (a slot, §16)                      | `..` `??`       | `DotDot` `QuestionQuestion` (§17) |

There is no `>>` token: `Option<Result<A, B>>` lexes as two `Gt` tokens. A lone
`&`, `|`, `.`, `#`, `@`, or any character outside the table is an `Error` token;
the lexer never stops and never panics. An `Error` token covers one whole UTF-8
character, so every token boundary is a character boundary.

### 2.3 Numbers are spelled, not valued

A `Number` token is its **source text**. The lexer does not convert it; the
syntax tree carries `"0.1"`, not an IEEE double. `bdl-syntax` exposes the
spelling and an exact decimal reading (`digits × 10^exponent`, so `0.1` is
`1 × 10⁻¹`), never an `f64`. Conversion to a machine number happens at the
elaboration boundary, is documented there as technical debt (DI-1, ADR-0011),
and will move behind the exact/symbolic numeric layer when that lands.
Consequences:

- `1e999` is a valid token. Whether it fits a runtime number is not a syntax
  question.
- Two literals are syntactically equal iff their spellings are equal; `1.0` and
  `1` are different literals with the same value.

### 2.4 Comments

`//` to end of line; `/* … */` not nested (`/* a /* b */` ends at the first
`*/`). Nesting was not adopted for v0.1 because it makes "comment out this
region" fail on any region containing `*/`, which is the more common mistake in
a designer-facing editor; revisit if the editor grows a comment-toggle that
needs it. An unterminated `/*` is a `BlockComment` token to the end of the file
plus one diagnostic.

Comments and whitespace are **trivia**: the parser looks past them, the tree
keeps them. A comment directly above an item (no blank line between) is attached
inside that item's node (so tooling can treat it as the item's documentation);
all other trivia sits between nodes in the enclosing node.

### 2.5 Identifiers

ASCII only in v0.1: `[A-Za-z_][A-Za-z0-9_]*`, excluding the keywords and the
lone `_`. Unicode identifiers are deferred (normalisation, confusables and code
generation would each need a policy). Since the text is the semantic source of
every project (ADR-0023), a name is an identifier on every surface: Studio
refuses a create or rename the text cannot spell (`edit.invalid_name`, with the
identifier it would be — _A name is one word, without spaces: `Light_Output`_),
and a display name in a legacy JSON project is mapped once, deterministically,
when the project is migrated (`bdl-text::identifier_from`: every run of other
characters becomes one `_`, a leading digit gets a `_` before it, a keyword or
an empty result gets a trailing `_`).

Capitalisation is a convention, not a rule: concepts and constructors are
conventionally `UpperCamel`, mappings and parameters `lowerCamel`. The parser
does not distinguish (see §8.1).

---

## 3. Syntax tree

The parser builds a **lossless concrete syntax tree** (Rowan): every token,
including trivia and error tokens, is a leaf; nodes are labelled with the
`SyntaxKind`s below. `root.text() == source` holds for every input, valid or
not. Typed AST wrappers (`bdl_syntax::ast`) are views over this tree and store
nothing of their own.

Node kinds:

```text
Module  Formula
ConceptDecl  MappingDecl  MappingDef  EnumDecl  EnumVariant
TypeParamList  TypeArgList  TypeList  ParamList  ArgList  PatternList
Name  NameRef  UnitSuffix
NamedType  FunctionType  ParenType
NameExpr  LiteralExpr  ParenExpr  CallExpr  UnaryExpr  BinaryExpr
IfExpr  MatchExpr  MatchArmList  MatchArm  BlockExpr  LetStmt
ListExpr  TupleExpr  LambdaExpr  LambdaParams  BinderExpr  RangeExpr
UnitType  TupleType  UnitExpr
WildcardPattern  IdentPattern  LiteralPattern  ConstructorPattern
ErrorNode
```

Token kinds are the lexical classes of §2 plus one kind per keyword and
punctuation mark, and `Error` for characters the lexer has no token for. The
node that wraps tokens skipped during recovery is `ErrorNode`, to keep it
distinct from the `Error` token.

Deviations from the kind list proposed for this milestone: the `Error` node is
`ErrorNode` (see above); `TypeExpr` / `GenericType` are `NamedType` (with an
optional `TypeArgList`) — one kind with an optional child is simpler for
consumers than two kinds; `Pattern` is the four concrete pattern kinds;
`Formula` is the root of the expression-only entry point used by the canvas
formula field; `Name` / `NameRef` wrap identifier tokens at definition and
reference sites so rename tooling has one thing to look for.

---

## 4. Grammar (EBNF)

Trivia is omitted throughout. `X*` is zero or more, `X?` optional. Terminal
tokens are quoted; `Ident`, `Number` are the lexical classes above.

### 4.1 Module and items

```ebnf
Module          ::= Item* EOF

Item            ::= ConceptDecl | MappingDecl | EnumDecl

ConceptDecl     ::= "ordered"? "concept" Name ( ":" Type )?

MappingDecl     ::= "mapping" Name ":" Type MappingDef?
MappingDef      ::= NameRef "(" ParamList? ")" "=" Expr
ParamList       ::= Pattern ( "," Pattern )* ","?

EnumDecl        ::= "enum" Name TypeParamList? "{" ( EnumVariant ( "," EnumVariant )* ","? )? "}"
EnumVariant     ::= Name ( "(" TypeList ")" )?
TypeParamList   ::= "<" Name ( "," Name )* ","? ">"
TypeList        ::= Type ( "," Type )* ","?

Name            ::= Ident
NameRef         ::= Ident
```

A `MappingDef` belongs to the `MappingDecl` immediately before it. Its `NameRef`
is expected to repeat the declared name; the parser accepts any identifier and
lowering reports a mismatch (§8.2). A relationship without inputs is declared
`mapping level : () -> Brightness` — the **preferred spelling**: the empty
domain written out (§4.2) — and defined `level() = 0`.

**The legacy output-only shorthand.** `mapping level : Brightness` is
compatibility syntax for the same declaration: it parses, binds and elaborates
identically (one `Signature`, one canonical type `() -> Brightness`), and
nothing generated ever writes it — `bdl-text` (the Code view, write-back, a
migrated legacy project), the IDE's renderers and every example write `() -> B`.
The IDE reports it as a hint, never an error (`text.legacy_unit_domain`: _A
relationship with no inputs is written explicitly as `() -> RoomTemp`. The
output-only shorthand is deprecated._) with the quick fix _Make empty domain
explicit_ — one insertion of `() ->` before the signature; hover and Explain
show the declared spelling beside the canonical type. The hint is raised only
where a bare type means this shorthand — a `mapping`'s signature — never in a
concept's value form, an output's or a port's type. The formatter preserves the
authored spelling (§10): formatting is never a migration. Opt-in,
`bdld migrate-unit-domain <project>` rewrites every legacy signature of a
project losslessly (`bdl_syntax::migrate`: one insertion per signature, comments
and trivia untouched, identities and the design checked unchanged before
anything is written). Staged policy (ADR-0029 amendment): stage 1 — this; stage
2 — the hint becomes a warning, generated code never emits the shorthand
(already so); stage 3 — a language edition may remove it, with the migration
automatic. No removal is scheduled: there is no versioning policy for the
language yet. Port types (`requires n : T`, `param n : T`, `provides n : T`)
keep the bare output — a port's grammar has no domain to spell, and the policy
is about `mapping`.

**No Source keyword.** A relationship without inputs and without a definition —
`mapping tilt : () -> Tilt` alone — is what Studio and the IDE service present
as a _Source_: a value the environment provides, observed once per activation
(ADR-0032). The text has no keyword, attribute or comment for it, and none is
read: the role is derived from the declaration's shape and state wherever it is
shown, and adding a definition (`tilt() = …`) makes the same declaration an
ordinary relationship. A Source item of the Standard Library writes exactly this
pair — `concept RoomTemp : Temperature` and
`mapping TempSensor : () -> RoomTemp` — in the preferred spelling.

### 4.2 Types

```ebnf
Type            ::= TypeAtom ( "->" Type )?              (* "->" is right-associative *)
TypeAtom        ::= NamedType | ParenType | UnitType | TupleType
NamedType       ::= NameRef TypeArgList?
TypeArgList     ::= "<" Type ( "," Type )* ","? ">"
ParenType       ::= "(" Type ")"
UnitType        ::= "(" ")"                              (* the empty product *)
TupleType       ::= "(" Type ( "," Type )+ ","? ")"      (* a product domain *)
```

`A -> B -> C` is `A -> (B -> C)`; `(A -> B) -> C` needs the parentheses. A
relationship's type is one thing, `domain(inputs) -> B`, however it is spelled:

```text
domain([])        = ()                 the empty product
domain([A])       = A
domain([A, B, …]) = (A, B, …)          ≅  A -> B -> …  by currying
```

so `mapping f : () -> B` — and the legacy shorthand `mapping f : B`, §4.1 —
declare the one type `() -> B`, and `mapping f : (A, B) -> C` is
`mapping f : A -> B -> C`. `()` is the empty product — `Product([]) ≅ ()` —
never the word _unit_ (a measurement unit in BDL) and never `_`; it can only
open a signature: after an input (`A -> () -> B`), as the output, or as a
concept's value form it is refused (`text.unknown_concept` /
`binding.unknown_concept`, in those words). The kernel has no unit type: it
encodes `() -> B` as `B` (unit elimination, the unique argument erased) and
`(A, B) -> C` as `A -> B -> C` (`bdl_ir::ty`, ADR-0029), so a relationship
without inputs is not a category of its own — it keeps its identity,
realization, timing domain and dependencies, and is read as a value (`f`, the
application to the unique argument, §4.3). A concept is written by its name
(`Tilt`), never `Sem<Tilt>` — `sem`, `q`, `rep`, `mk` are kernel and explanation
vocabulary. Generic forms are syntax only; which of them exist and what they
mean is decided by elaboration: in a concept's value form `List<R>`,
`Pair<R₁, R₂>` and `Option<R>` over value forms (§15); anything else
(`Result<A, B>`) parses and is refused. There is no type-level computation.

### 4.3 Expressions

```ebnf
Expr            ::= OrExpr

OrExpr          ::= AndExpr ( "||" AndExpr )*
AndExpr         ::= EqExpr ( "&&" EqExpr )*
EqExpr          ::= RelExpr ( ( "==" | "!=" ) RelExpr )?          (* non-associative *)
RelExpr         ::= RangeExpr ( ( "<" | "<=" | ">" | ">=" | "in" ) RangeExpr )?  (* non-associative *)
RangeExpr       ::= CoalesceExpr ( ".." CoalesceExpr )?              (* §17; only after "in" *)
CoalesceExpr    ::= AddExpr ( "??" CoalesceExpr )?                    (* §17; right *)
AddExpr         ::= MulExpr ( ( "+" | "-" ) MulExpr )*
MulExpr         ::= UnaryExpr ( ( "*" | "/" ) UnaryExpr )*
UnaryExpr       ::= ( "!" | "-" ) UnaryExpr | PostfixExpr
PostfixExpr     ::= PrimaryExpr CallSuffix*
CallSuffix      ::= "(" ArgList? ")"
ArgList         ::= Expr ( "," Expr )* ","?

PrimaryExpr     ::= NameExpr | LiteralExpr | ParenExpr | TupleExpr | ListExpr
                  | LambdaExpr | IfExpr | MatchExpr | BlockExpr | SlotExpr
                  | UnitExpr | BinderExpr
SlotExpr        ::= "?"                                             (* §16 *)
UnitExpr        ::= "(" ")"                                         (* the unique value of () *)
BinderExpr      ::= ( "all" | "any" | "map" | "filter" ) Name "in" RangeExpr ":" Expr  (* §17 *)
NameExpr        ::= NameRef
LiteralExpr     ::= Number UnitSuffix? | "true" | "false"
UnitSuffix      ::= Ident
ParenExpr       ::= "(" Expr ")"
TupleExpr       ::= "(" Expr ( "," Expr )+ ","? ")"                 (* §15 *)
ListExpr        ::= "[" ( Expr ( "," Expr )* ","? )? "]"           (* §15 *)
LambdaExpr      ::= LambdaParams "=>" Expr                          (* §15 *)
LambdaParams    ::= Name | "(" Name ( "," Name )* ","? ")"
IfExpr          ::= "if" Expr "then" Expr "else" Expr
MatchExpr       ::= "match" Expr "{" MatchArm* "}"
MatchArm        ::= Pattern "=>" Expr ","?
BlockExpr       ::= "{" LetStmt* Expr "}"
LetStmt         ::= "let" Pattern "=" Expr ";"
```

The implementation is a Pratt parser over the precedence table in §5; the EBNF
above is its specification, not its shape. Additionally, a comparison may not be
a _direct_ operand of another comparison (§5.1); the EBNF's `?` on
`EqExpr`/`RelExpr` states the same-level case, the cross-level case
(`a == b < c`) is rejected by the parser with the same diagnostic.

### 4.4 Patterns

```ebnf
Pattern         ::= WildcardPattern | IdentPattern | LiteralPattern | ConstructorPattern
WildcardPattern ::= "_"
IdentPattern    ::= Name
LiteralPattern  ::= "true" | "false" | "-"? Number      (* Number must be a whole number *)
ConstructorPattern ::= NameRef "(" PatternList? ")"
PatternList     ::= Pattern ( "," Pattern )* ","?
```

Deliberately not in v0.1: `@` bindings, ranges, rest patterns, record
destructuring, or-patterns, guards.

---

## 5. Precedence and associativity

From loosest to tightest:

| level | operators                               | associativity | node         |
| ----- | --------------------------------------- | ------------- | ------------ |
| 1     | `\|\|`                                  | left          | `BinaryExpr` |
| 2     | `&&`                                    | left          | `BinaryExpr` |
| 3     | `==` `!=`                               | **none**      | `BinaryExpr` |
| 4     | `<` `<=` `>` `>=` `in`                  | **none**      | `BinaryExpr` |
| 4a    | `..`                                    | **none**      | `RangeExpr`  |
| 4b    | `??`                                    | right         | `BinaryExpr` |
| 5     | `+` `-`                                 | left          | `BinaryExpr` |
| 6     | `*` `/`                                 | left          | `BinaryExpr` |
| 7     | prefix `!` `-`                          | —             | `UnaryExpr`  |
| 8     | call `f(…)`                             | postfix, left | `CallExpr`   |
| 9     | `if` `match` `{ }` `( )` literals names | primary       | —            |

Pratt binding powers used by the parser: `||` 1/2, `&&` 3/4, `==` `!=` 5/6, `<`…
7/8, `..` 8/9, `??` 10/9, `+` `-` 11/12, `*` `/` 13/14, unary 15, call 17. A
binder (`all x in xs: body`, §17) is a primary expression whose body extends as
far right as possible, like `if`; its collection is parsed at the binding power
of `..`, so it stops at the colon.

Reading rules:

- `-x * y` is `(-x) * y`; `-f(x)` is `-(f(x))`; `!a && b` is `(!a) && b`.
- `f(x)(y)` parses as `CallExpr(CallExpr(f, x), y)`. Typing may reject it; the
  syntax admits it so that the grammar has no special case.
- `if`, `match` and blocks are primary expressions whose branch/arm/tail
  expressions extend as far right as possible: `if c then a else b + 1` is
  `if c then a else (b + 1)`, and `1 + if c then a else b` is legal.
- A unit suffix binds to its number only: `90 deg / 2` is `(90 deg) / 2`.
- `()` is the unique value of the empty product: `f(())` applies a relationship
  of type `() -> B` to it and means exactly `f` (and `f()`); anywhere else `()`
  is `formula.unit.not_a_value`. It is never an empty argument list (`f()` has
  none), a grouping (`(a)`), a grouped value (`(a, b)`) or a slot (`?`).
- A rule extends as far right as possible, like `if`:
  `any(xs, x => x < 30 deg && held)` gives the whole conjunction to the rule.
- `x in lo .. hi` is `x in (lo .. hi)`; `x + y in lo .. hi` is
  `(x + y) in (lo .. hi)`; `x in lo + d .. hi - d` is
  `x in ((lo + d) .. (hi - d))`; `x ?? d + 1` is `x ?? (d + 1)` and `x ?? 0 < 1`
  is `(x ?? 0) < 1` (§17).

### 5.1 Comparisons do not chain

`a < b < c`, `a == b == c` and `a == b < c` are syntax errors
(`syntax.chained_comparison`): _comparisons cannot be chained_, with the fix
_write `a < b && b < c`_ (or parentheses). The parser still builds the
left-nested tree so later tooling sees a complete expression, but the parse has
an error. C's silent `(a < b) < c` is not inherited.

---

## 6. Units

```ebnf
LiteralExpr ::= Number UnitSuffix?
UnitSuffix  ::= Ident
```

A unit is a bare identifier written after a number: `90 deg`, `25.4 mm`,
`2.5 s`. The space is optional (`90deg` lexes as `90` `deg` and parses the same
way) but canonical formatting writes one. This is unambiguous only because BDL
has no juxtaposition application — an identifier can never follow a complete
expression for any other reason. Rules:

- The lexer and parser do **not** know which identifiers are units. `90 foobar`
  parses; elaboration reports _`foobar` is not a unit_ (`formula.unit.unknown`).
  Units are semantic vocabulary.
- Units attach to number literals only. `x deg` and `f(x) mm` are syntax errors
  (an identifier after an expression), diagnosed as _expected an operator … — a
  unit can only follow a number, as in `90 deg`_.
- Dimensioned expressions get their dimension from typing, never from a postfix.
  Use parentheses and arithmetic: `distance / (2 s)`,
  `(tilt + offset) / (90 deg)`.
- The unit vocabulary is the registry in `bdl-elab::units` (FV Phase 10,
  `Surface/Units.lean`; Phase 10b `Surface/Charts.lean`): each unit has a stable
  id (`angle.deg`), a symbol, a dimension and a **chart** onto the canonical
  magnitude (radians, metres, seconds, kilograms, …) — linear (`scale`) or
  affine (`scale`, `offset`); the chart owns the conversion, and
  `convert(x, from, to)` is `coord_to ∘ reconstruct_from` for either shape.
  Every unit a formula can write is linear today; the affine temperature charts
  exist as tested infrastructure outside the registry. Registered today:
  `rad deg turn` (angle), `mm cm m km inch ft` (length), `ms s min h` (time),
  `g kg` (mass), `A mA`, `K`, `cd`, `mol`, and the derived
  `Hz N Pa kPa W V mV lx`. Inch is spelled `inch` because `in` is the membership
  keyword. °C and °F are affine, not linear, and are not registered (ISS-0004).
  A literal `n u` elaborates to the kernel literal `n × scale(u)` of the unit's
  dimension — `withUnit` — and nothing about a unit reaches `Ty`, `Value` or the
  runtime.
- There is no expression-level unit cast (`(x) deg`), and none is planned.
- One lookahead exception, for recovery only: a name followed by `=>` or `(` is
  never taken as a unit, because nothing but an operator may follow a unit. So a
  missing comma between match arms (`Off => 0` newline `Automatic => …`) is
  reported at `Automatic` instead of reading `0 Automatic` as a unit literal.
  Both readings are errors; this one keeps the next arm.

---

## 7. Comments, whitespace, layout

Whitespace and comments separate tokens and mean nothing else. There is no
layout rule: the indentation in this document is the canonical _formatting_
(§10), not grammar. Consequences that differ from Haskell:

- A definition may be on the same line as its signature:
  `mapping id : A -> A id(x) = x` is valid (and reformatted).
- A `match` arm's `=>` expression stops at `,` or `}`, not at a newline.
  `Off => 0 Automatic => 1` (missing comma) is an error at `Automatic`.
- A block's final expression is the one not followed by `;`, and it is
  mandatory: `{ let x = 1; }` is an error (_this block has no final expression
  to be its value_).

---

## 8. Ambiguity decisions

### 8.1 Bare identifier in a pattern

`None` and `x` are both `Ident`. The parser builds an `IdentPattern` for any
bare identifier and a `ConstructorPattern` only for `Ident ( … )`. Whether an
`IdentPattern` binds a variable or names a nullary constructor is decided by
name resolution (as in Rust), not by the parser. The capitalisation convention
(§2.5) was considered as a lexical rule (as in Haskell / MoonBit) and left for
revisiting (§13): it would make patterns syntactically unambiguous at the cost
of forbidding `Tilt` as a parameter name, which designers may well type.

### 8.2 The definition name repeats the declaration name

`mapping f : A -> B` followed by `g(x) = …` parses (the `MappingDef` attaches to
the preceding `MappingDecl`); lowering reports _the definition is named `g`, but
the mapping above is `f`_. Free-standing definitions — Haskell's separated
signature and equations — are not items in v0.1. A `MappingDef` with no
preceding `MappingDecl` (e.g. at the top of a file) is therefore a syntax error:
_expected `concept`, `mapping` or `enum`_.

### 8.3 `<` in types

In type position `<` and `>` are generic brackets; in expression position they
are comparisons. The two positions never overlap (a type appears only after `:`,
`->`, inside `TypeArgList`/`TypeList`), so there is no disambiguation and no
`>>` splitting.

### 8.4 Unary minus and literals

`-1` is `UnaryExpr(Minus, LiteralExpr(1))` in expressions — the lexer never
produces negative numbers — and `LiteralPattern(Minus, Number)` in patterns.

### 8.5 `if` as a primary expression

`if` is an expression; `else` is mandatory; there is no statement form.
`a + if c then 1 else 2 * 3` is `a + (if c then 1 else (2 * 3))`.

### 8.6 Number followed by a keyword or punctuation

`1 then`, `1,`, `1;`, `1)` — a unit suffix is only an `Ident` token, and
keywords are not identifiers, so no keyword is ever read as a unit.

### 8.7 `(` starts a parenthesis, a grouped value or a rule

After `(`: an expression followed by `)` is a parenthesis, followed by `,` a
grouped value. A rule's parameter list is recognised by lookahead — `(` Name
(`,` Name)\* `,`? `)` `=>` — before any expression is parsed, so `(a, b)` and
`(a, b) => a` never compete; `x => e` is recognised by `Ident` `=>` at the start
of an expression.

---

## 9. Malformed input, recovery, diagnostics

Parsing is total: any byte sequence yields a tree whose text is the input, plus
zero or more diagnostics. It never panics and never fails to make progress.
Diagnostics are structured
(`SyntaxError { code, span, message, expected, found, hint }`) and worded for
the designer first; the token vocabulary goes into `expected`/`found`.

### 9.1 Recovery anchors

| context                                              | anchors (the parser resynchronises here without consuming) |
| ---------------------------------------------------- | ---------------------------------------------------------- |
| top level                                            | `concept` `mapping` `enum` EOF                             |
| match arms                                           | `,` `}` and any token that can start a pattern             |
| block                                                | `;` `}` `let`                                              |
| parameter / argument / pattern / type-argument lists | `,` `)` (`>` for type arguments)                           |
| types                                                | `->` `,` `)` `>` `=` and any item keyword                  |

Everything skipped between an error and its anchor is wrapped in an `ErrorNode`
so the tree remains lossless. Inside an expression, `Error` _tokens_ (characters
the lexer rejected, already reported) are skipped silently, so `a § b` reports
the character once and then `b` as an unexpected name. After a definition body,
any token before the next item keyword is reported once (_expected an operator
or the next declaration after this definition_) and skipped to that keyword; a
type list stops without consuming at `Ident (`, which can only be the next
definition. Each recovery either consumes at least one token or stops at an
anchor the enclosing rule handles; loops that find nothing to consume terminate.

### 9.2 Codes and messages

| code                          | example message                                                                                                                                      |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `syntax.expected`             | expected `)` to close this call · expected an expression after `=` · expected `,` or `}` after this match arm · expected `->` or the end of the type |
| `syntax.unexpected`           | expected an operator or the end of the formula here · expected `concept`, `mapping` or `enum`                                                        |
| `syntax.chained_comparison`   | comparisons cannot be chained; write `a < b && b < c`                                                                                                |
| `syntax.reserved_word`        | `context` is reserved for a future version of BDL                                                                                                    |
| `syntax.invalid_character`    | `§` cannot appear in BDL source                                                                                                                      |
| `syntax.unterminated_comment` | this `/*` comment is never closed                                                                                                                    |
| `syntax.literal_pattern`      | only whole numbers can be matched, not `1.5`                                                                                                         |
| `syntax.empty`                | the formula is empty                                                                                                                                 |

Lowering (§11) adds two structural checks with the same error type, only on
mappings that parsed cleanly: _this definition is named `g`, but the mapping
declared above it is `f`_ and _`h` reads 2 inputs by its signature, but its
definition names 1_ (both `syntax.unexpected`).

### 9.3 Worked example

```bdl
concept Tilt : Angle

mapping broken : Tilt -> Brightness
broken(tilt =
  if tilt < 10 deg < 20 deg then
    foo(tilt
  else
    0

concept Later : Scalar
```

produces `concept Tilt`, a `mapping broken` whose definition has a parameter
list missing `)`, a chained comparison, and a call missing `)`, and then
`concept Later` — three diagnostics, three declarations, one tree
(`crates/bdl-syntax/test_data/invalid/broken_lamp.snap`).

---

## 10. Canonical formatting

Not implemented in this milestone; recorded so the tree and the grammar leave a
formatter no decisions to invent.

- Indentation is two spaces. Tabs are reformatted to spaces.
- One blank line between top-level items; comments keep their position relative
  to the item they precede.
- `concept Name : Type` and `mapping name : Type` on one line, one space around
  `:` and `->`.
- A mapping definition starts on the line after its signature; the body starts
  on the next line, indented — unless it is a single short expression that fits,
  in which case it may follow `=` on the same line.

  ```bdl
  mapping dimByTilt : Tilt -> Brightness
  dimByTilt(tilt) =
    clamp(tilt / (90 deg), 0, 1)
  ```

- `if c then` / body / `else` / body on four lines when any part is not a single
  token; otherwise on one line.
- `match` arms one per line, each ending with `,` — the trailing comma is
  **required by the formatter** even on the last arm.

  ```bdl
  match value {
    Some(x) => x,
    None => 0,
  }
  ```

- Blocks: `{`, one `let` per line, the tail expression on its own line, `}`.

  ```bdl
  {
    let normalized = tilt / (90 deg);
    let bounded = clamp(normalized, 0, 1);
    bounded
  }
  ```

- Enum variants one per line with trailing commas.
- Calls `f(a, b)` — no space inside parentheses, one after each comma. Binary
  operators are surrounded by single spaces; unary operators touch their
  operand; a unit follows its number with one space.
- Parentheses the author wrote are kept (`ParenExpr` is in the tree).
- A signature's spelling is kept: `mapping f : B` is not rewritten to
  `mapping f : () -> B` by formatting (§4.1); that is the migration's job, on
  request.

---

## 11. Relationship to the semantic model

```text
source text
  │  Logos
  ▼
tokens (with trivia)
  │  event parser → sink
  ▼
Rowan CST (lossless)
  │  bdl_syntax::ast (typed views)
  ▼
typed AST
  │  bdl_syntax::lower
  ▼
lowered surface tree (names, spans, exact literals; no ids)
  │  bdl-elab (name resolution, rep/mk, units, typing)
  ▼
Core IR
```

| textual construct                                                                                                                                                                                                                                                                                                                       | surface model                             | kernel                                                                                                                                                          |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `concept C : R` — `R` is `Bool`, `Count`, or a quantity name from the shared vocabulary (`bdl_model::quantity`: `Scalar`, `Angle`, `Length`, `Time`, `Mass`, `Current`, `Temperature`, `Amount`, `Luminous`, `Speed`, `Acceleration`, `AngularVelocity`, `Frequency`, `Force`, `Pressure`, `Torque`, `Power`, `Voltage`, `Illuminance`) | `Concept { name, representation }`        | `Θ C = some R`                                                                                                                                                  |
| `concept C`                                                                                                                                                                                                                                                                                                                             | `Concept { representation: None }`        | `Θ C = none` (open)                                                                                                                                             |
| `mapping f : A -> B -> C`, `mapping f : (A, B) -> C`                                                                                                                                                                                                                                                                                    | `MappingBlock { signature: (A, B) -> C }` | `DesignDecl` with `expectedType = sem A → sem B → sem C` (the canonical `(A, B) -> C`, curried)                                                                 |
| `mapping f : B`, `mapping f : () -> B`                                                                                                                                                                                                                                                                                                  | `MappingBlock { signature: () -> B }`     | `DesignDecl` with `expectedType = sem B` (the canonical `() -> B`, the unit eliminated — ADR-0029)                                                              |
| no definition                                                                                                                                                                                                                                                                                                                           | `definition: None`                        | unresolved declaration — a legal state                                                                                                                          |
| `f(a, b) = e`                                                                                                                                                                                                                                                                                                                           | `Definition::Formula`                     | realization `λa.λb. mk C (…)`                                                                                                                                   |
| `90 deg`                                                                                                                                                                                                                                                                                                                                | number `90` with unit `deg`               | scaled dimensioned literal (linear units only, DI-7)                                                                                                            |
| `f(x)`                                                                                                                                                                                                                                                                                                                                  | call of a relationship                    | `app (declRef f) x`; arguments are semantic values (an input, a relationship's value), never bare numbers                                                       |
| `level`, `level()`, `level(())` (relationship without inputs, `() -> B`)                                                                                                                                                                                                                                                                | reference                                 | `declRef level` — the application to the unique argument, erased                                                                                                |
| `{ let x = v; e }`                                                                                                                                                                                                                                                                                                                      | block                                     | `app (λx:τ. e) v` — a beta-redex; lowering makes it a `Let`                                                                                                     |
| `if c then a else b`                                                                                                                                                                                                                                                                                                                    | conditional                               | `ite c a b` (strict: both branches are evaluated, DI-26)                                                                                                        |
| `Some(e)`, `None`                                                                                                                                                                                                                                                                                                                       | option constructors                       | `some e`, `none`; the payload is a representation value                                                                                                         |
| `match s { Some(x) => a, None => b }`                                                                                                                                                                                                                                                                                                   | match                                     | `app (λs. ite (isSome s) (app (λx. a) (getD s d)) b) s` — the scrutinee is bound once, tests are `isSome` / `eq` / the Boolean, bindings are `getD` projections |
| `delay(init, v)`, `sync(domain, init, v)`                                                                                                                                                                                                                                                                                               | memory                                    | `delay init v`, `sync domain init v` — legal only outside every binder                                                                                          |
| `[a, b]`, `(a, b)`                                                                                                                                                                                                                                                                                                                      | a collection, a grouped value             | `cons a (cons b nil)`, `pair a b` (§15)                                                                                                                         |
| `min(a, b)`, `any(xs, x => p)`, `x in xs`, …                                                                                                                                                                                                                                                                                            | an equation of the library                | the library's closed combinator applied to the arguments, its scheme matched against their kinds (`docs/spec/equation-library.md`)                              |
| `ordered concept C : Scalar`; `concept C : List<R>` / `Pair<R₁, R₂>` / `Option<R>`                                                                                                                                                                                                                                                      | `Concept { ordered, representation }`     | `OrdDecl C`; `Θ C = list R` / `R₁ × R₂` / `opt R`                                                                                                               |
| `enum`, constructor patterns other than `Some`/`None`                                                                                                                                                                                                                                                                                   | not in the surface model                  | not in the kernel (no sum types) — syntax only; `formula.constructor.unknown` (DI-19)                                                                           |

### 11.1 Implementation support matrix

| construct                                                                                            | syntactically accepted | semantically elaborated                                                                                                                | backend executable                                  |
| ---------------------------------------------------------------------------------------------------- | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| names, literals, units, `+ - * /`, comparisons, `&& \|\| !`                                          | yes                    | yes                                                                                                                                    | yes                                                 |
| `if … then … else`                                                                                   | yes                    | yes                                                                                                                                    | yes (strict)                                        |
| `f(a, …)` with semantic arguments; `level`                                                           | yes                    | yes (`formula.call.*` diagnostics)                                                                                                     | yes — inlined, no closures                          |
| `{ let x = …; e }`, nested, shadowing                                                                | yes                    | yes (lexical; a `let` may shadow an input or an outer `let`)                                                                           | yes (`Let`)                                         |
| `Some(e)` / `None`                                                                                   | yes                    | yes (`None` takes its payload type from a sibling branch or arm; `formula.option.undetermined` otherwise)                              | yes                                                 |
| `match` on Bool, Option (nested), dimensionless numbers, a semantic value through its representation | yes                    | yes, with conservative exhaustiveness (`formula.match.non_exhaustive`) and unreachable-arm warnings                                    | yes                                                 |
| `_`, name, `true`/`false`, whole-number, `Some(p)`, `None` patterns                                  | yes                    | yes; duplicate binders refused                                                                                                         | yes                                                 |
| `delay` / `sync` in a `let` value, a scrutinee, an `if` branch                                       | yes                    | yes                                                                                                                                    | yes — one state cell per written form               |
| `delay` / `sync` in a block's result, an arm, a formula with inputs                                  | yes                    | no — `formula.temporal.under_binder` / `under_inputs`                                                                                  | —                                                   |
| `enum` items, other constructors                                                                     | yes (CST, lowering)    | no — `formula.constructor.unknown` (DI-19)                                                                                             | —                                                   |
| a relationship as a value, `f(x)(y)`, calling an input                                               | yes                    | no — `formula.mapping.needs_arguments`, `formula.call.not_a_relationship` (the higher-order boundary, DI-24, is closed at the surface) | —                                                   |
| `[a, b]`, `(a, b)`, `x in xs`, the equations, a rule `x => e` as an equation's argument              | yes                    | yes (`docs/spec/equation-library.md` §5 diagnostics; `formula.rule.not_a_value` for a rule anywhere else)                              | yes — collections need an allocator (`collections`) |
| `==` on truth values, counts, optional values, collections, grouped values, concept values           | yes                    | yes (structural; across concepts `semantic.concept_mismatch`)                                                                          | yes                                                 |
| `<` / `min` on concept values                                                                        | yes                    | only for `ordered concept`s represented by a quantity (`semantic.no_order` otherwise)                                                  | yes                                                 |
| number patterns on counts                                                                            | yes                    | yes (a whole, non-negative number)                                                                                                     | yes                                                 |

The formula field in Studio and a `.bdl` file's definition bodies go through the
same elaborator (`bdl-elab::formula`), so this matrix holds for both. A textual
definition's parameter names are lexical bindings stored in
`MappingBlock.parameters` (§14.4); a relationship authored without parameter
names falls back to concept display-name resolution.

Rules the syntax layer keeps out of itself: `SemanticId`, `DeclId`, grants,
dimensions, clocks, causality, hardware. The tree records what was written;
`bdl-elab` and `bdl-check` decide what it means. The formula field in Studio
parses with the expression entry point of the **same** parser (`Formula` root),
so there is one expression grammar. Formula syntax errors reach Studio as
`formula.parse.unexpected_token`, one per error, with the `syntax.*` code in the
technical line and the hint as a fix.

Exact numerics: the lowered tree carries `NumberLiteral { text }` with
`decimal()` giving `digits × 10^exponent` exactly. The only place an `f64`
appears is `bdl-elab`'s literal emission, marked as debt against the planned
symbolic layer
(`typed expr → SemFree subset → symbolic IR → transformation → reification into Core → re-check`).
`simplify(expr)` and `differentiate(expr, x)` will be ordinary calls; no CAS
grammar is added. See ADR-0014 for why this stack (Logos, hand-written event
parser, Rowan) was chosen, and DI-18 / DI-19 for the numeric boundary and the
constructs that parse ahead of the kernel.

---

## 12. Future-reserved syntax

Reserved words (§2.1) still without a production: `context`, `require`. Reserved
shapes:

- `context Name when Expr { Item* }` — behavioural contexts.
- temporal phrases `previous`, `hold`, `count`, `rise`, `every` — an open design
  issue (ISS-0010); if adopted they would be ordinary calls
  (`previous(x, init)`), so no grammar change is needed.
- `require` — commitments on a declaration.

Reserved punctuation with no token yet: `#`, `|`, `&`, `..`, `::`. They lex as
`Error` so that no program depends on them. `@` and `.` became tokens in v0.2
(§14).

---

## 13. Decisions to revisit

1. Lexical capitalisation for constructors vs. bindings in patterns (§8.1);
   currently name-resolution decides.
2. Free-standing `MappingDef` items (§8.2); currently a definition must follow
   its signature.
3. Nested block comments (§2.4).
4. Whether the future-reserved words should be reserved in identifier position
   at all (`output` is a tempting parameter name).
5. Whether `f(x)(y)` should be a parse error rather than a typing error.
6. A dedicated integer literal class in the lexer (today the parser checks the
   spelling in literal patterns).

---

## 14. Project items (v0.2)

v0.1 spelled concepts, relationships and enums; the sources must spell
everything the model holds (ADR-0020, ADR-0023). v0.2 adds the items below.
Every one lowers to an existing `bdl-model` / `bdl-system` structure; none adds
a semantic notion. The item keywords are real keywords; `for`, `pin`,
`realization`, `init`, `as`, `optional` are **contextual** — identifiers
everywhere else, recognised by spelling only where the grammar expects them.

### 14.1 Timing domains, physical outputs, drives, devices

```ebnf
Item          ::= ConceptDecl | MappingDecl | EnumDecl | ClockDecl | OutputDecl
                | DriveDecl | DeviceDecl | ComponentDecl | InstanceDecl
                | BindDecl | ExportDecl

ClockDecl     ::= "clock" Name
ClockTag      ::= "@" NameRef                      (* the domain a declaration updates in *)
MappingDecl   ::= "mapping" Name ":" Type ClockTag? MappingDef?
OutputDecl    ::= "output" Name ":" Type ClockTag? ("optional")?
DriveDecl     ::= "drive" NameRef "=" NameRef       (* output = relationship *)
DeviceDecl    ::= "device" Name ":" Ident ("for" NameRef)? DeviceBody?
DeviceBody    ::= "{" ( (PinFix | RealizationFix) ","? )* "}"
PinFix        ::= "pin" Number "=" Ident
RealizationFix::= "realization" Ident             (* the output realization profile *)
```

```bdl
clock interaction

mapping tilt : () -> Tilt @interaction           // a value, supplied from outside
mapping brightness : () -> Brightness @interaction
brightness() = dimByTilt(tilt)

output light : Brightness @interaction            // a physical output, required by default
output indicator : Brightness @interaction optional
drive light = brightness                          // the single drive edge

device pwmLight : pwm_channel for light { realization pwm_duty8, pin 0 = D3 }
```

- `output` is a **physical output**
  (`PhysicalOutput { accepts, clock, required }`), never a relationship's
  produced concept and never a provided port — those are `mapping … : T` and
  `provides` (§14.2). The clock tag is optional (an output without one is
  _open_).
- `drive o = m` is `MappingBlock.drives = Some(o)`; a relationship drives at
  most one output and an output is driven by at most one relationship, as the
  model already requires. A second `drive` for the same output is a lowering
  fault, not a merge.
- A device's kind is a `DeviceKind` name in snake case (`pwm_channel`,
  `digital_output`, `h_bridge_channel`, `i2c_sensor`, `quadrature_encoder`,
  `uart`); `for` names the output it realises (absent for a sensor); pins are
  fixed by the device's requirement index and a board-relative pin name, which
  is an identifier (`D3`, `A4`, `GP15`).
- `realization <id>` names the **output realization profile** the device uses
  (`DeviceBinding.realization`, ADR-0036): a stable identifier from the
  compiler's registry (`pwm_duty8`, `pwm_duty4`, `i2c_level8`, `gpio_level`,
  `hbridge_signed`), written at most once per body — the last one wins — in any
  order with the pins; the printer writes it first. The profile prescribes the
  kind; a body that names a profile of another kind still parses and is
  diagnosed at deployment (`deploy.realization_kind_mismatch`), as is an id the
  registry does not know. A body without one (every file written before profiles
  existed) leaves the device placing by kind and generating no raw command
  (`docs/architecture/output-realization.md`).

### 14.2 Components

```ebnf
ComponentDecl ::= "component" Name "{" ComponentItem* "}"
ComponentItem ::= ConceptDecl | UseDecl | ClockDecl | ParamClockDecl
                | MappingDecl | EnumDecl | PortDecl | OutputDecl | DriveDecl | DeviceDecl
UseDecl       ::= "use" ("concept" | "output") NameRef
ParamClockDecl::= "param" "clock" Name
PortDecl      ::= ("requires" | "provides" | "param") Name ":" Type ClockTag? MappingDef?
```

```bdl
component AdaptiveLamp {
  use concept Tilt                 // the system's Tilt, shared
  use concept Brightness
  use concept Gain
  param clock main                 // a timing parameter: each instance names a system domain
  clock blink                      // a private domain of the component

  requires tiltValue : Tilt @main             // what the behavior needs
  param gain : Gain                           // configured per instance (a concept, like any value)

  mapping dimByTilt : Tilt -> Brightness      // component-local
  dimByTilt(t) = t / (90 deg)

  provides brightness : Brightness @main      // what the behavior offers
  brightness() = dimByTilt(tiltValue) * gain
}
```

- The body is an ordinary flat design in the component's own names: `concept`
  declares a **private** concept; `use concept X` declares a body concept named
  `X` that _is_ the system concept `X` (`shared_concepts[local] = system`).
  `use output X` does the same for a physical output (`external_outputs`).
  Nothing is shared by name coincidence; only `use` shares.
- `clock c` is a private domain; `param clock c` is a **timing parameter**
  (`interface.clock_params`). A `@c` tag on a port makes the port's clock
  contract `Private { c }` or `Parameter { c }` accordingly; no tag is
  `Agnostic`.
- `requires n : T` is a nullary relationship `n` of the body **without a
  definition** plus a required port whose contract is snapshotted from the
  declaration; `provides n : T` is a nullary relationship with a definition
  (which must follow, like any `MappingDef`) plus a provided port; `param n : T`
  is a nullary relationship without a definition plus a parameter port. A
  `requires` with a definition or a `provides` without one is a lowering fault —
  the contract stays what the text says, so the body's failure to keep it is the
  ordinary `component.*` finding.
- The public contract is exactly the port declarations. A body `mapping` is
  never exported by inference.

### 14.3 Instances, bindings, exports

```ebnf
InstanceDecl  ::= "instance" Name ":" NameRef InstanceBody?
InstanceBody  ::= "{" ( InstanceArg ","? )* "}"
InstanceArg   ::= NameRef "=" Expr           (* a clock parameter = a system clock name; a parameter port = a closed expression *)
BindDecl      ::= "bind" BindEnd "=" BindEnd ("init" Expr)?
BindEnd       ::= NameRef ("." NameRef)?     (* instance.port, or a top-level relationship *)
ExportDecl    ::= "export" BindEnd "as" Name
```

```bdl
instance lampA : AdaptiveLamp { main = interaction, gain = 2 }
instance lampB : AdaptiveLamp { main = interaction, gain = 1 }

bind lampA.tiltValue = tiltValue         // required port ← top-level value
bind lampB.tiltValue = tiltValue         // fan-out: the same value, twice
bind mirror = lampB.brightness           // open top-level relationship ← provided port
bind slow = lampA.brightness init 0      // across domains: carried, starting at 0

export lampB.tiltValue as tiltIn         // a required port left open, named at the system boundary
```

- `bind destination = source`: the destination is a required port or a parameter
  port (`instance.port`) or an open top-level relationship; the source is a
  provided port or a top-level relationship. This is
  `SystemEditOp::BindPorts { source, destination, transport }`; `init e` is
  `BindingTransport { init: "e" }` — the transport is explicit or absent, never
  inferred.
- An instance argument whose name is a timing parameter takes a system clock
  name; one whose name is a parameter port takes a closed expression
  (`ParameterValue`). Anything else is a lowering fault.
- `export instance.port as name` is `Export { port, name }`.

### 14.4 Parameters bind lexically

`dimByTilt(t) = t / (90 deg)`: the parameter names of a definition are the names
of the inputs _in that body_, positionally, stored on the relationship
(`MappingBlock.parameters`). They shadow nothing outside the body and are not
concept names: `t` resolves to the first input whatever the concept is called. A
body may still write the concept's name when the relationship has no parameter
names (a relationship authored in Studio), which keeps the v0.1 rule as the
fallback only.

### 14.5 Keys, not ids

Nothing in source names a stable identity. The loader keys every item by kind
and qualified name (`component:AdaptiveLamp/port:brightness`) and reconciles
keys against `.bdl/identities.json` (ADR-0020 §3–4). Groups and layout are not
in source.

### 14.7 Descriptions are doc comments

A run of `///` lines directly above an item (no blank line between) is the
item's description — the _Meaning_ field of Studio's inspector — and is the only
comment a tool rewrites: write-back prints a changed description as `///` lines
and leaves every `//` and `/* */` comment where it was.

```bdl
/// How far the lamp head is tilted from upright.
concept Tilt : Angle
```

### 14.6 Canonical formatting of the new items

One item per line unless it has a body; bodies are `{` on the item's line,
members indented two spaces, `}` on its own line; instance arguments on one line
separated by `,`; a port's definition follows on the next line like a mapping's.
A component's items are printed in the order concepts, `use`, clocks, ports,
mappings, outputs, drives, devices when a tool renders them; a formatter never
reorders what a person wrote.

---

## 15. Collections, grouped values, rules, membership, ordered concepts

The equation language (`docs/spec/equation-library.md`) adds five forms; all
lower to the existing Core.

- **Collection literal** `[a, b, c]`, `[]` — `ListExpr`. Elements are brought to
  one kind like the branches of an `if`; `[]` takes its element kind from the
  context or a sibling, like `None`.
- **Grouped value** `(a, b)` — `TupleExpr`; three or more parts nest to the
  right, `(a, b, c)` = `(a, (b, c))`. `(a)` stays a parenthesis. Parts keep
  their own kinds; `first(p)` and `second(p)` take them apart. There is no
  pattern for a grouped value.
- **Rule** `x => e`, `(x, acc) => e` — `LambdaExpr`; only ever an argument to an
  equation (`any`, `all`, `map`, `filter`, `foldr`, `minBy`, `maxBy`, `optElim`,
  `mapOpt`). Its parameters are lexical bindings that shadow outer names; it may
  not hold `delay`/`sync`. Written anywhere else it is
  `formula.rule.not_a_value`.
- **Membership** `x in xs` — a comparison-level binary operator (it does not
  chain, §5.1) that is the equation `contains(x, xs)`.
- **Ordered concept** `ordered concept Brightness : Scalar` — the designer
  declares that two values of the concept can be put in order; only a quantity
  value form carries it: `ordered concept Mode : Count` is a load fault
  (`text.order_needs_quantity`) and the concept is loaded unordered. Every other
  concept's values compare for equality only. `ordered` is a keyword.
- **Value forms** in `concept C : …`: `List<R>`, `Pair<R₁, R₂>`, `Option<R>`
  over value forms (`Bool`, `Count`, a quantity name, or one of these);
  `text.unknown_representation` names the shape otherwise.

Canonical formatting: no space inside `[` `]`; `(a, b)` as a call's arguments;
`x => e` with spaces around `=>`; `in` spaced like a comparison.

## 16. Slots

```ebnf
SlotExpr ::= "?"
```

A **slot** is an expression not yet written: `tilt / ?`, `clamp(?, 0, 1)`. It
parses wherever a value may stand and formats as itself. It is the Formula
Composer's hole (docs/architecture/studio-ui.md §4b, FV Phase 10
`Composer.lean`'s `PExpr.hole`) and belongs to authoring: elaboration refuses it
with `formula.slot.empty` — _This slot is empty; it expects an angle._ when the
context says what it expects — so a formula with a slot is a definition that
does not check, like any other draft that is not finished, and no slot ever
reaches Core, the checker, the evaluator or generated code. A slot may be typed
as text and saved (an incomplete definition is a legal state of a design); it is
never a value, an operator or a unit (`1 ? 2` is a syntax error).

## 17. Natural forms: binders, ranges, defaults

```ebnf
BinderExpr ::= ( "all" | "any" | "map" | "filter" ) Name "in" RangeExpr ":" Expr
RangeExpr  ::= CoalesceExpr ".." CoalesceExpr
CoalesceExpr ::= AddExpr "??" CoalesceExpr
```

Three surface forms over the equation language of §15 — a way of writing
`all(xs, x => body)`, `inRange(x, lo, hi)` and `getOrElse(x, d)` in the order a
designer says them. Each is **desugared once, by the elaborator**, to the
library equation it names applied to the same arguments; the parser keeps the
natural form as its own node, the formatter keeps whichever spelling was
written, and nothing new reaches Core, the checker, the evaluator or generated
code (FV Phase 11, `BDL/Surface/Natural.lean`: `desugar` is one-way; typing,
evaluation and clocks are the library's — `binder_*_typed`, `binder_*_eval`,
`range_typed`, `range_eval`, `binder_clock`, `range_clock`, `coalesce_typed`).

- **Binder** `all reading in readings: reading < limit` — `BinderExpr`. The
  first word is a **contextual keyword**: `all`, `any`, `map`, `filter` are
  binders only in the head position `word Name in …`; anywhere else they are
  ordinary names, so a relationship called `map` stays callable (`map(x)`,
  `all(x, y)`, `all + 1`, `all in xs` are what they were). The local `reading`
  is the rule's parameter (`readings`, `reading => reading < limit`): visible in
  the body only, one element of the collection (`binder_local_type`), shadowing
  an outer name of the same spelling lexically, and shadowed by an inner binder
  (`all x in xs: any x in ys: x` reads `ys`'s `x`); it is never a name of the
  design, so references, rename and semantic tokens treat it as a parameter. The
  collection is an expression short of the colon (a comparison, a range or
  another binder there needs parentheses); the body is the rest, like a rule's.
  `all`/`any`/`filter` need a `true`/`false` body; `map` makes a collection of
  what the body gives. There is no comprehension language: no generators,
  `yield`, `where` or unbounded quantifiers — nested binders cover those cases.
- **Range** `x in lo .. hi` — `RangeExpr` as the right operand of `in`, the
  closed range `inRange(x, lo, hi)` (both ends belong). `..` binds tighter than
  `in` and weaker than `+ -`, and does not chain (`1 .. 2 .. 3` is
  `syntax.chained_comparison`). A range is not a value: written anywhere but
  after `in` it is `formula.range.outside_in`. Both ends must be comparable with
  the subject — the same concept when the subject is a concept value, its
  dimension otherwise, under the ordering policy of §15 (`Mode` values against
  `Mode` values have no order). The lexer keeps numbers whole: `1.0..2.0` is
  `1.0` `..` `2.0`, `1..2` is `1` `..` `2`; there is no unary `+`.
- **Default** `x ?? d` — `getOrElse(x, d)`: `x` when present, `d` when absent.
  Right-associative, between `..` and `+ -`.

Diagnostics speak of the form, not of the equation it becomes:
`formula.binder.not_a_collection` — _all expects a collection after 'in'._;
`formula.binder.body` — _The body of 'filter' must be true or false._;
`formula.range.endpoint` — _This range endpoint must be an angle._ (explanation
_Both ends of the range must be comparable with angle._); `semantic.no_order`
for an unordered concept; `formula.coalesce.not_optional` /
`formula.coalesce.default`; a local used outside its body is
`formula.name.unknown`, whose fix lists the locals in scope where there are any.

Canonical formatting: one space around `in`, `..` and `??`; no space before the
binder's colon, one after: `all reading in readings: reading < limit`. Nothing
is indentation-sensitive. The call forms of §15 are never rewritten to these
forms, nor the reverse.
