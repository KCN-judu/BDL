---
kind: specification
area: language
status: current
---

# BDL 内核规范（从 BDL_FV Lean 4 开发转录）

这是工程实现要对齐的**精确契约**。每一条都对应 `BDL_FV/BDL/Core/*.lean` 或
`BDL_FV/BDL/Validation/Hardware.lean`
里的定义；实现可以换数据表示（例如用哈希表代替函数环境、用 `Int`/浮点代替
`Nat`），但**判定结果必须与 Lean 定义一致**。

Lean 源文件对照：

| 本文节                    | Lean 文件                                 |
| ------------------------- | ----------------------------------------- |
| §1 身份与类型             | `Core/Base.lean`                          |
| §2 声明与环境             | `Core/Interface.lean`, `Core/Decl.lean`   |
| §3 类型判定               | `Core/Typing.lean`                        |
| §4 满足性、良构、生命周期 | `Core/Satisfaction.lean`, `Core/Env.lean` |
| §5 依赖与因果             | `Core/Dependency.lean`                    |
| §6 单域求值               | `Core/Reactive.lean`                      |
| §7 时钟域与多域求值       | `Core/Clock.lean`                         |
| §8 物理输出               | `Core/Output.lean`                        |
| §9 硬件验证               | `Validation/Hardware.lean`                |

---

## 1. 身份与类型（Base）

三种**互相独立**的名义身份，加声明身份：

```text
SemanticId  = { n : Nat }   -- 概念（是类型）
ClockId     = { n : Nat }   -- 时钟域（不是速率）
OutputId    = { n : Nat }   -- 物理 sink（是资源）
DeclId      = { n : Nat }   -- 声明（是值）
```

显示名不在内核里。

量纲：三个基本维度的指数向量（论文说"不是 SI 目录"，实现可扩展）：

```text
Dim = { length, time, angle : Int }
Dim.zero, Dim.add, Dim.sub, Dim.Length, Dim.Time, Dim.Angle
```

类型：

```text
Ty ::= bool | nat | arr Ty Ty | sem SemanticId | q Dim | opt Ty
```

谓词：

- `Ty.SemFree`：不含 `sem`（递归进 arr/opt）
- `Ty.Data`：不含 `arr`（递归进 opt）

原语及其类型（量纲代数**全部**在这里）：

```text
Prim ::= lit d n | add d | sub d | mul d₁ d₂ | div d₁ d₂
       | lt d | eq d | not | and | or | ite τ
       | none τ | some τ | isSome τ | getD τ

lit d n   : q d
add d     : q d → q d → q d            sub d : 同
mul d₁ d₂ : q d₁ → q d₂ → q (d₁+d₂)
div d₁ d₂ : q d₁ → q d₂ → q (d₁−d₂)
lt d, eq d: q d → q d → bool
not : bool → bool ; and, or : bool → bool → bool
ite τ  : bool → τ → τ → τ
none τ : opt τ ; some τ : τ → opt τ
isSome τ : opt τ → bool ; getD τ : opt τ → τ → τ
```

表达式（de Bruijn）：

```text
Expr ::= var i | boolLit b | natLit n
       | lam dom body | app f a
       | declRef d
       | rep e | mk s e
       | prim p
       | delay init e
       | sync src init e
```

辅助：

- `refs e`：所有 `declRef`（含 delay/sync 两个操作数）
- `instRefs e`：瞬时引用 —— `delay i _` 只取 `i.instRefs`；`sync _ i _` 只取
  `i.instRefs`（传输永不瞬时）
- `DelayFree e`：不含 delay/sync
- `RefFree e`：`refs e = []`

## 2. 声明与环境（Interface, Decl）

```text
PropertyId      -- 原子标签（commitment / obligation 共用）
DeclInterface   = { expectedType : Ty, commitments : List PropertyId }
DesignDecl      = { id : DeclId, interface : DeclInterface, realization : Option Expr }
DeclEnv         = DeclId → Option DesignDecl
```

- `InterfaceRefines old new` ⇔
  `old.expectedType = new.expectedType ∧ old.commitments ⊆ new.commitments`（可判定预序）
- `DeclEnv.update Δ h` = 把 `h` 存到 `h.id` 下
- `DeclEnv.tyView Δ d` = `(Δ d).map (·.interface.expectedType)` ——
  **类型判定读 Δ 的唯一入口**
- `DeclEnv.realizationOf Δ d` = `(Δ d).bind (·.realization)`

结构生命周期序：

- `DeclLeq h₁ h₂` ⇔ 同 id ∧ `InterfaceRefines h₁.interface h₂.interface` ∧ (∀ e,
  h₁.realization = some e → h₂.realization = some e)（一次写入）
- `EnvRefines Δ₁ Δ₂` ⇔ ∀ d h₁, Δ₁ d = some h₁ → ∃ h₂, Δ₂ d = some h₂ ∧ DeclLeq
  h₁ h₂（允许新声明）

概念环境：

```text
ConceptEnv Θ = SemanticId → Option Ty
ConceptEnv.WF Θ  ⇔ ∀ s R, Θ s = some R → R.SemFree ∧ R.Data
ConceptRefines Θ₁ Θ₂ ⇔ ∀ s R, Θ₁ s = some R → Θ₂ s = some R   -- 只增不改
```

Grant：

```text
Grant = SemanticId → Prop
Ty.grant : Ty → List SemanticId     -- 签名结果位置的概念
  grant (sem s)   = [s]
  grant (arr _ b) = grant b
  grant _         = []                -- 注意：opt (sem s) 不授予 s
Grant.of τ = fun s => s ∈ τ.grant
Grant.none / Grant.all
```

## 3. 类型判定（Typing）

`HasType Θ Δ G Γ e τ`，`Γ : List Ty`（de Bruijn 上下文）：

```text
var      Γ[i] = τ                                  ⊢ var i : τ
boolLit                                            ⊢ boolLit b : bool
natLit                                             ⊢ natLit n : nat
lam      (dom::Γ) ⊢ body : cod                     ⊢ lam dom body : arr dom cod
app      ⊢ f : arr dom cod ; ⊢ a : dom             ⊢ app f a : cod
declRef  Δ.tyView d = some τ                       ⊢ declRef d : τ
rep      Θ s = some R ; ⊢ e : sem s                ⊢ rep e : R
mk       G s ; Θ s = some R ; ⊢ e : R              ⊢ mk s e : sem s
prim                                               ⊢ prim p : p.ty
delay    τ.Data ; [] ⊢ i : τ ; [] ⊢ e : τ          [] ⊢ delay i e : τ        -- 仅空上下文
sync     τ.Data ; [] ⊢ i : τ ; [] ⊢ e : τ          [] ⊢ sync c i e : τ       -- 仅空上下文；域与类型无关
```

`infer Θ Δ G Γ e : Option Ty` 是语法制导的推断，已证 sound / complete /
unique。实现按上面规则直接写即可；关键细节：`app` 要求 `dom = dom'`
精确相等（无子类型）；`delay`/`sync` 在 `Γ ≠ []` 时返回 none。

## 4. 满足性、全局良构、生命周期（Satisfaction, Env）

```text
Evidence = DeclEnv → Expr → PropertyId → Prop        -- 由 validation 层提供
Evidence.Monotone ev ⇔ ∀ Δ₁ Δ₂ e p, EnvRefines Δ₁ Δ₂ → ev Δ₁ e p → ev Δ₂ e p

Satisfies ev Θ Δ Γ e S ⇔
    HasType Θ Δ (Grant.of S.expectedType) Γ e S.expectedType
  ∧ ∀ p ∈ S.commitments, ev Δ e p

WellFormedDecl ev Θ Δ Γ h ⇔ ∀ e, h.realization = some e → Satisfies ev Θ Δ Γ e h.interface

GlobalWF ev Θ Δ ⇔ ∀ d h, Δ d = some h → h.id = d ∧ WellFormedDecl ev Θ Δ [] h
```

生命周期步（`DeclRefines ev Θ Δ Γ h h'`，副条件在**当前** Δ 中检查）：

1. `refine`：h 未解决，`InterfaceRefines S S'`
2. `realize`：h 未解决，`Satisfies ev Θ Δ Γ e S`
3. `strengthen`：h 已实现为 e，`InterfaceRefines S S'` 且
   `Satisfies ev Θ Δ Γ e S'`（重新验证）

定理（实现可依赖的保证）：

- `DeclLeq B B'` ⇒ Δ 中每个类型判定在 `Δ[B']`
  中仍成立（只用到 expectedType 不变）
- `GlobalWF` ∧ 一步/多步 `DeclRefines` ∧ `ev` 单调 ⇒ `GlobalWF (Δ[B'])`
- 绑定一个未绑定概念保持 GlobalWF

## 5. 依赖与因果（Dependency）

```text
dependsOn Δ a b     ⇔ b ∈ (realizationOf Δ a).refs
instDependsOn Δ a b ⇔ b ∈ (realizationOf Δ a).instRefs
Acyclic Δ           -- 结构无环（仅 delay-free 片段有意义）
Causal Δ ⇔ ∃ rank R, (∀ d, rank d < R) ∧ ∀ a b, InstDependsOn Δ a b → rank b < rank a
```

- delay-free 片段上 `Causal ⇔ Acyclic`
- `A := delay 0 B; B := A` 是 causal；部分经过 delay 的环不是
- 已知保守点：`A := λx. A x` 被 Causal 拒绝但 `declRef A` 能求值为闭包
- 有限设计上用拓扑排序实现 rank 即可

`Unfolds Δ e e'`：把所有 declRef 展开成闭项（仅 delay-free 片段；在环上无定义）。

## 6. 单域求值（Reactive）

```text
Value ::= bool b | nat n | sem s v | none | some v | clo ρ body | prim p args
Input  = DeclId → Nat → Value          -- 未解决声明在每个 tick 的输入
```

`Prim.arity`、`Prim.compute`（饱和求值；形状不对返回
`nat 0`，类型保证不会发生）、
`applyPrim p args = if args.length = p.arity then p.compute args else prim p args`（部分应用）。

`Ev Δ I t ρ e v`（tick t，局部环境 ρ）：

```text
var        ρ[i] = v
boolLit/natLit/lam(→ clo ρ body)
appClo     f ⇓ clo ρ' body ; a ⇓ va ; (va::ρ') body ⇓ v
appPrim    f ⇓ prim p args ; a ⇓ va ; ⇓ applyPrim p (args ++ [va])
refRealized  realizationOf d = some b ; [] b ⇓ v   at tick t      -- 注意：body 在空环境求值
refInput     realizationOf d = none ; ⇓ I d t
rep        e ⇓ sem s w ; ⇓ w
mk         e ⇓ w ; ⇓ sem s w
prim       ⇓ applyPrim p []
delayZero  t=0 : i ⇓ v at 0
delaySucc  t+1 : e ⇓ v at t
syncZero/syncSucc  单域下 sync 与 delay 同
```

定理：确定性（无条件）；Causal ∧ GlobalWF
∧ 输入良类型 ⇒ 每个声明每个 tick 有值且与类型相关（逻辑关系）。可执行解释器
`evalF Δ I fuel t ρ e : Option Value` 已证 sound。

> 实现注意：Lean 的 `nat` 减法截断到 0、除法向下取整；`q d` 的值也是
> `nat`。工程实现换成有符号/浮点时要记录为偏离。

## 7. 时钟域与多域求值（Clock）

```text
Sched = ClockId → Nat → Bool                     -- 全局 tick 上各域是否激活；在设计之外
prevAct S c t = 最大的 t' < t 使 S c t'，无则 none
Sched.periodic period c t = (t % period c = 0)
Sched.always
ClockEnv Κ = DeclId → Option ClockId             -- none = 域无关的纯映射
```

域判定 `clockedB Κ (c : Option ClockId) e`：

```text
lam/app/rep/mk    递归
declRef d         Κ d = none ∨ Κ d = c
delay i e         c = some c' 时：i, e 都在 some c' 下；c = none → false
sync c' i e       c = some c₀ 时：i 在 some c₀ 下，e 在 some c' 下；c = none → false
其他              true
WellClocked Κ Δ ⇔ ∀ d b, realizationOf Δ d = some b → Clocked Κ (Κ d) b
```

多域求值 `MEv S Δ I c t ρ e v`（c = 当前域）：与 `Ev` 同，除

```text
syncNone   prevAct S c' t = none      ; i ⇓ v (在 c, t)
syncSome   prevAct S c' t = some t'   ; e ⇓ v (在 c', t')
delay      在域 c 下 ≡ sync c
```

定理：`Sched.always`
下 MEv 与 Ev 一致；确定性且同时激活域间无顺序可观测；跨域环永不瞬时（Causal 不变）；Causal
∧ GlobalWF ∧ WellClocked ⇒ 全域每 tick 有值。解释器
`mevalF S Δ I fuel c t ρ e`。

## 8. 物理输出（Output）

```text
OutputSpec = { accepts : Ty, clock : ClockId }
OutputEnv Ω = OutputId → Option OutputSpec       -- 由 deployment 声明
DriveEnv  β = DeclId → Option OutputId            -- drive edge，一次写入

DriveWF Ω Κ Δ β ⇔ ∀ d o, β d = some o →
    ∃ spec, Ω o = some spec ∧ Δ.tyView d = some spec.accepts ∧ Κ d = some spec.clock
SingleDriver β    ⇔ ∀ d₁ d₂ o, β d₁ = some o → β d₂ = some o → d₁ = d₂
Driven β o        ⇔ ∃ d, β d = some o
CompleteOutputs β req ⇔ ∀ o ∈ req, Driven β o

PartialOutputWF   = DriveWF ∧ SingleDriver
ExecutableOutputs = PartialOutputWF ∧ CompleteOutputs

PhysicalOutput S Δ I Ω β o t v ⇔ ∃ d spec, β d = some o ∧ Ω o = some spec ∧ MEv S Δ I spec.clock t [] (declRef d) v
```

- 边不做转换/同步：慢驱动者读快值要在上游
  `sync`；sink 接受表示类型时前面要有显式 `rep` 类型的声明
- `DriveRefines β₁ β₂`、`DriveEnv.bind β d o`（绑到未驱动 sink 是 refinement）
- 组合（优先级/混合/最大/钳位）= 单驱动者里的普通计算

## 9. 硬件验证（Hardware，validation 层）

```text
Capability ::= digitalIn | digitalOut | pwm | analogIn | interrupt
             | i2cSDA | i2cSCL | spiMOSI | spiMISO | spiSCK | spiSS | uartTX | uartRX
ResourceId  = { n : Nat }
Resource    = { id, caps : List Capability, units : List (Capability × Nat) }   -- unit = 背后的定时器/外设编号
Hardware    = { resources : List Resource, shareable : List Capability }        -- 总线可共享，其余独占

RequirementId = { n : Nat }
UnitRel ::= same | distinct
Requirement = { id, cap, fixed : Option ResourceId, group : Option (Nat × UnitRel) }
Assignment  = List (Requirement × ResourceId)
```

有效性：

- `ReqOK H req r`：r 有 cap，且 `fixed` 若有则等于 r
- `Compatible H (a,ra) (b,rb)`：
  - 同资源 ⇒ 同 cap 且 cap ∈ shareable
  - 同 group ⇒ `same`: unit 相等；`distinct`: unit 不等
- `PartialValid H A`：每项 ReqOK 且两两 Compatible
- `ValidFor H R A`：PartialValid 且 A 恰好覆盖 R
- `HardwareSatisfiable H R ⇔ ∃ A, ValidFor H R A`

求解器：`candidates H req` = 支持该 cap 的资源（fixed 时只有那个）；`solveAux`
深度优先、按前缀两两兼容剪枝； `solve H R : Option Assignment`，已证 sound +
complete ⇒ 可判定。

`Hardware.Extends H₁ H₂`（给已有资源加 cap/unit/sharing）保持每个有效分配；删资源、加/强化需求、固定引脚可能不保持。

解释：`Explanation ::= noCapableResource req | blocked req (List (ResourceId × RequirementId))`；
`diagnose H R` 贪心放置报首个死路 —— 仅在 `solve`
返回 none 时有意义，不是最小 unsat core。

管线：`OutputId → DeviceKind → Requirements → solve → Assignment`。设备种类例：

- H-bridge 通道：1 pwm + 1 digitalOut
- I2C 传感器：i2cSDA + i2cSCL，group `same`
- 正交编码器：2 interrupt
- UART：uartTX + uartRX，group `same`

### Arduino Nano 表（可直接搬用）

```text
D0  [digitalIn, digitalOut, uartRX]           uartRX→0
D1  [digitalIn, digitalOut, uartTX]           uartTX→0
D2  [digitalIn, digitalOut, interrupt]
D3  [digitalIn, digitalOut, pwm, interrupt]   pwm→2
D4  [digitalIn, digitalOut]
D5  [digitalIn, digitalOut, pwm]              pwm→0
D6  [digitalIn, digitalOut, pwm]              pwm→0
D7, D8 [digitalIn, digitalOut]
D9  [digitalIn, digitalOut, pwm]              pwm→1
D10 [digitalIn, digitalOut, pwm, spiSS]       pwm→1, spiSS→0
D11 [digitalIn, digitalOut, pwm, spiMOSI]     pwm→2, spiMOSI→0
D12 [digitalIn, digitalOut, spiMISO]          spiMISO→0
D13 [digitalIn, digitalOut, spiSCK]           spiSCK→0
A0–A3 [analogIn, digitalIn, digitalOut]
A4  [analogIn, digitalIn, digitalOut, i2cSDA] i2cSDA→0
A5  [analogIn, digitalIn, digitalOut, i2cSCL] i2cSCL→0
A6, A7 [analogIn]
shareable = [i2cSDA, i2cSCL]
```

已验证的测试用例（实现的回归测试）：

- 4×H-bridge + IMU：SAT，`M1→D3/D0, M2→D5/D1, M3→D6/D2, M4→D9/D4, IMU→A4/A5`
- 7×pwm：UNSAT（6 个 PWM 引脚）；同需求在 big board（+D40–D45, timers 3/4/5）SAT
- 2×interrupt + 6×pwm：UNSAT（D3 既是唯一第二个中断脚又是 PWM 脚）
- 两个 pwm 固定到同一引脚：拒绝
- 两个 I2C 传感器都在 A4/A5：接受（总线共享）
- 4×pwm group `distinct`：Nano 只有 3 个定时器 → UNSAT
- TX/RX group `same`：保持在同一 UART

## 10. 全局良构条件汇总（"可执行设计"要同时满足）

1. `GlobalWF ev Θ Δ`（每个 realization 满足自己的 interface）
2. `ConceptEnv.WF Θ`
3. `Causal Δ`
4. `WellClocked Κ Δ`
5. `DriveWF Ω Κ Δ β ∧ SingleDriver β`
6. 可执行时另加 `CompleteOutputs β required`
7. 硬件：`solve H (requirementsOf Ω deviceKinds) = some A`（不进内核，每次重算）

对应工作区状态：1–2 → type-valid；3 → temporally valid；4 →
clock-consistent；5–6 → output-complete；7 → hardware-feasible。
