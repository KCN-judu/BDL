# 实现前需要决定的事项

论文与 Lean 开发把内核钉死了，但**消解器、编辑器、固件路径都没有实现**（论文原话："no elaborator, editor, firmware generator, or user study has yet been built"）。
下面是工程实现必须自己拍板的点，按层分组。每一项决定后请记录到 `docs/decisions/` 并在此处打钩。

## A. 语言 / 运行时选型（等待用户指示）

- [ ] 实现语言与目标平台（宿主语言同时也是 "supplied computation block" 的语言）
- [ ] 数值表示：Lean 用 `Nat`（截断减法、整除）。工程实现大概率要 `f64`/定点；擦除后生成代码的数值类型
- [ ] 内核数据结构：函数环境 → map；`DeclId` 等是否保留为 newtype over integer
- [ ] 交付形态：库 + CLI？语言服务器？带 canvas 的编辑器？固件生成器？先做哪个

## B. 内核层（Lean 已固定语义，实现只需选表示）

- [ ] `PropertyId` 的具体集合（Lean 里是抽象标签；需要至少：monotone、range、determinism、totality、worstCaseState、designRate）
- [ ] `Evidence` 关系的具体实现，且要能**区分**两类证据：单调（存活 refinement）vs 目标相关（每次重算）
- [ ] `Dim` 是否从 3 维扩展到完整 SI 7 维（论文明确说 3 维只是"够测试抽象"）
- [ ] 对象语言是否加 `list` 类型 + 少量 list 算子（事件 window buffer 需要；论文说"内核目前没有"）
- [ ] `Causal` 对 lambda-guarded cycle 的保守性是否接受

## C. Surface 层（论文只给了词汇，没给语法）

- [ ] 文本语法 vs 只有图形/JSON 表示。至少需要一个可序列化的 surface AST
- [ ] Mapping Block 定义形式：formula / curve / example-fit / component 各自的表示与到 `Expr` 的消解
- [ ] 时间修饰符完整列表与消解（`for`, `after ... by`, `while`, `until` 论文未单独执行，需自己定形状）
- [ ] 上下文（StateHandler）的 surface 表示；带自有 clock 的上下文——论文说未测试，工具应拒绝
- [ ] "多候选定义、一个 active"——论文留开；建议实现为 surface 存多个候选、内核只见 active 那个，切换 = edit
- [ ] 单位系统：线性缩放；仿射单位（℃）论文未建模，是否拒绝或以 surface 方案处理
- [ ] 设备种类（DeviceKind）目录与它到 Requirements 的映射（H-bridge、I2C、encoder、UART 已有例子）
- [ ] 板描述格式（Nano 表已有；其他板需要数据格式）

## D. 消解器诊断（论文给了措辞原则，没给格式）

- [ ] 诊断的结构化格式：位置（canvas 对象 / wire / output / deployment requirement）、产品语言文案、可选修复
- [ ] 工作区七种状态如何计算并归属到对象
- [ ] "explanation view"：展示消解结果（held value → delayed cell + gate 等）
- [ ] 声明式证据（组件作者断言）与计算式证据在 UI/输出里的区分方式

## E. 后端

- [ ] 擦除 + 展平后的目标代码（Arduino C++? Rust embedded? 仿真器?）
- [ ] 调度器实现：全局 base tick + 周期表；`sync` 的"严格早于"语义要保持
- [ ] 数值偏离记录：浮点非结合性与符号规范化（论文要求作为 obligation 记录）
- [ ] supplied block 的 FFI 边界与其 validation 义务如何声明

## F. 测试策略

- [ ] 与 BDL_FV 对齐的回归测试：至少复现论文报告的每条 trace / SAT / UNSAT 结果（见 02-kernel-spec §9）
- [ ] 台灯场景作为端到端集成测试
- [ ] 是否做 Lean ↔ 实现的差分测试（从 Lean 导出用例）
