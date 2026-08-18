# 代办事项回收站 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 列表与详情的删除默认把事项移入回收站；独立回收站弹层支持多选还原 / 彻底删除，以及只读详情。

**Architecture:** `work_order.deleted_at` 软删除。主清单 `find_by_filters` 排除已删除行；新 command 负责 list_trashed / trash / restore / 批量硬删。前端主清单与详情改为三按钮确认框；`RecycleBin.vue` 作为与设置同级的 `n-modal`。

**Tech Stack:** Rust / rusqlite / Tauri 2 / specta、Vue 3、Naive UI、TypeScript

**Spec:** [2026-08-18-recycle-bin-design.md](../specs/2026-08-18-recycle-bin-design.md)

## Global Constraints

- 数据方案必须是 `work_order.deleted_at`（NULL = 主清单）
- 拒绝写回收站事项：`ServiceError::Validation("Work order is in recycle bin")`，前端展示「该事项在回收站，无法修改」
- `delete_work_order` 保持彻底删除（含已在回收站的事项）
- 移入 / 还原只写 `deleted_at`，不改 `updated_at`、`priority`、进度、标签、附件
- `next_priority` 仍按全表 `MAX(priority)`，不排除回收站
- 不改 settings API、不改 ZIP 备份逻辑
- 本仓库无前端单测框架：前端 Task 用 `npm run build`（含 `vue-tsc --noEmit`）+ 文内手工检查
- **不要执行 git commit / push**（用户明确要求不做 git 操作）；计划中无 Commit 步骤

---

## File Structure

| 文件 | 职责 |
|------|------|
| `src-tauri/src/db/schema.sql` | 新库 `deleted_at` + 索引 |
| `src-tauri/src/db/migrate.rs` | 存量库加列 + `idx_work_order_deleted_at` |
| `src-tauri/src/db/connection.rs` | 调用新迁移 |
| `src-tauri/src/models/work_order.rs` | `deleted_at: Option<NaiveDateTime>` |
| `src-tauri/src/services/work_order_service.rs` | 筛选排除、trash/restore/list_trashed、写保护 |
| `src-tauri/src/services/progress_log_service.rs` | 写操作拒绝回收站事项 |
| `src-tauri/src/services/attachment_service.rs` | 新增/单条删除前拒绝回收站所属工单 |
| `src-tauri/src/commands/work_order.rs` | 新 command；批量硬删走现有附件级联 |
| `src-tauri/src/lib.rs` | 注册 command |
| `src/bindings.ts` | specta 重新导出 |
| `src/api/workOrders.ts` | 封装新 API |
| `src/composables/useTrashConfirm.ts` | 三按钮删除确认 + 彻底删除二次确认 |
| `src/views/WorkOrderList.vue` | 回收站入口、删除双路径、右键两项 |
| `src/views/RecycleBin.vue` | 回收站弹层 |
| `src/views/WorkOrderDetail.vue` | `readOnly`、删除双路径 |
| `src/App.vue` | 挂载回收站弹层 |
| `src/styles/main.css` | 回收站列表列宽 |
| `src/components/TagPicker.vue` | 只读禁用 |
| `src/types/shortcuts.ts` / `useShortcuts.ts` | 回收站打开时屏蔽主清单快捷键 |

---

## 任务清单

- [x] Task 1: schema / 迁移 / `WorkOrder.deleted_at`
- [x] Task 2: 主清单排除、list_trashed、trash、restore
- [x] Task 3: 写保护 + 批量硬删
- [x] Task 4: Tauri commands + bindings + 前端 API
- [x] Task 5: 删除确认 composable + 主清单双路径 / 右键 / 入口
- [x] Task 6: RecycleBin 弹层 + App 挂载
- [x] Task 7: 只读详情 + 详情删除双路径
- [x] Task 8: 收尾验证

---

### Task 1: schema / 迁移 / 模型

**Files:**
- Modify: `src-tauri/src/db/schema.sql`
- Modify: `src-tauri/src/db/migrate.rs`
- Modify: `src-tauri/src/db/connection.rs`
- Modify: `src-tauri/src/models/work_order.rs`

**Interfaces:**
- Produces:
  - 列 `work_order.deleted_at TIMESTAMP`（可空）
  - 索引 `idx_work_order_deleted_at`
  - `pub fn migrate_work_order_deleted_at(conn: &Connection) -> Result<(), ServiceError>`
  - `WorkOrder.deleted_at: Option<NaiveDateTime>`（serde `deletedAt`）

- [ ] **Step 1: Write the failing test**

在 `migrate.rs` 增加：

```rust
#[cfg(test)]
mod deleted_at_tests {
    use super::*;
    use crate::db::connection::open_connection;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn existing_rows_have_null_deleted_at_and_column_is_writable() {
        let dir = temp_dir("migrate-deleted-at");
        let conn = open_connection(&dir).unwrap();
        conn.execute(
            "INSERT INTO work_order (title, description, status, priority, due_date, created_at, updated_at)
             VALUES ('Keep', NULL, 'NOT_STARTED', 0, NULL, datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        migrate_work_order_deleted_at(&conn).unwrap();
        let deleted: Option<String> = conn
            .query_row(
                "SELECT deleted_at FROM work_order WHERE title = 'Keep'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(deleted.is_none());
        conn.execute(
            "UPDATE work_order SET deleted_at = datetime('now') WHERE title = 'Keep'",
            [],
        )
        .unwrap();
        let deleted: Option<String> = conn
            .query_row(
                "SELECT deleted_at FROM work_order WHERE title = 'Keep'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(deleted.is_some());
        let _ = std::fs::remove_dir_all(dir);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml migrate_work_order_deleted_at -- --nocapture`  
Expected: compile fail（函数不存在）或测试 fail。

- [ ] **Step 3: Implement schema + migrate + model**

`schema.sql` 的 `work_order` 增加 `deleted_at TIMESTAMP`，并：

```sql
CREATE INDEX IF NOT EXISTS idx_work_order_deleted_at ON work_order(deleted_at);
```

`migrate.rs`：

```rust
pub fn migrate_work_order_deleted_at(conn: &Connection) -> Result<(), ServiceError> {
    if !column_exists(conn, "work_order", "deleted_at")? {
        conn.execute("ALTER TABLE work_order ADD COLUMN deleted_at TIMESTAMP", [])?;
    }
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_work_order_deleted_at ON work_order(deleted_at)",
        [],
    )?;
    Ok(())
}
```

`connection.rs` 的 `open_connection` 在其它 migrate 之后调用 `migrate::migrate_work_order_deleted_at(&conn)?`。

`WorkOrder` 增加：

```rust
pub deleted_at: Option<NaiveDateTime>,
```

放在 `updated_at` 之后。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml deleted_at -- --nocapture`  
Expected: PASS（含本 Task 测试；若 `row_to_work_order` 尚未读新列，后续 Task 2 会一起修编译）。

---

### Task 2: 主清单排除、list_trashed、trash、restore

**Files:**
- Modify: `src-tauri/src/services/work_order_service.rs`

**Interfaces:**
- Consumes: `WorkOrder.deleted_at`、`read_optional_datetime_column`
- Produces:
  - `pub fn list_trashed(conn: &Connection) -> Result<Vec<WorkOrder>, ServiceError>`
  - `pub fn trash_work_orders(conn: &mut Connection, ids: &[i64]) -> Result<(), ServiceError>`
  - `pub fn restore_work_orders(conn: &mut Connection, ids: &[i64]) -> Result<(), ServiceError>`
  - `pub fn ensure_not_trashed(conn: &Connection, id: i64) -> Result<(), ServiceError>`
  - `find_by_filters` 始终附加 `deleted_at IS NULL`

- [ ] **Step 1: Write the failing tests**

在 `work_order_service.rs` tests 中增加（沿用现有 `temp_db` / `input` / `config` / `tag_config`）：

```rust
    #[test]
    fn find_by_filters_excludes_trashed_and_list_trashed_is_newest_first() {
        let (mut conn, dir) = temp_db();
        let a = create(&conn, input("A"), &config(), &tag_config()).unwrap();
        let b = create(&conn, input("B"), &config(), &tag_config()).unwrap();
        trash_work_orders(&mut conn, &[a.id.unwrap()]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        trash_work_orders(&mut conn, &[b.id.unwrap()]).unwrap();
        let main = find_by_statuses(&conn, &[], None).unwrap();
        assert!(main.is_empty());
        let trash = list_trashed(&conn).unwrap();
        assert_eq!(trash.len(), 2);
        assert_eq!(trash[0].id, b.id);
        assert_eq!(trash[1].id, a.id);
        assert!(trash[0].deleted_at.is_some());
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn trash_hides_from_main_keeps_progress_and_restore_preserves_priority_updated_at() {
        let (mut conn, dir) = temp_db();
        let wo = create(&conn, input("Keep"), &config(), &tag_config()).unwrap();
        let id = wo.id.unwrap();
        let priority = wo.priority;
        let updated_at = wo.updated_at;
        crate::services::progress_log_service::add_log(
            &conn,
            id,
            &crate::models::progress_log::ProgressLogInput {
                title: "step".into(),
                content: None,
                status: "IN_PROGRESS".into(),
                extra_fields: None,
            },
            &config(),
        )
        .unwrap();
        trash_work_orders(&mut conn, &[id]).unwrap();
        assert!(find_by_statuses(&conn, &[], None).unwrap().is_empty());
        let logs = crate::services::progress_log_service::find_by_work_order_id(&conn, id).unwrap();
        assert_eq!(logs.len(), 1);
        restore_work_orders(&mut conn, &[id]).unwrap();
        let restored = get_required(&conn, id).unwrap();
        assert!(restored.deleted_at.is_none());
        assert_eq!(restored.priority, priority);
        assert_eq!(restored.updated_at, updated_at);
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn trash_restore_skip_mismatched_and_empty_ids() {
        let (mut conn, dir) = temp_db();
        let wo = create(&conn, input("Live"), &config(), &tag_config()).unwrap();
        let id = wo.id.unwrap();
        trash_work_orders(&mut conn, &[]).unwrap();
        restore_work_orders(&mut conn, &[]).unwrap();
        trash_work_orders(&mut conn, &[id, 9_999_999]).unwrap();
        trash_work_orders(&mut conn, &[id]).unwrap();
        restore_work_orders(&mut conn, &[id, 9_999_999]).unwrap();
        restore_work_orders(&mut conn, &[id]).unwrap();
        assert_eq!(find_by_statuses(&conn, &[], None).unwrap().len(), 1);
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib trash_hides -- --nocapture`  
Expected: compile fail（函数不存在）。

- [ ] **Step 3: Implement**

`WORK_ORDER_SELECT` 增加 `deleted_at`。`row_to_work_order` 读取 `deleted_at`。`find_by_filters` 始终加入 `deleted_at IS NULL`。

```rust
pub const RECYCLE_BIN_VALIDATION: &str = "Work order is in recycle bin";

pub fn ensure_not_trashed(conn: &Connection, id: i64) -> Result<(), ServiceError> {
    let wo = get_required(conn, id)?;
    if wo.deleted_at.is_some() {
        return Err(ServiceError::Validation(RECYCLE_BIN_VALIDATION.into()));
    }
    Ok(())
}

pub fn list_trashed(conn: &Connection) -> Result<Vec<WorkOrder>, ServiceError> {
    let mut stmt = conn.prepare(&format!(
        "{WORK_ORDER_SELECT} WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC"
    ))?;
    let rows = stmt.query_map([], row_to_work_order)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(enrich_with_tags(conn, row?)?);
    }
    Ok(result)
}

pub fn trash_work_orders(conn: &mut Connection, ids: &[i64]) -> Result<(), ServiceError> {
    if ids.is_empty() {
        return Ok(());
    }
    let tx = conn.transaction()?;
    let now = format_datetime(Utc::now().naive_utc());
    for id in ids {
        tx.execute(
            "UPDATE work_order SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
            params![now, id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn restore_work_orders(conn: &mut Connection, ids: &[i64]) -> Result<(), ServiceError> {
    if ids.is_empty() {
        return Ok(());
    }
    let tx = conn.transaction()?;
    for id in ids {
        tx.execute(
            "UPDATE work_order SET deleted_at = NULL WHERE id = ?1 AND deleted_at IS NOT NULL",
            params![id],
        )?;
    }
    tx.commit()?;
    Ok(())
}
```

`update` 与 `update_priorities` 在写入前调用 `ensure_not_trashed`。

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib -- services::work_order_service`  
Expected: PASS。

---

### Task 3: 写保护 + 批量硬删

**Files:**
- Modify: `src-tauri/src/services/progress_log_service.rs`
- Modify: `src-tauri/src/services/attachment_service.rs`
- Modify: `src-tauri/src/services/work_order_service.rs`（可增加 `exists` 辅助，硬删仍由 command 调现有 `delete` + `delete_all_for_work_order`）

**Interfaces:**
- Consumes: `ensure_not_trashed`
- Produces: 进度增删改、附件新增/单条删除在所属工单已删除时 Validation；`permanently_delete_work_orders` 在 command 层实现

- [ ] **Step 1: Write the failing tests**

`work_order_service` tests：

```rust
    #[test]
    fn update_trashed_work_order_is_rejected() {
        let (mut conn, dir) = temp_db();
        let wo = create(&conn, input("X"), &config(), &tag_config()).unwrap();
        let id = wo.id.unwrap();
        trash_work_orders(&mut conn, &[id]).unwrap();
        let err = update(&conn, id, input("X2"), &config(), &tag_config()).unwrap_err();
        match err {
            ServiceError::Validation(msg) => assert_eq!(msg, RECYCLE_BIN_VALIDATION),
            other => panic!("{other:?}"),
        }
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }
```

`progress_log_service` tests：对已 trash 的工单 `add_log` 失败。

`attachment_service` tests：对已 trash 的工单 `add_from_bytes` 失败；`delete_all_for_work_order` 仍可用于硬删。

`work_order_service`：

```rust
    #[test]
    fn permanently_delete_removes_row_progress_and_skips_missing() {
        // 在 command 层测也可；service 层测 delete() 对已 trash 行仍可删
        let (mut conn, dir) = temp_db();
        let wo = create(&conn, input("Gone"), &config(), &tag_config()).unwrap();
        let id = wo.id.unwrap();
        trash_work_orders(&mut conn, &[id]).unwrap();
        delete(&conn, id).unwrap();
        let err = get_required(&conn, id).unwrap_err();
        assert!(matches!(err, ServiceError::NotFound(_)));
        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Expected: `update` 当前仍会成功。

- [ ] **Step 3: Implement guards**

`update` 开头：`ensure_not_trashed(conn, id)?;`（在 `get_required` 之后或合并）。  
`update_priorities`：每个 id `ensure_not_trashed`。  
`add_log` / `update_log` / `delete_log`：在确认工单存在后 `ensure_not_trashed(conn, work_order_id)?`。  
附件：在 `write_validated_image` 与 `delete_one` 中解析所属工单并 `ensure_not_trashed`。`delete_all_for_work_order` **不**拦截。

- [ ] **Step 4: Run tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib`  
Expected: PASS。

---

### Task 4: Commands + bindings + 前端 API

**Files:**
- Modify: `src-tauri/src/commands/work_order.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/api/workOrders.ts`
- Generated: `src/bindings.ts`（`npm run bindings` 或 `cargo test export_bindings`）

**Interfaces:**
- Produces:
  - `list_trashed_work_orders() -> Vec<WorkOrder>`
  - `trash_work_orders(ids: Vec<i64>)`
  - `restore_work_orders(ids: Vec<i64>)`
  - `permanently_delete_work_orders(ids: Vec<i64>)`：事务内对存在的 id 先 `delete_all_for_work_order` 再 `work_order_service::delete`；不存在则跳过

```rust
#[tauri::command]
#[specta::specta]
pub fn permanently_delete_work_orders(
    state: State<'_, AppState>,
    ids: Vec<i64>,
) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let tx = conn.transaction().map_err(map_err)?;
    for id in ids {
        if work_order_service::get_required(&tx, id).is_err() {
            continue;
        }
        attachment_service::delete_all_for_work_order(&tx, &state.data_dir, id).map_err(map_err)?;
        work_order_service::delete(&tx, id).map_err(map_err)?;
    }
    tx.commit().map_err(map_err)?;
    Ok(())
}
```

前端：

```ts
export function listTrashedWorkOrders(): Promise<WorkOrder[]> {
  return commands.listTrashedWorkOrders();
}
export function trashWorkOrders(ids: number[]): Promise<void> {
  return commands.trashWorkOrders(ids).then(() => undefined);
}
export function restoreWorkOrders(ids: number[]): Promise<void> {
  return commands.restoreWorkOrders(ids).then(() => undefined);
}
export function permanentlyDeleteWorkOrders(ids: number[]): Promise<void> {
  return commands.permanentlyDeleteWorkOrders(ids).then(() => undefined);
}
```

- [ ] **Step 1–4:** 实现 command、注册、导出 bindings、封装 API。  
Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib export_bindings` 然后 `npm run build`（或仅 bindings 导出）。

---

### Task 5: 主清单删除双路径 / 右键 / 入口

**Files:**
- Create: `src/composables/useTrashConfirm.ts`
- Modify: `src/views/WorkOrderList.vue`

**Interfaces:**
- Produces: `confirmMoveToTrash({ count, singular?, onTrash, onPermanentRequest })` 与 `confirmPermanentDelete({ count, onConfirm })`
- 工具栏「删除选中」改为三按钮 dialog；成功提示「已移入回收站 N 条」或「已彻底删除 N 条」
- 设置旁「回收站」按钮，emit `openRecycleBin`
- 右键：移入回收站 / 彻底删除（彻底删除必须二次确认）

按钮从左到右：**取消**、**彻底删除**、**移入回收站**（主按钮）。点彻底删除必须再开：`此操作不可恢复，确定彻底删除 N 条？`

- [ ] **Step 1:** 实现 composable + 列表接入  
- [ ] **Step 2:** `npm run build` 通过

---

### Task 6: RecycleBin + App 挂载

**Files:**
- Create: `src/views/RecycleBin.vue`
- Modify: `src/App.vue`
- Modify: `src/styles/main.css`
- Modify: `src/types/shortcuts.ts`、`src/composables/useShortcuts.ts`（回收站打开时不跑 list 快捷键）

行为：
- `n-modal` preset card，宽约 840px，可滚动
- 表头：全选、标题、标签、状态、删除时间（`formatServerDateTime(deletedAt)`）
- 行点击打开只读详情（勾选 `@click.stop`）
- 工具栏还原（无二次确认）、彻底删除（二次确认）；`selectedCount === 0` 禁用
- 空状态：`回收站为空`
- 关闭后 `list.reload()`
- 加载失败：页内 `message.error`，不自动关闭

- [ ] **Step 1:** 实现并挂载  
- [ ] **Step 2:** `npm run build` 通过

---

### Task 7: 只读详情 + 详情删除双路径

**Files:**
- Modify: `src/views/WorkOrderDetail.vue`
- Modify: `src/components/TagPicker.vue`（`disabled`）

- `readOnly: boolean` 默认 `false`；回收站传入 `true`
- 只读：标题「查看代办」；字段禁用；无保存/事项删除/进度增删改；`AttachmentGallery` `readonly`；仅可关闭
- 加载：`get_work_order`，失败则报错并关闭
- `detail.delete` 在 `readOnly` 下 `enabled: false`
- 非只读删除走与列表相同的三按钮 dialog；彻底删除可用 `deleteWorkOrder` 或 `permanentlyDeleteWorkOrders([id])`

- [ ] **Step 1:** 实现  
- [ ] **Step 2:** `npm run build` 通过

---

### Task 8: 收尾验证

- [ ] **Step 1:** `cargo test --manifest-path src-tauri/Cargo.toml --lib`  
Expected: PASS

- [ ] **Step 2:** `npm run build`  
Expected: PASS

- [ ] **Step 3:** 对照 spec §7 手工清单（主清单进回收站、右键、全选还原/硬删、只读详情、关闭后列表刷新）
