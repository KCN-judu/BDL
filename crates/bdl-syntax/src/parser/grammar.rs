//! Module and items (`docs/spec/textual-syntax.md` §4.1) and the formula entry
//! point.

use super::{expr, pattern, types, Parser};
use crate::kind::SyntaxKind::{self, *};
use crate::syntax::SyntaxErrorCode;

/// The keywords a top-level item may start with (§14.1).
const ITEM_START: &[SyntaxKind] = &[
    KwConcept,
    KwMapping,
    KwEnum,
    KwClock,
    KwOutput,
    KwDrive,
    KwDevice,
    KwComponent,
    KwInstance,
    KwBind,
    KwExport,
];

/// The keywords a component item may start with, plus the brace that
/// closes the body (§14.2).
const COMPONENT_ITEM_START: &[SyntaxKind] = &[
    KwConcept, KwMapping, KwEnum, KwClock, KwOutput, KwDrive, KwDevice, KwUse, KwParam, KwRequires,
    KwProvides, RBrace,
];

/// Where a top-level item is: the anchors recovery stops at.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    Module,
    Component,
}

impl Scope {
    fn anchors(self) -> &'static [SyntaxKind] {
        match self {
            Scope::Module => ITEM_START,
            Scope::Component => COMPONENT_ITEM_START,
        }
    }
    fn expected(self) -> &'static str {
        match self {
            Scope::Module => "expected a declaration here: `concept`, `mapping`, `enum`, `clock`, `output`, `drive`, `device`, `component`, `instance`, `bind` or `export`",
            Scope::Component => "expected a component item here: `concept`, `use`, `clock`, `param`, `requires`, `provides`, `mapping`, `enum`, `output`, `drive`, `device` — or `}` to close the component",
        }
    }
}

/// `Module ::= Item* EOF`
pub(super) fn module(p: &mut Parser<'_>) {
    while !p.at_eof() {
        match p.current() {
            KwConcept => concept_decl(p, Scope::Module),
            KwMapping => mapping_decl(p, Scope::Module),
            KwEnum => enum_decl(p, Scope::Module),
            KwClock => clock_decl(p, Scope::Module),
            KwOutput => output_decl(p, Scope::Module),
            KwDrive => drive_decl(p, Scope::Module),
            KwDevice => device_decl(p, Scope::Module),
            KwComponent => component_decl(p),
            KwInstance => instance_decl(p),
            KwBind => bind_decl(p),
            KwExport => export_decl(p),
            _ => unexpected_item(p, Scope::Module),
        }
    }
}

fn unexpected_item(p: &mut Parser<'_>, scope: Scope) {
    let message = if p.at(Ident) && p.nth(1) == LParen {
        format!(
            "a definition of `{}` needs its `mapping` signature right above it",
            p.current_text()
        )
    } else if p.current().is_component_item_start() && scope == Scope::Module {
        format!(
            "`{}` belongs inside a `component {{ … }}` body",
            p.current_text()
        )
    } else if p.current().is_item_start() && scope == Scope::Component {
        format!(
            "`{}` is a top-level declaration; close the component with `}}` first",
            p.current_text()
        )
    } else {
        scope.expected().to_owned()
    };
    p.error(SyntaxErrorCode::Unexpected, message);
    p.hint("Every declaration starts with one of these keywords.");
    p.skip_until(scope.anchors());
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
fn concept_decl(p: &mut Parser<'_>, scope: Scope) {
    let m = p.start();
    p.bump(); // concept
    if !name(p, "expected the concept's name after `concept`") {
        recover_item(p, scope);
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
    recover_item(p, scope);
    m.complete(p, ConceptDecl);
}

/// `MappingDecl ::= "mapping" Name ":" Type ClockTag? MappingDef?`
fn mapping_decl(p: &mut Parser<'_>, scope: Scope) {
    let m = p.start();
    p.bump(); // mapping
    if !name(p, "expected the mapping's name after `mapping`") {
        recover_item(p, scope);
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
    clock_tag(p);
    if p.at(Ident) && p.nth(1) == LParen {
        mapping_def(p, scope);
    }
    recover_item(p, scope);
    m.complete(p, MappingDecl);
}

/// `ClockTag ::= "@" NameRef` — optional; nothing is consumed without `@`.
fn clock_tag(p: &mut Parser<'_>) -> bool {
    if !p.at(At) {
        return false;
    }
    let m = p.start();
    p.bump(); // @
    if p.at(Ident) {
        name_ref(p);
    } else {
        p.error_expecting("expected a timing domain's name after `@`", &[Ident]);
        p.hint("For example `mapping tilt : Tilt @interaction`.");
    }
    m.complete(p, ClockTag);
    true
}

/// `MappingDef ::= NameRef "(" ParamList? ")" "=" Expr`
fn mapping_def(p: &mut Parser<'_>, scope: Scope) {
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
        p.hint("The definition is what the mapping computes, e.g. `tilt / (90 deg)`.");
    } else if !p.at_eof() && !p.at_any(scope.anchors()) {
        expr::unexpected_after_expr(p, "the next declaration after this definition");
        p.skip_until(scope.anchors());
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
fn enum_decl(p: &mut Parser<'_>, scope: Scope) {
    let m = p.start();
    p.bump(); // enum
    if !name(p, "expected the enum's name after `enum`") {
        recover_item(p, scope);
        m.complete(p, EnumDecl);
        return;
    }
    if p.at(Lt) {
        type_param_list(p);
    }
    if !p.eat(LBrace) {
        p.error_expecting("expected `{` and the enum's variants", &[LBrace]);
        recover_item(p, scope);
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
        } else if p.at_any(ITEM_START) && !p.at(RBrace) {
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
            if (p.at_any(ITEM_START) && !p.at(RBrace)) || p.at_eof() {
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
    recover_item(p, scope);
    m.complete(p, EnumDecl);
}

// ---- v0.2 items (§14) ------------------------------------------------------

/// `ClockDecl ::= "clock" Name`
fn clock_decl(p: &mut Parser<'_>, scope: Scope) {
    let m = p.start();
    p.bump(); // clock
    name(p, "expected the timing domain's name after `clock`");
    recover_item(p, scope);
    m.complete(p, ClockDecl);
}

/// `OutputDecl ::= "output" Name ":" Type ClockTag? ("optional")?`
fn output_decl(p: &mut Parser<'_>, scope: Scope) {
    let m = p.start();
    p.bump(); // output
    if !name(p, "expected the output's name after `output`") {
        recover_item(p, scope);
        m.complete(p, OutputDecl);
        return;
    }
    if !p.eat(Colon) {
        p.error_expecting("expected `:` and the concept the output accepts", &[Colon]);
        p.hint("A physical output is declared as `output light : Brightness @interaction`.");
    }
    if !types::type_(p) {
        p.error_expecting("expected the concept the output accepts", &[Ident]);
    }
    clock_tag(p);
    if p.at(Ident) && p.current_text() == "optional" {
        p.bump();
    }
    recover_item(p, scope);
    m.complete(p, OutputDecl);
}

/// `DriveDecl ::= "drive" NameRef "=" NameRef`
fn drive_decl(p: &mut Parser<'_>, scope: Scope) {
    let m = p.start();
    p.bump(); // drive
    if p.at(Ident) {
        name_ref(p);
    } else {
        p.error_expecting("expected the output's name after `drive`", &[Ident]);
        p.hint("`drive light = brightness` connects the relationship `brightness` to the output `light`.");
    }
    if p.eat(Eq) {
        if p.at(Ident) {
            name_ref(p);
        } else {
            p.error_expecting(
                "expected the driving relationship's name after `=`",
                &[Ident],
            );
        }
    } else {
        p.error_expecting("expected `=` and the driving relationship", &[Eq]);
    }
    recover_item(p, scope);
    m.complete(p, DriveDecl);
}

/// `DeviceDecl ::= "device" Name ":" Ident ("for" NameRef)? DeviceBody?`
fn device_decl(p: &mut Parser<'_>, scope: Scope) {
    let m = p.start();
    p.bump(); // device
    if !name(p, "expected the device's name after `device`") {
        recover_item(p, scope);
        m.complete(p, DeviceDecl);
        return;
    }
    if !p.eat(Colon) {
        p.error_expecting("expected `:` and the device's kind", &[Colon]);
        p.hint("For example `device pwmLight : pwm_channel for light`.");
    }
    if p.at(Ident) {
        name_ref(p); // the kind
    } else {
        p.error_expecting(
            "expected the device's kind, such as `pwm_channel`",
            &[Ident],
        );
    }
    if p.at(Ident) && p.current_text() == "for" {
        p.bump();
        if p.at(Ident) {
            name_ref(p);
        } else {
            p.error_expecting(
                "expected the output this device realises, after `for`",
                &[Ident],
            );
        }
    }
    if p.at(LBrace) {
        device_body(p);
    }
    recover_item(p, scope);
    m.complete(p, DeviceDecl);
}

/// `DeviceBody ::= "{" ( PinFix ","? )* "}"`, `PinFix ::= "pin" Number "=" Ident`
fn device_body(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // {
    loop {
        if p.at(RBrace) || p.at_eof() {
            break;
        }
        if p.at(Ident) && p.current_text() == "pin" {
            let pin = p.start();
            p.bump(); // pin
            if !p.eat(Number) {
                p.error_expecting("expected the requirement's index after `pin`", &[Number]);
            }
            if !p.eat(Eq) {
                p.error_expecting("expected `=` and a pin name", &[Eq]);
            }
            if p.at(Ident) {
                name_ref(p);
            } else {
                p.error_expecting("expected a board pin name such as `D3`", &[Ident]);
            }
            pin.complete(p, PinFix);
            p.eat(Comma);
            continue;
        }
        if p.at_any(ITEM_START) {
            p.error_expecting("expected `}` to close the device", &[RBrace]);
            m.complete(p, DeviceBody);
            return;
        }
        p.error_and_bump(
            SyntaxErrorCode::Expected,
            "expected `pin <index> = <name>` or `}` here",
        );
    }
    p.expect(RBrace, "expected `}` to close the device");
    m.complete(p, DeviceBody);
}

/// `ComponentDecl ::= "component" Name "{" ComponentItem* "}"`
fn component_decl(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // component
    if !name(p, "expected the component's name after `component`") {
        recover_item(p, Scope::Module);
        m.complete(p, ComponentDecl);
        return;
    }
    if !p.at(LBrace) {
        p.error_expecting("expected `{` and the component's items", &[LBrace]);
        p.hint("A component is `component Name { … }` with its ports and relationships inside.");
        recover_item(p, Scope::Module);
        m.complete(p, ComponentDecl);
        return;
    }
    let body = p.start();
    p.bump(); // {
    loop {
        match p.current() {
            RBrace | Eof => break,
            KwConcept => concept_decl(p, Scope::Component),
            KwMapping => mapping_decl(p, Scope::Component),
            KwEnum => enum_decl(p, Scope::Component),
            KwClock => clock_decl(p, Scope::Component),
            KwOutput => output_decl(p, Scope::Component),
            KwDrive => drive_decl(p, Scope::Component),
            KwDevice => device_decl(p, Scope::Component),
            KwUse => use_decl(p),
            KwParam => param_decl(p),
            KwRequires | KwProvides => port_decl(p),
            k if k.is_item_start() => {
                // A top-level keyword inside the body: the `}` is missing.
                p.error_expecting("expected `}` to close the component", &[RBrace]);
                break;
            }
            _ => unexpected_item(p, Scope::Component),
        }
    }
    p.expect(RBrace, "expected `}` to close the component");
    body.complete(p, ComponentBody);
    recover_item(p, Scope::Module);
    m.complete(p, ComponentDecl);
}

/// `UseDecl ::= "use" ("concept" | "output") NameRef`
fn use_decl(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // use
    if p.at(KwConcept) || p.at(KwOutput) {
        p.bump();
    } else {
        p.error_expecting(
            "expected `concept` or `output` after `use`",
            &[KwConcept, KwOutput],
        );
        p.hint("`use concept Brightness` shares the system's concept with this component.");
    }
    if p.at(Ident) {
        name_ref(p);
    } else {
        p.error_expecting("expected the shared item's name", &[Ident]);
    }
    recover_item(p, Scope::Component);
    m.complete(p, UseDecl);
}

/// `ParamClockDecl ::= "param" "clock" Name` or a parameter port
/// (`PortDecl` starting with `param`).
fn param_decl(p: &mut Parser<'_>) {
    if p.nth(1) == KwClock {
        let m = p.start();
        p.bump(); // param
        p.bump(); // clock
        name(
            p,
            "expected the timing parameter's name after `param clock`",
        );
        recover_item(p, Scope::Component);
        m.complete(p, ParamClockDecl);
        return;
    }
    port_decl(p);
}

/// `PortDecl ::= ("requires" | "provides" | "param") Name ":" Type ClockTag? MappingDef?`
fn port_decl(p: &mut Parser<'_>) {
    let m = p.start();
    let word = p.current_text().to_owned();
    p.bump(); // requires | provides | param
    if !name(p, &format!("expected the port's name after `{word}`")) {
        recover_item(p, Scope::Component);
        m.complete(p, PortDecl);
        return;
    }
    if !p.eat(Colon) {
        p.error_expecting("expected `:` and the port's concept", &[Colon]);
        p.hint("For example `requires tiltValue : Tilt @main`.");
    }
    if !types::type_(p) {
        p.error_expecting("expected the concept the port carries", &[Ident]);
    }
    clock_tag(p);
    if p.at(Ident) && p.nth(1) == LParen {
        mapping_def(p, Scope::Component);
    }
    recover_item(p, Scope::Component);
    m.complete(p, PortDecl);
}

/// `InstanceDecl ::= "instance" Name ":" NameRef InstanceBody?`
fn instance_decl(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // instance
    if !name(p, "expected the instance's name after `instance`") {
        recover_item(p, Scope::Module);
        m.complete(p, InstanceDecl);
        return;
    }
    if !p.eat(Colon) {
        p.error_expecting("expected `:` and the component's name", &[Colon]);
        p.hint("For example `instance lampA : AdaptiveLamp { main = interaction }`.");
    }
    if p.at(Ident) {
        name_ref(p);
    } else {
        p.error_expecting("expected the component this is an instance of", &[Ident]);
    }
    if p.at(LBrace) {
        let body = p.start();
        p.bump(); // {
        loop {
            if p.at(RBrace) || p.at_eof() {
                break;
            }
            if p.at(Ident) {
                let arg = p.start();
                name_ref(p);
                if !p.eat(Eq) {
                    p.error_expecting("expected `=` and the argument's value", &[Eq]);
                }
                if expr::expr(p).is_none() {
                    p.error_expecting("expected a timing domain or a value after `=`", &[Ident]);
                }
                arg.complete(p, InstanceArg);
                p.eat(Comma);
                continue;
            }
            if p.at_any(ITEM_START) {
                p.error_expecting("expected `}` to close the instance", &[RBrace]);
                body.complete(p, InstanceBody);
                recover_item(p, Scope::Module);
                m.complete(p, InstanceDecl);
                return;
            }
            p.error_and_bump(
                SyntaxErrorCode::Expected,
                "expected `name = value` or `}` here",
            );
        }
        p.expect(RBrace, "expected `}` to close the instance");
        body.complete(p, InstanceBody);
    }
    recover_item(p, Scope::Module);
    m.complete(p, InstanceDecl);
}

/// `BindEnd ::= NameRef ("." NameRef)?`
fn bind_end(p: &mut Parser<'_>, what: &str) -> bool {
    if !p.at(Ident) {
        p.error_expecting(&format!("expected {what}"), &[Ident]);
        return false;
    }
    let m = p.start();
    name_ref(p);
    if p.eat(Dot) {
        if p.at(Ident) {
            name_ref(p);
        } else {
            p.error_expecting("expected a port name after `.`", &[Ident]);
        }
    }
    m.complete(p, BindEnd);
    true
}

/// `BindDecl ::= "bind" BindEnd "=" BindEnd ("init" Expr)?`
fn bind_decl(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // bind
    if bind_end(
        p,
        "the destination (`instance.port` or a relationship) after `bind`",
    ) {
        if p.eat(Eq) {
            if bind_end(
                p,
                "the source (`instance.port` or a relationship) after `=`",
            ) && p.at(Ident)
                && p.current_text() == "init"
            {
                p.bump(); // init
                if expr::expr(p).is_none() {
                    p.error_expecting("expected the initial value after `init`", &[Number]);
                }
            }
        } else {
            p.error_expecting("expected `=` and the source of the binding", &[Eq]);
            p.hint("`bind lampA.tiltValue = tiltValue` supplies the port from the relationship.");
        }
    }
    recover_item(p, Scope::Module);
    m.complete(p, BindDecl);
}

/// `ExportDecl ::= "export" BindEnd "as" Name`
fn export_decl(p: &mut Parser<'_>) {
    let m = p.start();
    p.bump(); // export
    if bind_end(p, "the instance's port to export, as `instance.port`") {
        if p.at(Ident) && p.current_text() == "as" {
            p.bump();
            name(p, "expected the exported name after `as`");
        } else {
            p.error_expecting("expected `as` and the exported name", &[Ident]);
        }
    }
    recover_item(p, Scope::Module);
    m.complete(p, ExportDecl);
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

/// After an item: anything before the next item keyword (or, in a
/// component body, the closing brace) is an error.
fn recover_item(p: &mut Parser<'_>, scope: Scope) {
    if p.at_eof() || p.at_any(scope.anchors()) {
        return;
    }
    let message = if p.at(Ident) && p.nth(1) == LParen {
        format!(
            "a definition of `{}` needs its `mapping` signature right above it",
            p.current_text()
        )
    } else {
        scope.expected().to_owned()
    };
    p.error(SyntaxErrorCode::Unexpected, message);
    p.skip_until(scope.anchors());
}
