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
| 拖动空白画布 | 平移 |
| 滚轮 / 捏合 | 以指针为中心缩放 |
| Home · ⌘0 | 显示全部 |
| 点击 | 选中 |
| ⇧-点击 | 加入 / 移出多选 |
| 在空白画布上 ⇧-拖动 | 框选 |
| 拖动节点 | 移动（仅布局） |
| 从插口拖到插口 | 连线 |
| 把已连接的输入插口拖开 → 空白画布 | 断开 |
| ⌫ / Delete | 删除选中项（或断开选中的绑定连线） |
| 双击概念或关系 | 原地重命名 |
| 双击实例 | 打开它的组件来源 |
| 双击行为标题 | 重命名 |
| 右键点击 | 上下文菜单 |
| 把库中的一行拖到画布上 | 插入概念 |

## 公式字段

| 按键 | 作用 |
| --- | --- |
| ⌘↩ | 添加 / 保存定义 |
| Esc | 还原草稿；补全打开时先关闭它 |
| Return | 换行 |
| ⌃Space | 补全 |
| ↑ / ↓ · Return / Tab | 在补全列表中移动 · 接受 |

## Code view

| 按键 | 作用 |
| --- | --- |
| ⌃Space | completion at the caret |
| ↑ / ↓ · Return / Tab · Esc | move in the completion list · accept · close it |
| rest the pointer on a name | its card; typing or moving away hides it |
| ⌘-click a name · F12 | go to where it is declared (another file opens) |
| ⇧F12 | list every place that names it; Esc or × closes the list |
| ⌥⇧F · _Format_ | lay the file out the canonical way, as one edit |

## 内联重命名（画布、库行）

| 按键 | 作用 |
| --- | --- |
| Return | 确认 |
| Esc | 保留旧名字 |
| 点击别处 | 确认已输入的内容 |

## 面板

Return 提交，Esc 取消；取消按钮在主按钮左侧。
