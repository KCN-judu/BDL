# Text projects

Findings a text project reports about its files, in the editor (on the item) and
in `bdld check` (with `file:line:col`). A text project's own findings are red:
the file says something the design cannot mean, and the item is left out until
it is fixed. Everything else — a wrong unit, a missing driver, a domain crossing
— is the same finding as in Studio, on the same page of this section.

## _… is not a concept of this project; declare it with `concept … : …` or pick an existing one._

The name in a signature, `output`, `use concept` or port is not declared in any
file of the project. Declare it (any file under `src/`) or fix the spelling.
Inside a component, only the body's own concepts and the ones it `use`s are in
scope. — `text.unknown_concept`

## _… is not a timing domain here; declare it with `clock …`._

The `@name` names no domain in scope: at the top level, a `clock` of the
project; in a component, one of its `clock` or `param clock` items. —
`text.unknown_clock`

## _… is not a relationship of this project._ · _… is not a physical output of this project._ · _… is not a component of this project._ · _… is not an instance of this project._ · _… has no port …_

A `drive`, `device … for`, `instance … : C`, `bind` or `export` names something
that does not exist at that level. Check the spelling and where it is declared;
a component's ports are the `requires`, `provides` and `param` items of its
body. — `text.unknown_relationship`, `text.unknown_output`,
`text.unknown_component`, `text.unknown_instance`, `text.unknown_port`

## _… is declared twice (… first); the first declaration is the one used._

Two items of the same kind and name in the project (or in one component). The
second is ignored. Rename or remove one. — `text.duplicate_item`

## _this output is already driven by …; an output has one driver, so this `drive` is ignored._

Two `drive` lines for one output. An output has exactly one final driver
([Physical outputs](../concepts/physical-outputs.md)); combine the two
relationships into one that drives it. — `text.second_driver`

## _… is a required port: what it needs comes from outside, so it has no definition here._ · _… is a parameter: each instance gives it a value, so it has no definition here._

A `requires` or `param` port has a `name() = …` definition. Only a `provides`
port is computed by the component; make it `provides`, or drop the definition. —
`text.port_definition`

## _… is a timing parameter of …; give it a timing domain of the system._ · _… is neither a timing parameter nor a parameter port of …_

The braces of an instance give every `param clock` a domain of the system and
every `param` a constant, by name. — `text.bad_argument`

## _a binding joins an instance's port to something; two top-level relationships are related by a formula, not a binding._ · _only an instance's port can be exported, as `instance.port`._

`bind` needs an `instance.port` on at least one side; `export` takes exactly
one. Two top-level relationships are connected by naming one in the other's
formula. — `text.bad_binding`

## _… is not a value form; write Bool, Count or a quantity such as …_ · _… is not a device kind; one of pwm_channel, …_

The type after `concept … :` or the kind after `device … :` is not one the tool
knows. — `text.unknown_representation`, `text.unknown_kind`

## _a parameter is a plain name (or `_`); patterns cannot destructure an input._

`f(Some(x)) = …` is not allowed; name the input and `match` on it in the body. —
`text.bad_parameter`

## _enums are syntax only in this version; the design model has no sum types yet._

Reported as _open_, not an error: the `enum` parses and is skipped. —
`text.unsupported_item`

## _… is new and more than one declaration of its kind disappeared from this file, so it could not be matched to an existing identity; it has a fresh one. Use the editor's rename to keep an identity._

You renamed two items of the same kind in one file by hand between two reads of
the project. The tools cannot tell which old item became which new one, so both
are new: nothing semantic changes, but their canvas positions and group
membership are gone. Rename one at a time, or use the editor's rename, which
knows the identity ([Overview — Identity](../textual/overview.md#identity)). —
`text.ambiguous_identity`

## A banner in Studio: _… changed on disk since the project was opened: …_

You pressed Save while a source file had been changed by another tool. Nothing
was written. _Reload from disk_ takes the files as they are now (Studio's
unsaved edits are dropped); _Overwrite_ writes Studio's version over them;
_Dismiss_ leaves both as they are
([Authoring a project as text](../workflows/authoring-as-text.md)). —
`project.changed_on_disk`

## Related

[Syntax basics](../textual/syntax-basics.md) ·
[Types, units and concepts](type-and-concept-errors.md) ·
[Connections](connection-errors.md)
