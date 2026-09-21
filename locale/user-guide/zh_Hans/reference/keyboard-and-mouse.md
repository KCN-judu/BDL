<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/reference/keyboard-and-mouse.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/reference/keyboard-and-mouse.md) · 简体中文 · [日本語](../../ja/reference/keyboard-and-mouse.md)
>
> 本页尚未完全翻译；未翻译的段落以英文显示。

# 键盘与鼠标

快捷键按 macOS 列出。在 Windows 上界面中显示相同的按键并以 Ctrl 代替，但绑定尚未在那里生效——请使用工具栏和菜单。

## 应用程序

| 按键 | 作用 |
| --- | --- |
| ⌘S | 保存项目——原样保存一切，包括未完成的公式和不能构建的文本 |
| ⌘W | 关闭项目——已编辑时询问 _是否保存更改？_ |
| ⌘Q | 退出——先问同一个问题 |
| ⌘Z / ⇧⌘Z | 撤销 / 重做——设计编辑和行为组编辑，同一份历史 |
| ⌘1 · ⌘2 · ⌘3 · ⌘4 | 设计 · 仿真 · 部署 · 监视 |

项目管理器上的 _新建项目…_（⌘N）和 _打开项目…_（⌘O）显示了快捷键，但目前要通过点击来使用。

## 画布

| 操作 | 结果 |
| --- | --- |
| 从左向右拖动空白画布 | 选中完全落在框内的节点（窗口） |
| 从右向左拖动空白画布 | 选中框内或被框碰到的节点（交叉） |
| ⌘-拖动 / ⇧-拖动出一个矩形 | 把框内节点加入选区 / 移出选区 |
| 中键拖动 · Space + 拖动 · 触控板双指 | 平移 |
| 滚轮 · 捏合 · ⌘ + 双指 | 以指针为中心缩放 |
| Home · ⌘0 | 显示全部 |
| 点击 | 选中一个 |
| ⌘-点击（Windows 和 Linux 上为 Ctrl） | 加入 / 移出选区 |
| ⇧-点击 | 选中从当前对象到它的连接链——当这样的链恰有一条时 |
| ⌘A | 选中视图中的所有节点 |
| Esc | 取消进行中的操作；再按一次则清空选区 |
| ← → ↑ ↓（⇧：一点） | 微移选区 |
| 拖动节点 | 移动选区（仅布局） |
| 把一个块的输出插槽拖到一个输出上 | 使它成为驱动者 |
| 把一个块的输出插槽拖到没有公式的块上、拖到 `?` 插槽上，或拖到有 `?` 插槽的映射块上 | 公式得到这个块的名字（一次文本编辑） |
| 从插口拖到插口 | 连线（一个绑定、一个驱动） |
| 把被驱动输出的插槽，或映射块的输入插槽，拖离 → 空白画布 | 断开（读取连线：名字变成 `?`） |
| ⌫ / Delete | 删除所选（或断开所选的连线） |
| 双击一个块或它的映射块 | 就地重命名这个块 |
| 双击实例 | 打开它的组件来源 |
| 双击行为标题 | 重命名 |
| 右键点击 · Control-点击 | 上下文菜单 |
| 把库中的一行拖到画布上 | 插入概念 |

## 公式字段

| 按键 | 作用 |
| --- | --- |
| ⌘↩ | 添加 / 保存定义 |
| Esc | 还原草稿；补全打开时先关闭它 |
| Return | 换行 |
| ⌃Space | completion (in the Formula view it also opens as you type a name) |
| ↑ / ↓ · Return / Tab | 在补全列表中移动 · 接受 |

## Formula view

| 按键 | 作用 |
| --- | --- |
| ← → | the previous / next place — out of a denominator, past a parenthesis, into the next part |
| ↑ ↓ | the row above / below (a numerator from its denominator, a branch from the next) |
| Home / End | the ends of the enclosing part; again, the ends of the formula |
| Tab / ⇧Tab | the next / previous empty slot |
| ) , | leave the parentheses / the next argument |
| letters, digits | type into the slot or the name or number at the caret; a space after a number starts its unit |
| + − \* / < > = & \| | the operator after the part at the caret, with a slot for the other side |
| ! | negate the part |
| ( | apply the name before the caret (`clamp` → `clamp(?, ?, ?)`), or group a slot |
| ⌫ / ⌦ | a character, or the whole part beside the caret |
| 点击 | place the caret and select the part |

## 代码视图

| 按键 | 作用 |
| --- | --- |
| ⌃Space | 在光标处补全 |
| ↑ / ↓ · Return / Tab · Esc | 在补全列表中移动 · 接受 · 关闭 |
| 把指针停在名字上 | 它的卡片；输入或移开即隐藏 |
| ⌘-点击名字 · F12 | 跳到它声明的位置（会打开另一个文件） |
| ⇧F12 | 列出提到它的每一处；Esc 或 × 关闭列表 |
| ⌥⇧F · _格式化_ | 把文件排成规范布局，作为一次编辑 |

## 内联重命名（画布、库行）

| 按键 | 作用 |
| --- | --- |
| Return | 确认 |
| Esc | 保留旧名字 |
| 点击别处 | 确认已输入的内容 |

## 面板

Return 提交，Esc 取消；取消按钮在主按钮左侧。
