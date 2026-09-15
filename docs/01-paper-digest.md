# BDL 论文精读摘要（面向工程实现）

来源：`reference/paper/paper.md`（*BDL: A Behavior Design Language*，2026-09-15 修订版），
以及形式化开发 [KCN-judu/BDL_FV](https://github.com/KCN-judu/BDL_FV)（Lean 4，Phase 0–7，无 `sorry`）。

本文只记录**对工程实现有约束力**的内容；论证过程、反例细节见 BDL_FV 的 `REPORT.md` / `DESIGN_DECISIONS.md` / `MINIMALITY.md`。
精确的数据结构与判定规则见 [02-kernel-spec.md](02-kernel-spec.md)。

---

## 1. 一句话

BDL 是给工业设计师用的产品行为设计语言：**带类型的语义关系**（`?f : Tilt -> Brightness`）是一等设计对象，
关系可以在没有实现体时就合法存在（signature-first），一切设计师可见的构造（时间修饰、上下文、事件策略、输出选择）
都被消解（elaborate）到一个很小的形式内核上。

## 2. 三层架构（论文 §Architecture）

```
Surface（设计师可见）   语义属性 · Mapping Block · 时间修饰符 · 上下文(StateHandler) · 设备种类 · 单位 · 显示名
        │ elaboration ↓            diagnostics ↑
Kernel（形式对象）       声明环境 Δ + 类型判定 + tick 求值 + 域判定 + 全局良构条件
        │ commitments/evidence ↓    target board ↓
Validation（内核之外）   commitment 的证据 · 硬件可行性求解器 · 数值约束（未建模）
```

- 消解是**单向**的：内核永远不需要恢复 surface 结构。
- 类型判定只读声明的 **type view**（expectedType）与概念的 **representation view**（Θ）。不读 realization、commitment、evidence、clock、binding。
- Validation 分两种证据，**永不合并**：
  - 随 refinement 存活的（monotone evidence，如组合式单调性证明）
  - 每次改动后重算的（board 可行性）

## 3. 内核构造清单（实现必须提供的）

| 构造 | 内核地位 | 要点 |
|---|---|---|
| `DesignDecl = ⟨id, interface, realization?⟩` | K | id 稳定；interface = 冻结的 expectedType + 单调增长的 commitments；realization 一次写入 |
| `DeclEnv : DeclId → Option DesignDecl` | K | 设计 = 环境；未解决声明 = `realization = none`，无其他区分 |
| `declRef d` 类型规则只查 `tyView` | K | 客户稳定性（refinement 不破坏客户）的充分且必要条件 |
| `Ty.sem SemanticId` | K | 名义语义类型；`Tilt ≠ MotorAngle` 即使表示相同；跨概念 = 普通箭头声明，不是 cast |
| `ConceptEnv Θ : SemanticId → Option Ty` | K | 概念到表示的一次写入绑定；表示类型须 sem-free 且为 data（无函数） |
| `rep e` / `mk s e` + Grant | K | `rep` 处处可用；`mk s` 仅在签名结果位置宣告 `sem s` 的声明的 realization 里可用（`Grant.of τ`） |
| `Ty.q Dim` + 算术原语类型 | K | 量纲代数全部在 `Prim.ty` 里；无专门量纲规则；单位是 surface（线性缩放字面量） |
| `delay init e` | K | 唯一单域时间原语；只能 top-level（不在 λ 下）、只能 data 类型；每个 delay 必带显式初值 |
| `Causal Δ`（瞬时依赖图无环） | K | 取代结构无环；经 delay 的环合法；有 rank 见证 |
| `ClockId` + `ClockEnv Κ` + `Clocked` 判定 | K | 域是名义身份不是速率；速率是 validation 数据；域不进类型 |
| `sync src init e` | K | 唯一跨域传输原语；读 src 域**严格早于**当前 tick 的最后一次激活；`delay ≡ sync own` |
| `OutputId` + `OutputSpec` + `DriveEnv β` | K | 物理 sink 名义身份；一次写入的 drive edge；`DriveWF`：驱动者类型 **等于** sink 接受类型且 clock 相同 |
| `SingleDriver β` / `CompleteOutputs` | K | 每个 sink 至多一个驱动者；可执行设计要求每个必需 sink 都被驱动 |
| 硬件资源/需求/求解器 | V | 有限 CSP（一元+二元约束）；DFS 求解器 sound+complete；`diagnose` 给出首个死路 |

**已被移除（不要实现为内核）**：`Signal τ` 类型、`Event τ` 类型、clock-in-type、effect rows、action requests、
per-context policies / 运行时仲裁、五阶段 tick with resolve step。

## 4. Refinement vs Edit（交互模型的组织原则）

Refinement（保持一切已建立的东西，客户不需要重查）：
1. 给未解决声明的 interface 增加 commitment
2. 给未解决声明写入满足 interface 的 realization
3. 强化已实现声明的 interface（需重新验证 body）
4. 绑定一个未绑定的概念到表示
5. 把未绑定声明绑到未被驱动的 sink

Edit（合法，但要重开传递依赖者的验证）：
- 改 expectedType（保持 id）→ 所有客户类型破坏
- 删 commitment → 类型检查沉默，但客户的 commitment 失去支撑
- 替换/分离 realization → 查过 body 的证据作废
- 新 id 替换旧 id → 悬空引用
- 改/赋 clock domain → 客户域判定破坏
- 改 output binding 目标 → completeness / single-driver 可能破坏
- 改概念表示、加/删 delay、改初值、改 sink 接受类型、重命名 sink、分离 drive edge

## 5. 派生算子（surface → 声明形状；每行都是自引用的 self-delayed 环）

| Surface | 声明形状 |
|---|---|
| `previous x` | `delay init x` |
| `previous x`（无初值） | `delay none (some x)`，缺失推给消费者 |
| `hold init e` | `getD e (delay init self)` |
| `count e` | `ite (isSome e) (1 + delay 0 self) (delay 0 self)` |
| `since e` | `ite (isSome e) 0 (1 + delay 0 self)` |
| `once e` | `delay false self ∨ isSome e` |
| `every n` | 基于 delay 的模 n 计数器 |
| `rise b` | `b ∧ ¬ delay false b`，作为 optional Boolean |
| `after e by d` / `p for d` / `while p` / `until e` | 上述 + 比较 + 激活的组合（论文未单独执行） |

事件（occurrence）在单域内 = `opt τ` 流。跨域事件传输需要 **window model**：
源侧累积 log + 目的侧 cursor（`sync` 一个累加器 + `delay` 一个游标）；`latest`/`count` 等策略是 window 的函数；
缓冲区上界是 validation 义务。内核目前没有 list 类型，所以 buffer 归约只是语义层结果——**实现时需要自己补 list**。

## 6. 上下文（StateHandler）消解（仅覆盖已测试的情况）

一个上下文消解为：
- 激活声明（Boolean）
- 进入声明 = 激活的上升沿 `rise`
- 本地时间状态 = 被激活门控、在进入时重置的 delayed cell
- 非激活时贡献默认值
- 多上下文竞争同一输出 → 在**唯一驱动者声明**里的普通条件表达式

已测试：条件激活、进入、进入重置状态、非激活默认、事件锁存激活 + exit-wins、上下文本地输出选择、嵌套选择+输出。
**未测试**：带自己 clock domain 的上下文、跨独立时钟上下文的嵌套 —— 工具应明确拒绝/报告，不要静默消解。

## 7. 消解器 pass（论文 §Elaboration，尚未有实现）

1. **Name & signature resolution** — 语义属性→SemanticId，Mapping 签名→DeclInterface，上下文、设备种类、单位、导入组件
2. **Formula elaboration** — 在 `Grant.of(签名)` 下检查定义；标量公式自动包 `mk_Brightness(...)`；单位→缩放字面量；curve/example/component 都消解到同一 realization 形式
3. **Temporal lowering** — 时间短语→§5 的形状；上下文→§6；可重用有状态组件在每个使用点实例化为新声明
4. **Domain assignment** — 记录 Κ，检查 `Clocked`；跨域引用无 transport 时在引用处报错，给两种解法（加 sync + 初值 / 把读者移到源域）
5. **Output binding** — 记录 β，检查类型/clock 相等、single-driver、（可执行时）completeness；多上下文驱动一输出时生成一个 selector 驱动者
6. **Hardware validation** — 设备种类→requirements → solve → assignment 或 explanation
7. **Normalization & erasure** — 擦除 sem/dim/domain（已证 sound）；`rep(mk s e) ⇝ e`；展平后在 universal grant 下检查；边界处（supplied block、设备绑定、生成代码公共接口）保留 wrapper

## 8. 工作区状态（设计师看到的诊断层级）

| 状态 | 已确定的内容 | 显示位置 |
|---|---|---|
| declared | 关系已命名、已定型；他人可依赖；无定义 | canvas |
| defined | 附加了公式/曲线/样例/组件 | inspector |
| type-valid | 定义产出签名承诺的类型；声明的属性成立 | inspector |
| temporally valid | 每个 held/delayed 值有首 tick 值；无瞬时环 | canvas |
| clock-consistent | 每次跨域读取都有 transport 和初值 | canvas |
| output-complete | 每个必需物理输出恰有一个最终目标 | outputs |
| hardware-feasible | 所选板能承载每个派生需求（**唯一依赖设计以外东西的状态**，每次改动重算） | deployment |

诊断措辞原则：用产品语言不用内核术语。例：
- 不说 "single-driver violation"，说 "this output already has a final driver, `dimByTilt`; combine the two brightness values before connecting the output"
- 跨域：说两个值在不同域更新，问读者应如何看源（上次值+初值 / 移到源域）
- 硬件冲突：在第七个 PWM 通道上报 "需要 7 条 PWM，只有 6 条"，并列出每个 PWM 引脚被谁占用

由声明（supplied block 作者断言）而非分析得出的属性，显示上必须与工具计算出的属性**区分开**。

## 9. Supplied computation blocks（宿主语言写的组件）

- 纯函数或有状态 transducer；**不能驱动输出**
- 必须声明：确定性、良类型输入上的全域性、输出范围、下游依赖的属性（如单调性）、最坏状态大小、设计更新率 —— 这些是 validation 义务
- 无可重用有状态组件原语；每个使用点由消解器实例化新声明（因为 delay 不能在 binder 下）
- 标准库走同一接口

## 10. 运行场景（可作为第一个集成测试）

**倾斜台灯**：`Tilt`、`Brightness`、`Temperature`、`Held`
- `dimByTilt : Tilt -> Brightness`（先声明后定义，曲线 `clamp(0.2 + 0.8·θ/60°, 0, 1)`）
- `hold while not Held`
- `Warm` 条件 + `every 1 s` 脉冲；`Critical` 强制 heater 为 0
- 两个行为竞争 light → `lampTarget : Brightness` 单驱动者
- 两个域：interaction（Tilt/Held，50 Hz）、ambient（Temperature，1 Hz）；`heaterTarget` 跨域读 → sync + 初值
- 板：Arduino Nano；light=PWM，heater=数字输出，IMU=I2C，温度=模拟输入

**四电机 + IMU**（SAT）：`M1 -> D3/D0, M2 -> D5/D1, M3 -> D6/D2, M4 -> D9/D4, IMU -> A4/A5`
**七路 PWM**（UNSAT on Nano，SAT on big board）；**2 interrupt + 6 PWM**（计数够但 UNSAT，D3 冲突）。

## 11. 论文明确留开的问题（实现时需自行决定，见 03-open-questions.md）

- 多候选定义、一个 active 的 surface 便利如何归约到 write-once realization
- 接口级引用（commitment 提到其他声明）未建模
- realization 是否可把 grant 委托给高阶参数
- 仿射单位（℃ vs K）
- 事件缓冲需要对象语言的 list 类型
- unfolding 与 tick 求值一致性只证了一阶
- lambda-guarded cycle 上 `Causal` 是保守的
- 数值电气/时序约束、求和约束（非二元）
- `diagnose` 是首个死路不是最小 unsat core
