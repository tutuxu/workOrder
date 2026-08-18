# 代办事项回收站设计

**日期：** 2026-08-18  
**状态：** 已批准（待用户确认书面 spec）  
**范围：** 仅代办事项（work order）的软删除 / 还原 / 彻底删除；进度日志与附件不单独进回收站

---

## 1. 目标

列表与详情中的删除默认把事项移入回收站，而不是立刻销毁。用户可从工具栏打开独立回收站页，多选后还原或彻底删除。回收站内可打开只读详情。不自动清空。

## 2. 需求摘要

| 项 | 决定 |
|----|------|
| 数据方案 | `work_order.deleted_at`（方案 1） |
| 主清单默认删除 | 移入回收站 |
| 主清单彻底删除 | 同一确认框内的次要操作；卡片右键也提供 |
| 回收站入口 | 主清单工具栏独立按钮，打开 `n-modal`（与设置同级） |
| 回收站选择 | 多选 + 表头全选；不做「清空全部 / 全部还原」 |
| 自动清理 | 无；一直保留直到手动彻底删除 |
| 回收站展示 | 仅列表，无卡片 / 筛选 / 搜索 |
| 回收站详情 | 只读（可看标题、描述、进度、附件，不可改） |
| 关联数据 | 进回收站时保留进度、标签、附件文件；彻底删除才级联硬删 |
| 还原后排序 | 保留原 `priority`，不改 `updated_at` |

## 3. 架构

### 3.1 数据

`work_order` 增加可空列：

```sql
deleted_at TIMESTAMP
```

- `NULL`：在主清单  
- 非空：在回收站，值为移入时间（UTC，与现有时间列同一套读写）

已有行迁移后均为 `NULL`。新建库在 `schema.sql` 中带上该列；已有库走增量迁移 `migrate_work_order_deleted_at`，并建索引 `idx_work_order_deleted_at`。

`WorkOrder` 模型增加 `deleted_at: Option<NaiveDateTime>`（前端 `deletedAt`）。主清单返回的事项该字段为 `null`。

ZIP 备份拷贝整个 SQLite，无需单独备份回收站。

### 3.2 查询规则

| 场景 | 规则 |
|------|------|
| `list_work_orders` / `find_by_filters` | `deleted_at IS NULL` |
| `list_trashed_work_orders` | `deleted_at IS NOT NULL`，按 `deleted_at DESC` |
| `get_work_order` | 不区分是否在回收站，供只读详情 |
| `update_work_order` / `update_priorities` | 目标已删除则 `Validation` 拒绝 |
| 进度日志增删改、附件增删 | 所属事项已删除则 `Validation` 拒绝 |
| 进度日志列表、附件列表 | 允许读取回收站事项（只读详情） |
| 创建事项 | `deleted_at` 为 `NULL` |
| `next_priority` | 仍按全表 `MAX(priority)`，不强制排除回收站 |

移入 / 还原只写 `deleted_at`，不改 `updated_at`、`priority`、进度、标签、附件。

### 3.3 文件改动预期

| 文件 | 改动 |
|------|------|
| `src-tauri/src/db/schema.sql` | `work_order.deleted_at` |
| `src-tauri/src/db/migrate.rs` | 加列 + 索引 |
| `src-tauri/src/db/connection.rs` | 调用新迁移 |
| `src-tauri/src/models/work_order.rs` | `deleted_at` |
| `src-tauri/src/services/work_order_service.rs` | 筛选排除已删除；trash / restore / 批量硬删 |
| `src-tauri/src/services/progress_log_service.rs` | 写操作拒绝回收站事项 |
| `src-tauri/src/services/attachment_service.rs` | 新增/删除附件前：owner 为工单则该工单未删除；owner 为进度日志则其所属工单未删除 |
| `src-tauri/src/commands/work_order.rs` | 新 command；`delete_work_order` 保持彻底删除 |
| `src-tauri/src/lib.rs` | 注册 command |
| `src/bindings.ts` | specta 重新导出 |
| `src/api/workOrders.ts` | 封装新 API |
| `src/views/WorkOrderList.vue` | 回收站入口、删除确认改为双路径 |
| `src/views/WorkOrderDetail.vue` | 只读模式；删除确认双路径 |
| `src/views/RecycleBin.vue` | 新：回收站弹层 |
| `src/App.vue` | 挂载回收站弹层 |
| `src/styles/main.css` | 回收站列表样式（复用 `list-row` 为主） |

不改 settings API、不改 ZIP 备份逻辑。

## 4. 后端 API

### 4.1 保留

`delete_work_order(id)`：**彻底删除**单条（含已在回收站的事项）。与当前硬删相同：删附件文件、该事项下全部进度日志及附件，再删 `work_order` 行（`work_order_tag` 依赖现有 `ON DELETE CASCADE`）。须能定位已删除行（`get_required` 不要求 `deleted_at IS NULL`）。

### 4.2 新增

| 命令 | 行为 |
|------|------|
| `list_trashed_work_orders()` | 返回回收站事项，含 `deletedAt`，按删除时间新到旧 |
| `trash_work_orders(ids)` | 事务内：对 `deleted_at IS NULL` 的行写入当前时间；已在回收站的 id 跳过 |
| `restore_work_orders(ids)` | 事务内：将 `deleted_at` 置 `NULL`；不在回收站的 id 跳过 |
| `permanently_delete_work_orders(ids)` | 事务内：对存在的 id 逐条走与 `delete_work_order` 相同的硬删；不存在的 id 跳过 |

空 `ids`：直接成功，不报错。事务失败则全部回滚。

拒绝写回收站事项时，使用现有 `ServiceError::Validation`，文案固定为 `Work order is in recycle bin`，前端展示为「该事项在回收站，无法修改」。

## 5. UI 行为

### 5.1 入口

主清单工具栏第一行，「设置」按钮旁增加「回收站」。点击后 `App.vue` 打开 `RecycleBin` 弹层（`n-modal`，preset card，宽度约 840px，内容可滚动）。回收站打开时主清单不可操作（与设置弹层相同）。关闭回收站后调用列表 `reload()`。

不新增打开回收站的快捷键。现有 `list.deleteSelected` 改为走「移入回收站」确认框（见 §5.2）。

### 5.2 主清单 / 详情：删除双路径

工具栏「删除选中」、详情页「删除」、快捷键 `list.deleteSelected` 不再使用仅含「删除 / 取消」的 `n-popconfirm`，改为 `n-dialog`：

- 标题：确认删除  
- 正文：`确定将选中的 N 条移入回收站？可在回收站还原。`（详情单条则「该代办事项」）  
- 按钮从左到右：**取消**、**彻底删除**、**移入回收站**（主按钮）  
- 「移入回收站」：调用 `trash_work_orders`  
- 「彻底删除」：再开第二个 dialog：`此操作不可恢复，确定彻底删除 N 条？` → 确认后调用 `permanently_delete_work_orders`（详情单条可用 `delete_work_order`）

未选中时「删除选中」仍禁用。成功后提示「已移入回收站 N 条」或「已彻底删除 N 条」，并刷新列表、清空选中。

### 5.3 卡片右键

在现有「打开」「多选」之外调整删除项：

| 项 | 行为 |
|----|------|
| 移入回收站 | 非多选：确认后 trash 当前项。多选且有选中：trash 全部选中。多选但选中为空：禁用 |
| 彻底删除 | 同上，但走不可恢复二次确认 + `permanently_delete_work_orders` |

右键「移入回收站」与工具栏使用同一套三按钮 dialog（取消 / 彻底删除 / 移入回收站）。点「彻底删除」必须再走不可恢复二次确认。

### 5.4 回收站页

- 表头：全选勾选、标题、标签、状态、删除时间  
- 行：勾选；点击行主体打开只读详情（勾选 `@click.stop`）  
- 无拖拽  
- 工具栏：**还原**、**彻底删除**；`selectedCount === 0` 时禁用  
- 还原：不二次确认，调用 `restore_work_orders`，成功后从当前列表移除这些行并清空其选中  
- 彻底删除：不可恢复确认后 `permanently_delete_work_orders`  
- 空状态：`回收站为空`  
- 删除时间：`formatServerDateTime(deletedAt)`（与列表「最后更新」同一格式）

不做卡片模式、不做状态/标签筛选、不做搜索、不做未勾选的「清空回收站」。

### 5.5 只读详情

`WorkOrderDetail` 增加 `readOnly: boolean`（默认 `false`）。回收站传入 `true`。

只读时：

- 标题、描述、状态、标签、计划完成时间、状态附加字段均为禁用  
- 不显示保存、事项删除  
- 不显示新增/编辑/删除进度；进度时间线只读展开查看  
- `AttachmentGallery` 使用已有 `readonly`，不可添加或删除附件  
- 仅可关闭  
- 加载用现有 `get_work_order` + 进度/附件列表；若 get 失败（刚被彻底删除），提示错误并关闭详情  

快捷键 `detail.delete` 在只读模式下不生效。

## 6. 错误与边界

- 批量 API 失败：前端 `message.error`，选中集合不变，不关闭弹层  
- 对已在回收站的 id 再 trash / 对未删除 id 再 restore：后端跳过，整批仍成功  
- 彻底删除时 id 已不存在：跳过  
- 写回收站事项：后端 `Validation`；只读 UI 不发这些请求  
- `list_trashed_work_orders` 失败：回收站页内提示，不自动关闭  
- 主清单筛选/搜索/刷新后：沿用现有逻辑，不可见 id 从 `selectedIds` 移除（已删除项本就不会出现在主清单）  
- 回收站与设置不同时打开；实现上由 `App.vue` 两个 `v-if` 独立控制即可，不强制互斥

## 7. 测试要点

**Rust**

- 迁移后已有行 `deleted_at` 为 `NULL`，新列可写  
- `find_by_filters` 不含已删除事项；`list_trashed` 只含已删除且按时间倒序  
- `trash` 后主清单消失、回收站可见、进度与附件仍在  
- `restore` 后回到主清单，`priority` 与 `updated_at` 不变  
- `permanently_delete` 后行、进度、附件文件均不存在  
- 批量中混有已删除 / 不存在 id 时跳过且事务成功  
- 对回收站事项 `update` / 加进度失败  

**前端（手工验收即可，无现成前端单测框架则不新增测试基建）**

- 主清单删除选中默认进回收站；确认框可走彻底删除  
- 卡片右键两项行为正确  
- 回收站全选、还原、彻底删除  
- 只读详情无保存/编辑/删除入口，附件只读  
- 关闭回收站后主清单出现已还原项、不再出现已彻底删除项  

## 8. 非目标

- 进度日志、附件单独进回收站  
- 到期自动清空、设置项里的保留天数  
- 未勾选的「清空回收站 / 全部还原」  
- 回收站内搜索、筛选、卡片视图、拖拽  
- 打开回收站的快捷键  
- 回收站按钮上的未读/数量角标  
- 主清单工具栏并排两个删除按钮  
- 在回收站内编辑事项  
