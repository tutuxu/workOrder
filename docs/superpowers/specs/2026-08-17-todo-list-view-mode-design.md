# 代办列表视图切换（列表 / 卡片）设计

**日期：** 2026-08-17  
**状态：** 已批准（待用户确认书面 spec）  
**范围：** 前端 `WorkOrderList` 展示形态；不改后端 API / Rust settings

---

## 1. 目标

在代办列表页增加控件，在 **列表** 与 **卡片（多列网格）** 两种展示形式之间切换；偏好跨会话记住。卡片模式下通过右键菜单进入多选，并用「取消多选」退出。

## 2. 需求摘要

| 项 | 决定 |
|----|------|
| 卡片布局 | 响应式多列网格 |
| 卡片字段 | 标题、标签、状态、计划完成时间、最后更新 |
| 列表模式 | 保持现有行式布局、表头、常驻勾选、拖拽 |
| 卡片拖拽 | 保留（拖拽把手） |
| 卡片多选 | 默认不显示勾选；右键菜单「多选」进入 |
| 右键菜单范围 | **仅卡片模式** |
| 退出多选 | **仅**工具栏「取消多选」 |
| 偏好持久化 | `localStorage`，键名 `workOrder.listViewMode` |
| 实现方式 | 同页双模板（方案 1），不拆子组件仓库结构、不改 settings API |

## 3. 架构

### 3.1 状态

在 `WorkOrderList.vue`（或紧邻的小型 composable，若文件过长再抽）中维护：

| 状态 | 类型 | 说明 |
|------|------|------|
| `viewMode` | `'list' \| 'card'` | 当前展示形式；读写 localStorage |
| `cardMultiSelect` | `boolean` | 卡片是否处于多选模式；仅 `viewMode === 'card'` 时有意义 |
| `selectedIds` | `Set<number>` | 与现有多选集合共用 |

规则：

- `viewMode === 'list'` 时，`cardMultiSelect` 视为无效；列表始终显示勾选。
- 进入卡片多选：`cardMultiSelect = true`，并将右键目标 id 加入 `selectedIds`。
- 点击「取消多选」：`cardMultiSelect = false`，并 `clearSelection()`。
- 卡片非多选模式下点击卡片主体：打开详情（与现有一致）；多选模式下点击勾选切换选中，点击主体是否打开详情：保持可打开详情（勾选 `@click.stop`）。

### 3.2 持久化

- 键：`workOrder.listViewMode`
- 合法值：`list` | `card`
- 读取失败或非法值时默认 `list`
- 用户切换分段控件时立即写入
- 不写入 Rust settings，不参与 ZIP 备份

### 3.3 文件改动预期

| 文件 | 改动 |
|------|------|
| `src/views/WorkOrderList.vue` | 视图切换、卡片模板、右键菜单、多选工具栏按钮 |
| `src/styles/main.css` | 卡片网格与卡片项样式 |
| （可选）`src/composables/useListViewMode.ts` | 若希望把 localStorage 读写从视图中抽出 |

不改 `bindings` / settings API / 后端。

## 4. UI 行为

### 4.1 视图切换控件

- 位置：工具栏第一行，「设置」按钮与搜索框之间
- 组件：`n-radio-group`（`type="button"`）或等价分段控件
- 选项文案：「列表」「卡片」
- 切换立即更新 `viewMode` 并持久化

### 4.2 列表模式

与现状一致：

- 表头 + 行式 `list-row`
- 常驻勾选、全选、删除选中
- `VueDraggable` + `.drag-handle`
- 状态色 / 逾期左边框沿用现有工具函数

### 4.3 卡片模式

- 容器：CSS Grid，例如 `repeat(auto-fill, minmax(240px, 1fr))`（具体 minmax 实现时可微调）
- 每张卡：竖排信息块；状态背景色 / 逾期样式与列表语义一致
- 拖拽：`VueDraggable` 作用于卡片网格；把手可放在卡片角部，类名仍用 `.drag-handle`
- 非多选：不渲染勾选框
- 多选中：每张卡显示勾选；工具栏「删除选中」可用

### 4.4 卡片右键菜单

使用 Naive UI `n-dropdown`（`trigger="manual"`）或等价上下文菜单，在卡片上 `@contextmenu.prevent` 打开。

菜单项：

| 项 | 行为 |
|----|------|
| 打开 | `openExisting(item)` |
| 多选 | `cardMultiSelect = true`；将当前 id 加入 `selectedIds` |
| 删除 | **非多选**：确认后删除当前右键条目。**多选中且 `selectedCount > 0`**：确认后删除全部选中（label 显示「删除选中」）。**多选中但选中为空**：该项禁用 |

说明：未进入多选时菜单显示「删除」。

### 4.5 多选工具栏（卡片）

当 `viewMode === 'card' && cardMultiSelect` 时，工具栏显示：

- **取消多选** — 唯一退出多选方式：`cardMultiSelect = false` + `clearSelection()`
- 现有 **删除选中** 在 `selectedCount > 0` 时可用（与列表共用逻辑）

不提供 Esc、再次右键「退出」、或切换视图作为正式退出路径。若用户从卡片切回列表：`cardMultiSelect` 置 `false`，但 **保留** `selectedIds`，以便列表勾选与删除选中衔接。若用户从列表切到卡片且 `selectedIds` 非空：自动 `cardMultiSelect = true` 并显示勾选；若为空则保持非多选。

### 4.6 视图切换与选中

| 切换 | 选中集合 | 卡片多选标志 |
|------|----------|--------------|
| 列表 → 卡片（有选中） | 保留 | `true` |
| 列表 → 卡片（无选中） | 空 | `false` |
| 卡片 → 列表 | 保留 | `false`（列表用常驻勾选展示） |

筛选 / 搜索 / 刷新后：不可见 id 从 `selectedIds` 移除（沿用现有 `syncLocalItems` 逻辑）。

### 4.7 空状态

两种视图均使用现有 `n-empty` 与 `emptyDescription`。

## 5. 错误与边界

- localStorage 不可用：内存中切换仍可用，不抛错打断列表加载
- 删除失败：沿用现有 `message.error`；不强制退出多选
- 右键菜单打开时若列表正在 loading：允许打开，操作仍走现有 API 错误处理

## 6. 测试要点

- 切换列表/卡片，刷新页面后偏好恢复
- 卡片非多选：无勾选；右键「多选」后出现勾选且当前项已选
- 「取消多选」清除选中并隐藏勾选
- 卡片多选删除 / 单条删除行为符合 §4.4
- 两种视图拖拽排序后顺序持久（现有 `reorder`）
- 列表多选 → 切卡片仍多选；卡片多选 →「取消多选」后再切列表无残留选中
- 筛选后不可见项从选中集合移除

## 7. 非目标

- 不改设置页、不备份视图偏好
- 不在列表模式增加右键菜单
- 不新增除「取消多选」以外的退出多选入口
- 不做卡片密度/列数用户自定义
- 卡片模式不提供「全选」（列表模式保留现有全选）
