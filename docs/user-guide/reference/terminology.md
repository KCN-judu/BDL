# Terminology

One word per idea. The first column is the word this guide uses; the
*Studio label* column is what the interface says where it differs; the
last column is the formal or internal name, which appears only in
*Explain* and in the technical documentation.

The ladder the whole guide stands on: a **value form** is what kind of
value something is; a **concept** is a named kind of value — a type, a
template; a **Sem block** (a *block*) is one instance of a concept in the
design; the **value at a tick** is what that block holds at one tick.
Concept : Sem block = type : instance. In parallel, a **rule** is a
template from concepts to a concept, and a **mapping block** is one
application of rules — the formula that defines one Sem block. Rule :
mapping block = template : application. The canvas draws Sem blocks,
mapping blocks and outputs, joined by read, produce and drive edges;
concepts and rules are templates and are not on it.

| Guide word | Meaning | Studio label | Formal / internal |
| --- | --- | --- | --- |
| **concept** | a named *kind* of value — *Tilt*, *Brightness*: a type, and the template its Sem blocks are instances of; never itself a value or a node | Concept | the nominal type `sem C` ("a Sem of *C*"), `ConceptId` |
| **Sem block** (*block*) | one instance of a concept in the design, holding one value per tick: a Source when nothing defines it, a value when its mapping block does; several Sem blocks of one concept are ordinary | a block on the canvas; *Source* / *Value* (Role) | unit-domain declaration of type `sem C`; an instance |
| **value at a tick** | what a Sem block holds at one tick — the number, the on/off state, the count the simulator shows in its column | the trace column, the probe | `Value.sem C v`, one per activation |
| **mapping block** | the definition of one Sem block — its formula, applying rules to the Sem blocks it reads — drawn as a node of its own beside that Sem block: one declaration, two nodes | Relationship (section); the node headed by the rules it applies, or *Formula* | the Sem block's realization, its one producer |
| **value form** | what kind of value a concept carries: quantity (with a unit), on / off, count, or *decide later* | Value | representation `Θ`: `q dim`, `bool`, `nat`, or none |
| **dimension** | the physical kind of a quantity (angle, length, time…), what formulas are checked against | Unit (chosen by quantity kind) | `Dim` |
| **relationship** | what *+* by *Mappings* creates: a name, a signature and an optional formula — a Sem block when it reads nothing, a rule when it reads something | Mapping (sidebar, sheet), Relationship (inspector) | declaration, `DeclId`, `MappingBlock` |
| **reads** / **produces** | a relationship's signature: a rule's parameters and its result; a Sem block's concept | Reads (a rule's parameter editor), Produces / Provides | interface, `Signature { inputs, output }` |
| **rule** | a relationship with inputs: a template from concepts to a concept that a Sem block's formula applies; it has no value of its own and is not a node on the canvas | *Rule* (Role), *Applies* | arrow-typed declaration |
| **value** (Role) | a Sem block with a mapping block: a computed value | *Value* (Role) | nullary declaration with a realization |
| **produces** | the concept a relationship's signature ends in, rule or Sem block alike | Produces, a Sem block's output socket | `Signature.output`, `Ty.grant` |
| **reads** (a Sem block) | the Sem blocks its formula names — the read edges into its mapping block | *Reads*, *Read by* (inspector) | `dependsOn`, `MappingAnalysis.references` |
| **read edge** | a link from a Sem block's output socket into a mapping block's socket for it: *this formula reads that block*; read off the compiler's analysis; *Disconnect* replaces the name in the formula by `?` | a link in the read block's colour | `references`, `ComposeAction.unreference` |
| **produce edge** | the short joint from a mapping block's output socket into its Sem block: *this formula defines that block* — the one definition; it cannot be disconnected (the inspector's *Detach definition* is the way) | the joint beside a block | `producedBy`, write-once |
| **drive edge** | a link from a Sem block's output socket into a physical output: *this block drives that output*; dragged to connect, disconnected like any link | a link into a sink | the drive relation `β` |
| **Source** | a Sem block with no formula: the environment provides its value, once per activation (zh-Hans 来源, ja 入力元; never *sensor*, *signal* or *source code*) | *Source* (canvas header, Role, the Library's *Sources*, the Simulate page's *Sources*) | unresolved nullary declaration at the environment boundary; `Source Δ d`, `I d t`; a derived role, stored nowhere |
| **formula** | the text that defines a relationship | Relationship (section) | definition, realization, Core term |
| **declared** | a rule that has no formula yet | *declared* (inspector, status line) | `realization = none` with inputs |
| **role** | what a relationship is where you are: *Source*, *Relationship*, or the port it backs | Role (inspector) | `relationship_role` |
| **environment** | what is outside the behavior model and provides a Source's value | *Provided by the environment* | the simulation's input `I` |
| **open** | a relationship whose formula waits on a concept's value form | *Checked once … is decided.* | `MappingStatus::Open`, `concept.unbound_representation` |
| **invalid** | a formula that does not check | red mark | `MappingStatus::Invalid` |
| **valid** | checks, has a value at every activation, timing consistent | — | `TypeValid` → `TemporallyValid` → `ClockConsistent` |
| **executable** | every needed relationship valid and every required output driven once | status line shows nothing open | `output_complete ∧ executable` |
| **timing domain** | a named rhythm; the values that tick together | Timing domain; *Updates in* | clock, `ClockId`, `Κ` |
| **memory** | last tick's value, written `delay(init, e)` | — | `delay` state cell |
| **carrying across domains** | reading another domain's last value, written `sync(domain, init, e)` or chosen in the *Starts at* sheet | Carry across timing domains; Starts at | `sync`, transport |
| **instantaneous cycle** | values needing each other in one tick | *depend on each other in the same instant* | not `Causal`, `reactive.instantaneous_cycle` |
| **physical output** | where a value leaves the design: a light, a motor | Output | `OutputId`, `Ω`, `OutputSpec` |
| **driver** / **drives** | the one Sem block connected to an output | Driver, Drives | drive edge `β` |
| **contested** | an output with more than one driver | *contested* | `SingleDriver` violated |
| **device** | the hardware that realises one output on a board | Device (Deploy) | `DeviceBinding`, `DeviceKind` |
| **board** / **target** | the hardware the placement is checked against | Target | `Hardware`, `TargetDescriptor` |
| **placement** | which board pin each device requirement uses | Placement | `Assignment`; `Requirement` |
| **feasible** | every requirement placed | *Feasible on …* | `DeploymentStatus::Feasible` |
| **behavior** (group) | a named set of relationships; organisation only | Behavior, Group as Behavior | `BehaviorGroup`, authoring metadata |
| **boundary** | what a behavior's members read from and give to the outside | Boundary: Inputs, Outputs, Open, Physical outputs, Internal | crossing-in, crossing-out |
| **component** | a reusable behavior with a source and a promise | Component | `BehaviorComponent`, `ComponentId` |
| **source** (of a component) | a component's own design — a different word from *Source* above | Edit Source | body |
| **promise** / **port** | what a component requires, provides, or takes as a parameter | Ports: Requires, Provides, Parameter | `BehaviorInterface`, `Port`, `PortContract`, `PortId` |
| **timing parameter** | a component's domain that each instance maps to a system domain | Timing parameters | clock parameter |
| **shared concept** | a concept of a source that stands for a system concept | Shared concepts | `shared_concepts` |
| **instance** | one use of a component | Instance | `ComponentInstance`, `ComponentInstanceId` |
| **binding** | a connection from a provided port or value to a required port or open relationship | Connection, Binding | `Binding`, `BindingId`, `Definition::Reference` |
| **fan-out** | one value read by several required ports | — | several bindings from one source |
| **version** | a copy of a component with the same promise | Duplicate as Version | `DuplicateComponent` |
| **system** | a design with instances; what every project holds | System (context bar) | `BehaviorSystem`; flattened design |
| **Design view / Code view / Split** | the project as a graph, as its `.bdl` files, or both side by side | Design · Code · Split (Design page) | `DesignView`; `GetSources`, `ApplySourceEdit` |
| **does not build yet** | the text in the Code view is not a design yet; the graph shows the last version that was | *This file does not build yet …* | a draft the loader refused; last-known-good revision |
| **revision** | one version of the design; every edit makes a new one | Explain | `Revision` |
| **draft** | a typed but not yet added formula | *unsaved* | `MappingDefinitionDraft` overlay |
| **finding** | what the compiler reports about an object | (under the field it concerns) | diagnostic, code |
| **fix** | an action the tool offers for a finding | Fixes | semantic action, edit plan |
| **item** | a ready-made Concept, or a Source preset, in the Library | Library | `LibraryItem` id — never an identity |

Words this guide does not use for designer-facing text: *mapping* (except
to name the Studio label), *signal*, *event*, *handler*, *flatten*,
*elaborate*, *judgment*, and any identifier such as `DeclId`.
