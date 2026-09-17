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

**Future-reserved** — lexed as keyword tokens today so that no v0.1 program uses
them as names, but with no grammar production. Writing one where a name is
expected reports _`context` is reserved for a future version of BDL_:

`context` · `output` · `clock` · `component` · `require`

### 2.2 Punctuation and operators

Longest match wins; `->`, `=>`, `==`, `!=`, `<=`, `>=`, `&&`, `||` are single
tokens, so `a->b` lexes as `a` `->` `b` and `a- >b` as `a` `-` `>` `b`.

| token       | kind                                          | token           | kind                          |
| ----------- | --------------------------------------------- | --------------- | ----------------------------- |
| `(` `)`     | `LParen` `RParen`                             | `+` `-` `*` `/` | `Plus` `Minus` `Star` `Slash` |
| `{` `}`     | `LBrace` `RBrace`                             | `!`             | `Bang`                        |
| `<` `>`     | `Lt` `Gt` (comparison _and_ generic brackets) | `<=` `>=`       | `Le` `Ge`                     |
| `:` `,` `;` | `Colon` `Comma` `Semi`                        | `==` `!=`       | `EqEq` `Ne`                   |
| `=`         | `Eq`                                          | `&&` `\|\|`     | `AndAnd` `OrOr`               |
| `->`        | `Arrow`                                       | `=>`            | `FatArrow`                    |

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

ConceptDecl     ::= "concept" Name ( ":" Type )?

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
lowering reports a mismatch (§8.2). A nullary mapping is declared
`mapping level : Brightness` and defined `level() = 0`.

### 4.2 Types

```ebnf
Type            ::= TypeAtom ( "->" Type )?              (* "->" is right-associative *)
TypeAtom        ::= NamedType | ParenType
NamedType       ::= NameRef TypeArgList?
TypeArgList     ::= "<" Type ( "," Type )* ","? ">"
ParenType       ::= "(" Type ")"
```

`A -> B -> C` is `A -> (B -> C)`; `(A -> B) -> C` needs the parentheses. A
concept is written by its name (`Tilt`), never `Sem<Tilt>` — `sem`, `q`, `rep`,
`mk` are kernel and explanation vocabulary. Generic forms such as
`Option<Brightness>` and `Result<Value, Error>` are syntax only; which of them
exist and what they mean is decided by elaboration. There is no type-level
computation.

### 4.3 Expressions

```ebnf
Expr            ::= OrExpr

OrExpr          ::= AndExpr ( "||" AndExpr )*
AndExpr         ::= EqExpr ( "&&" EqExpr )*
EqExpr          ::= RelExpr ( ( "==" | "!=" ) RelExpr )?          (* non-associative *)
RelExpr         ::= AddExpr ( ( "<" | "<=" | ">" | ">=" ) AddExpr )?  (* non-associative *)
AddExpr         ::= MulExpr ( ( "+" | "-" ) MulExpr )*
MulExpr         ::= UnaryExpr ( ( "*" | "/" ) UnaryExpr )*
UnaryExpr       ::= ( "!" | "-" ) UnaryExpr | PostfixExpr
PostfixExpr     ::= PrimaryExpr CallSuffix*
CallSuffix      ::= "(" ArgList? ")"
ArgList         ::= Expr ( "," Expr )* ","?

PrimaryExpr     ::= NameExpr | LiteralExpr | ParenExpr | IfExpr | MatchExpr | BlockExpr
NameExpr        ::= NameRef
LiteralExpr     ::= Number UnitSuffix? | "true" | "false"
UnitSuffix      ::= Ident
ParenExpr       ::= "(" Expr ")"
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
| 4     | `<` `<=` `>` `>=`                       | **none**      | `BinaryExpr` |
| 5     | `+` `-`                                 | left          | `BinaryExpr` |
| 6     | `*` `/`                                 | left          | `BinaryExpr` |
| 7     | prefix `!` `-`                          | —             | `UnaryExpr`  |
| 8     | call `f(…)`                             | postfix, left | `CallExpr`   |
| 9     | `if` `match` `{ }` `( )` literals names | primary       | —            |

Pratt binding powers used by the parser: `||` 1/2, `&&` 3/4, `==` `!=` 5/6, `<`…
7/8, `+` `-` 9/10, `*` `/` 11/12, unary 13, call 15.

Reading rules:

- `-x * y` is `(-x) * y`; `-f(x)` is `-(f(x))`; `!a && b` is `(!a) && b`.
- `f(x)(y)` parses as `CallExpr(CallExpr(f, x), y)`. Typing may reject it; the
  syntax admits it so that the grammar has no special case.
- `if`, `match` and blocks are primary expressions whose branch/arm/tail
  expressions extend as far right as possible: `if c then a else b + 1` is
  `if c then a else (b + 1)`, and `1 + if c then a else b` is legal.
- A unit suffix binds to its number only: `90 deg / 2` is `(90 deg) / 2`.

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
| `mapping f : A -> B -> C`                                                                                                                                                                                                                                                                                                               | `MappingBlock { signature: (A, B) -> C }` | `DesignDecl` with `expectedType = sem A → sem B → sem C`                                                                                                        |
| no definition                                                                                                                                                                                                                                                                                                                           | `definition: None`                        | unresolved declaration — a legal state                                                                                                                          |
| `f(a, b) = e`                                                                                                                                                                                                                                                                                                                           | `Definition::Formula`                     | realization `λa.λb. mk C (…)`                                                                                                                                   |
| `90 deg`                                                                                                                                                                                                                                                                                                                                | number `90` with unit `deg`               | scaled dimensioned literal (linear units only, DI-7)                                                                                                            |
| `f(x)`                                                                                                                                                                                                                                                                                                                                  | call of a relationship                    | `app (declRef f) x`; arguments are semantic values (an input, a relationship's value), never bare numbers                                                       |
| `level` (relationship without inputs)                                                                                                                                                                                                                                                                                                   | reference                                 | `declRef level`                                                                                                                                                 |
| `{ let x = v; e }`                                                                                                                                                                                                                                                                                                                      | block                                     | `app (λx:τ. e) v` — a beta-redex; lowering makes it a `Let`                                                                                                     |
| `if c then a else b`                                                                                                                                                                                                                                                                                                                    | conditional                               | `ite c a b` (strict: both branches are evaluated, DI-26)                                                                                                        |
| `Some(e)`, `None`                                                                                                                                                                                                                                                                                                                       | option constructors                       | `some e`, `none`; the payload is a representation value                                                                                                         |
| `match s { Some(x) => a, None => b }`                                                                                                                                                                                                                                                                                                   | match                                     | `app (λs. ite (isSome s) (app (λx. a) (getD s d)) b) s` — the scrutinee is bound once, tests are `isSome` / `eq` / the Boolean, bindings are `getD` projections |
| `delay(init, v)`, `sync(domain, init, v)`                                                                                                                                                                                                                                                                                               | memory                                    | `delay init v`, `sync domain init v` — legal only outside every binder                                                                                          |
| `enum`, constructor patterns other than `Some`/`None`                                                                                                                                                                                                                                                                                   | not in the surface model                  | not in the kernel (no sum types) — syntax only; `formula.constructor.unknown` (DI-19)                                                                           |

### 11.1 Implementation support matrix

| construct                                                                                            | syntactically accepted | semantically elaborated                                                                                                                | backend executable                    |
| ---------------------------------------------------------------------------------------------------- | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| names, literals, units, `+ - * /`, comparisons, `&& \|\| !`                                          | yes                    | yes                                                                                                                                    | yes                                   |
| `if … then … else`                                                                                   | yes                    | yes                                                                                                                                    | yes (strict)                          |
| `f(a, …)` with semantic arguments; `level`                                                           | yes                    | yes (`formula.call.*` diagnostics)                                                                                                     | yes — inlined, no closures            |
| `{ let x = …; e }`, nested, shadowing                                                                | yes                    | yes (lexical; a `let` may shadow an input or an outer `let`)                                                                           | yes (`Let`)                           |
| `Some(e)` / `None`                                                                                   | yes                    | yes (`None` takes its payload type from a sibling branch or arm; `formula.option.undetermined` otherwise)                              | yes                                   |
| `match` on Bool, Option (nested), dimensionless numbers, a semantic value through its representation | yes                    | yes, with conservative exhaustiveness (`formula.match.non_exhaustive`) and unreachable-arm warnings                                    | yes                                   |
| `_`, name, `true`/`false`, whole-number, `Some(p)`, `None` patterns                                  | yes                    | yes; duplicate binders refused                                                                                                         | yes                                   |
| number patterns on counts                                                                            | yes                    | no — `formula.unsupported` (no `nat` equality, DI-12)                                                                                  | —                                     |
| `delay` / `sync` in a `let` value, a scrutinee, an `if` branch                                       | yes                    | yes                                                                                                                                    | yes — one state cell per written form |
| `delay` / `sync` in a block's result, an arm, a formula with inputs                                  | yes                    | no — `formula.temporal.under_binder` / `under_inputs`                                                                                  | —                                     |
| `enum` items, other constructors                                                                     | yes (CST, lowering)    | no — `formula.constructor.unknown` (DI-19)                                                                                             | —                                     |
| a relationship as a value, `f(x)(y)`, calling an input                                               | yes                    | no — `formula.mapping.needs_arguments`, `formula.call.not_a_relationship` (the higher-order boundary, DI-24, is closed at the surface) | —                                     |

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
a semantic notion. The item keywords are real keywords; `for`, `pin`, `init`,
`as`, `optional` are **contextual** — identifiers everywhere else, recognised by
spelling only where the grammar expects them.

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
DeviceBody    ::= "{" ( PinFix ","? )* "}"
PinFix        ::= "pin" Number "=" Ident
```

```bdl
clock interaction

mapping tilt : Tilt @interaction                 // a value, supplied from outside
mapping brightness : Brightness @interaction
brightness() = dimByTilt(tilt)

output light : Brightness @interaction            // a physical output, required by default
output indicator : Brightness @interaction optional
drive light = brightness                          // the single drive edge

device pwmLight : pwm_channel for light { pin 0 = D3 }
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
