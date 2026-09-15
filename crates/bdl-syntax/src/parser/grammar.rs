//! Module and items (`docs/TEXTUAL_SYNTAX.md` §4.1) and the formula entry
//! point.

use super::{expr, pattern, types, Parser};
use crate::kind::SyntaxKind::{self, *};
use crate::syntax::SyntaxErrorCode;

const ITEM_START: &[SyntaxKind] = &[KwConcept, KwMapping, KwEnum];

/// `Module ::= Item* EOF`
pub(super) fn module(p: &mut Parser<'_>) {
    while !p.at_eof() {
        match p.current() {
            KwConcept => concept_decl(p),
            KwMapping => mapping_decl(p),
            KwEnum => enum_decl(p),
            _ => {
                let message = if p.at(Ident) && p.nth(1) == LParen {
                    format!(
                        "a definition of `{}` needs its `mapping` signature right above it",
                        p.current_text()
                    )
                } else {
                    "expected `concept`, `mapping` or `enum` here".to_owned()
                };
                p.error(SyntaxErrorCode::Unexpected, message);
                p.hint("Every top-level declaration starts with one of these keywords.");
                p.skip_until(ITEM_START);
            }
        }
    }
}

/// `Formula ::= Expr EOF`
pub(super) fn formula(p: &mut Parser<'_>) {
    if p.at_eof() {
        p.error(SyntaxErrorCode::Empty, "the formula is empty");
        return;
    }
    if expr::expr(p).is_none() {
        // Nothing usable at the start; say so and consume everything.
        p.error(SyntaxErrorCode::Unexpected, expr::no_expression_message(p));
        p.skip_until(&[]);
        return;
    }
    if !p.at_eof() {
        expr::unexpected_after_expr(p, "the end of the formula");
        p.skip_until(&[]);
    }
}

/// `ConceptDecl ::= "concept" Name (":" Type)?`
fn concept_decl(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // concept
    if !name(p, "expected the concept's name after `concept`") {
        recover_item(p);
        m.complete(p, ConceptDecl);
        return;
    }
    if p.eat(Colon) && !types::type_(p) {
        p.error_expecting(
            "expected what kind of value this concept is, after `:`",
            &[Ident],
        );
        p.hint("For example `concept Tilt : Angle`; leave the `:` off to declare an open concept.");
    }
    recover_item(p);
    m.complete(p, ConceptDecl);
}

/// `MappingDecl ::= "mapping" Name ":" Type MappingDef?`
fn mapping_decl(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // mapping
    if !name(p, "expected the mapping's name after `mapping`") {
        recover_item(p);
        m.complete(p, MappingDecl);
        return;
    }
    if !p.eat(Colon) {
        p.error_expecting(
            "expected `:` and the mapping's type after its name",
            &[Colon],
        );
        p.hint("A mapping is declared as `mapping name : Input -> Output`.");
    }
    if !types::type_(p) {
        p.error_expecting(
            "expected the mapping's type, such as `Tilt -> Brightness`",
            &[Ident, LParen],
        );
    }
    if p.at(Ident) && p.nth(1) == LParen {
        mapping_def(p);
    }
    recover_item(p);
    m.complete(p, MappingDecl);
}

/// `MappingDef ::= NameRef "(" ParamList? ")" "=" Expr`
fn mapping_def(p: &mut Parser<'_>) {
    let m = p.start();
    name_ref(p);
    param_list(p);
    if !p.eat(Eq) {
        p.error_expecting(
            "expected `=` and the mapping's definition after the parameters",
            &[Eq],
        );
    }
    if expr::expr(p).is_none() {
        p.error_expecting("expected an expression after `=`", &[]);
        p.hint("The definition is what the mapping computes, e.g. `clamp(tilt / (90 deg), 0, 1)`.");
    } else if !p.at_eof() && !p.at_any(ITEM_START) {
        expr::unexpected_after_expr(p, "the next declaration after this definition");
        p.skip_until(ITEM_START);
    }
    m.complete(p, MappingDef);
}

/// `ParamList ::= "(" (Pattern ("," Pattern)* ","?)? ")"`
fn param_list(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // (
    loop {
        if p.at(RParen) || p.at_eof() {
            break;
        }
        if !pattern::pattern(p) {
            if p.at(Comma) {
                p.error_and_bump(
                    SyntaxErrorCode::Expected,
                    "expected a parameter name before `,`",
                );
                continue;
            }
            // Not a parameter and not a separator: the list is over.
            p.error_expecting("expected `)` to close the parameter list", &[RParen, Comma]);
            m.complete(p, ParamList);
            return;
        }
        if p.at(RParen) {
            break;
        }
        if !p.eat(Comma) {
            if pattern::can_start(p.current()) {
                p.error_expecting("expected `,` between parameters", &[Comma]);
                continue;
            }
            p.error_expecting("expected `)` to close the parameter list", &[RParen, Comma]);
            m.complete(p, ParamList);
            return;
        }
    }
    p.expect(RParen, "expected `)` to close the parameter list");
    m.complete(p, ParamList);
}

/// `EnumDecl ::= "enum" Name TypeParamList? "{" (EnumVariant ("," EnumVariant)* ","?)? "}"`
fn enum_decl(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // enum
    if !name(p, "expected the enum's name after `enum`") {
        recover_item(p);
        m.complete(p, EnumDecl);
        return;
    }
    if p.at(Lt) {
        type_param_list(p);
    }
    if !p.eat(LBrace) {
        p.error_expecting("expected `{` and the enum's variants", &[LBrace]);
        recover_item(p);
        m.complete(p, EnumDecl);
        return;
    }
    loop {
        if p.at(RBrace) || p.at_eof() {
            break;
        }
        if p.at(Ident) {
            enum_variant(p);
        } else if p.at(Comma) {
            p.error_and_bump(
                SyntaxErrorCode::Expected,
                "expected a variant name before `,`",
            );
            continue;
        } else if p.at_any(ITEM_START) {
            p.error_expecting("expected `}` to close the enum", &[RBrace]);
            m.complete(p, EnumDecl);
            return;
        } else {
            p.error_and_bump(SyntaxErrorCode::Expected, "expected a variant name");
            continue;
        }
        if p.at(RBrace) {
            break;
        }
        if !p.eat(Comma) {
            if p.at(Ident) {
                p.error_expecting("expected `,` between variants", &[Comma]);
                continue;
            }
            if p.at_any(ITEM_START) || p.at_eof() {
                p.error_expecting("expected `}` to close the enum", &[RBrace]);
                m.complete(p, EnumDecl);
                return;
            }
            p.error_recover(
                SyntaxErrorCode::Expected,
                "expected `,` or `}` after this variant",
                &[RBrace, Comma, Ident],
            );
        }
    }
    p.expect(RBrace, "expected `}` to close the enum");
    recover_item(p);
    m.complete(p, EnumDecl);
}

/// `EnumVariant ::= Name ("(" TypeList ")")?`
fn enum_variant(p: &mut Parser<'_>) {
    let m = p.start();
    name(p, "expected a variant name");
    if p.at(LParen) {
        types::type_list(
            p,
            TypeList,
            LParen,
            RParen,
            "expected `)` to close the variant's fields",
        );
    }
    m.complete(p, EnumVariant);
}

/// `TypeParamList ::= "<" Name ("," Name)* ","? ">"`
fn type_param_list(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // <
    loop {
        if p.at(Gt) || p.at_eof() {
            break;
        }
        if !name(p, "expected a type parameter name") {
            if p.at(Comma) {
                p.bump_as_error();
                continue;
            }
            break;
        }
        if p.at(Gt) {
            break;
        }
        if !p.eat(Comma) {
            if p.at(Ident) {
                p.error_expecting("expected `,` between type parameters", &[Comma]);
                continue;
            }
            break;
        }
    }
    p.expect(Gt, "expected `>` to close the type parameters");
    m.complete(p, TypeParamList);
}

/// A definition-site name.  Reports reserved words and missing names.
pub(super) fn name(p: &mut Parser<'_>, message: &str) -> bool {
    if p.at(Ident) {
        let m = p.start();
        p.bump();
        m.complete(p, Name);
        return true;
    }
    if p.current().is_future_reserved() {
        reserved_word(p);
        let m = p.start();
        p.bump();
        m.complete(p, Name);
        return true;
    }
    p.error_expecting(message, &[Ident]);
    false
}

/// A reference-site name (the current token must be `Ident`).
pub(super) fn name_ref(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump();
    m.complete(p, NameRef);
}

pub(super) fn reserved_word(p: &mut Parser<'_>) {
    let word = p.current_text().to_owned();
    p.error(
        SyntaxErrorCode::ReservedWord,
        format!("`{word}` is reserved for a future version of BDL"),
    );
    p.hint(format!(
        "Choose another name; `{word}` will become a keyword."
    ));
}

/// After an item: anything before the next item keyword is an error.
fn recover_item(p: &mut Parser<'_>) {
    if p.at_eof() || p.at_any(ITEM_START) {
        return;
    }
    let message = if p.at(Ident) && p.nth(1) == LParen {
        format!(
            "a definition of `{}` needs its `mapping` signature right above it",
            p.current_text()
        )
    } else {
        "expected `concept`, `mapping` or `enum` here".to_owned()
    };
    p.error(SyntaxErrorCode::Unexpected, message);
    p.skip_until(ITEM_START);
}
