# 图片附件 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Git 策略：** 实现过程中**不**逐步提交 git；全部任务完成并验证通过后，由用户自行决定是否提交。

**Goal:** 在代办（WorkOrder）与处置过程（ProgressLog）中支持多张图片附件，通过文件选择器与剪贴板粘贴添加，支持逐张删除。

**Architecture:** 方案 A — 图片存 `{data_dir}/attachments/{owner_type}/{owner_id}/`，SQLite `attachment` 表存元数据；`attachment_service` 负责校验、读写与级联清理；前端复用 `AttachmentGallery.vue`，未保存记录时前端暂存、保存后批量上传。

**Tech Stack:** Tauri 2, Rust (rusqlite, uuid), tauri-specta, tauri-plugin-dialog, Vue 3, Naive UI, `@tauri-apps/api/core` (`convertFileSrc`)

**Spec:** [2026-07-07-image-attachments-design.md](../specs/2026-07-07-image-attachments-design.md)

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `src-tauri/src/db/schema.sql` | Modify | 新增 `attachment` 表 |
| `src-tauri/src/db/migrate.rs` | Modify | `migrate_attachment` |
| `src-tauri/src/db/connection.rs` | Modify | 启动时调用新迁移 |
| `src-tauri/src/models/attachment.rs` | Create | `OwnerType`, `Attachment` |
| `src-tauri/src/models/mod.rs` | Modify | `pub mod attachment` |
| `src-tauri/src/services/attachment_service.rs` | Create | 增删查、文件 I/O、魔数校验 |
| `src-tauri/src/services/mod.rs` | Modify | `pub mod attachment_service` |
| `src-tauri/src/commands/attachment.rs` | Create | 4 个 Command + `pick_attachment_file` |
| `src-tauri/src/commands/mod.rs` | Modify | `pub mod attachment` |
| `src-tauri/src/commands/work_order.rs` | Modify | 删除工单前清理附件 |
| `src-tauri/src/commands/progress_log.rs` | Modify | 删除过程前清理附件 |
| `src-tauri/src/lib.rs` | Modify | 注册新 Command |
| `src-tauri/Cargo.toml` | Modify | 添加 `uuid` 依赖 |
| `src-tauri/capabilities/default.json` | Modify | asset 协议 scope（图片预览） |
| `src/api/attachments.ts` | Create | 前端 API 封装 |
| `src/components/AttachmentGallery.vue` | Create | 可复用图片画廊 |
| `src/views/WorkOrderDetail.vue` | Modify | 嵌入画廊（代办 + 过程） |
| `src/types.ts` | Modify | 导出 `Attachment`, `OwnerType` |
| `src/bindings.ts` | Regenerate | `npm run bindings` |
| `docs/api/commands.md` | Modify | 附件 Command 文档 |

---

### Task 1: 数据库 schema 与迁移

**Files:**
- Modify: `src-tauri/src/db/schema.sql`
- Modify: `src-tauri/src/db/migrate.rs`
- Modify: `src-tauri/src/db/connection.rs`

- [ ] **Step 1: 在 schema.sql 末尾追加 attachment 表**

```sql
CREATE TABLE IF NOT EXISTS attachment (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_type VARCHAR(50) NOT NULL,
    owner_id INTEGER NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    original_name VARCHAR(255),
    mime_type VARCHAR(100) NOT NULL,
    file_size INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_attachment_owner ON attachment(owner_type, owner_id);
```

- [ ] **Step 2: 在 migrate.rs 添加 migrate_attachment**

```rust
/// 确保 attachment 表存在（存量库升级）。
pub fn migrate_attachment(conn: &Connection) -> Result<(), ServiceError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS attachment (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            owner_type VARCHAR(50) NOT NULL,
            owner_id INTEGER NOT NULL,
            file_name VARCHAR(255) NOT NULL,
            original_name VARCHAR(255),
            mime_type VARCHAR(100) NOT NULL,
            file_size INTEGER NOT NULL,
            created_at TIMESTAMP NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_attachment_owner ON attachment(owner_type, owner_id);",
    )?;
    Ok(())
}
```

- [ ] **Step 3: 在 connection.rs 的 open_connection 中调用 migrate_attachment**

在现有 `migrate::migrate_progress_log(conn)?;` 之后追加：

```rust
migrate::migrate_attachment(conn)?;
```

- [ ] **Step 4: 验证编译**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`  
Expected: 编译通过

---

### Task 2: Attachment 模型

**Files:**
- Create: `src-tauri/src/models/attachment.rs`
- Modify: `src-tauri/src/models/mod.rs`

- [ ] **Step 1: 创建 models/attachment.rs**

```rust
//! 图片附件模型。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum OwnerType {
    WorkOrder,
    ProgressLog,
}

impl OwnerType {
    pub fn as_str(self) -> &'static str {
        match self {
            OwnerType::WorkOrder => "work_order",
            OwnerType::ProgressLog => "progress_log",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "work_order" => Some(OwnerType::WorkOrder),
            "progress_log" => Some(OwnerType::ProgressLog),
            _ => None,
        }
    }

    pub fn dir_name(self) -> &'static str {
        self.as_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: Option<i64>,
    pub owner_type: OwnerType,
    pub owner_id: i64,
    pub file_name: String,
    pub original_name: Option<String>,
    pub mime_type: String,
    pub file_size: i64,
    pub created_at: NaiveDateTime,
    /// 完整磁盘路径，查询时由 service 填充，不存 DB。
    pub file_path: String,
}
```

- [ ] **Step 2: 在 models/mod.rs 注册**

```rust
pub mod attachment;
```

- [ ] **Step 3: 验证编译**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`  
Expected: 编译通过

---

### Task 3: attachment_service 核心逻辑

**Files:**
- Create: `src-tauri/src/services/attachment_service.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 添加 uuid 依赖到 Cargo.toml**

```toml
uuid = { version = "1", features = ["v4"] }
```

- [ ] **Step 2: 创建 attachment_service.rs**

实现以下函数（完整代码写入文件）：

| 函数 | 说明 |
|------|------|
| `attachments_dir(data_dir, owner_type, owner_id)` | 返回 `{data_dir}/attachments/{type}/{id}/` |
| `validate_image(mime, size, header)` | MIME 白名单 + 20MB 上限 + 魔数校验 |
| `detect_mime(header)` | 从文件头判断 MIME |
| `extension_for_mime(mime)` | 返回 `.jpg` / `.png` 等 |
| `list_by_owner(conn, data_dir, owner_type, owner_id)` | 查询并填充 file_path |
| `add_from_file(conn, data_dir, owner_type, owner_id, source_path, original_name)` | 读源文件、校验、复制到 attachments |
| `add_from_bytes(conn, data_dir, owner_type, owner_id, file_name, mime_type, data)` | 写入字节 |
| `delete_one(conn, data_dir, id)` | 删 DB + 删文件 + 清空目录 |
| `delete_all_for_owner(conn, data_dir, owner_type, owner_id)` | 删归属全部 |
| `delete_all_for_work_order(conn, data_dir, work_order_id)` | 删工单附件 + 其下全部 progress_log 附件 |

允许的 MIME：`image/jpeg`, `image/png`, `image/gif`, `image/webp`, `image/bmp`

魔数校验示例：

```rust
fn detect_mime(header: &[u8]) -> Option<&'static str> {
    if header.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }
    if header.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        return Some("image/png");
    }
    if header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if header.len() >= 12 && &header[0..4] == b"RIFF" && &header[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if header.starts_with(&[0x42, 0x4D]) {
        return Some("image/bmp");
    }
    None
}
```

存储文件名：`{uuid_v4}{ext}`，不使用用户原始文件名。

`add_from_file` / `add_from_bytes` 流程：
1. 校验 owner 记录存在（work_order 或 progress_log）
2. 读取并校验图片
3. `fs::create_dir_all(attachments_dir)`
4. 写入文件
5. INSERT attachment 表
6. 若 INSERT 失败，删除已写文件

- [ ] **Step 3: 编写单元测试**

```rust
#[test]
fn add_list_delete_attachment() { /* 创建工单 → 添加图片字节 → 列表非空 → 删除 → 列表空 → 文件不存在 */ }

#[test]
fn rejects_oversized_file() { /* >20MB 返回 Validation 错误 */ }

#[test]
fn rejects_non_image() { /* text/plain 字节返回 Validation 错误 */ }

#[test]
fn cascade_delete_work_order() { /* 工单+过程各有附件，delete_all_for_work_order 后全清 */ }
```

- [ ] **Step 4: 运行测试**

Run: `cargo test --manifest-path src-tauri/Cargo.toml attachment_service`  
Expected: 全部 PASS

---

### Task 4: Tauri Commands 与级联删除

**Files:**
- Create: `src-tauri/src/commands/attachment.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/commands/work_order.rs`
- Modify: `src-tauri/src/commands/progress_log.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/default.json`

- [ ] **Step 1: 创建 commands/attachment.rs**

```rust
//! 图片附件 Tauri Command。

use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::error::ServiceError;
use crate::models::attachment::{Attachment, OwnerType};
use crate::services::attachment_service;
use crate::AppState;

fn map_err(err: ServiceError) -> String {
    err.to_string()
}

#[tauri::command]
#[specta::specta]
pub fn list_attachments(
    state: State<'_, AppState>,
    owner_type: OwnerType,
    owner_id: i64,
) -> Result<Vec<Attachment>, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    attachment_service::list_by_owner(&conn, &state.data_dir, owner_type, owner_id)
        .map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn add_attachment_from_file(
    state: State<'_, AppState>,
    owner_type: OwnerType,
    owner_id: i64,
    source_path: String,
) -> Result<Attachment, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let path = std::path::Path::new(&source_path);
    let original_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(String::from);
    attachment_service::add_from_file(
        &conn,
        &state.data_dir,
        owner_type,
        owner_id,
        path,
        original_name,
    )
    .map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn add_attachment_from_bytes(
    state: State<'_, AppState>,
    owner_type: OwnerType,
    owner_id: i64,
    file_name: String,
    mime_type: String,
    data: Vec<u8>,
) -> Result<Attachment, String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    attachment_service::add_from_bytes(
        &conn,
        &state.data_dir,
        owner_type,
        owner_id,
        &file_name,
        &mime_type,
        &data,
    )
    .map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn delete_attachment(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    attachment_service::delete_one(&conn, &state.data_dir, id).map_err(map_err)
}

/// 打开图片文件选择器，返回选中路径（未选则 None）。
#[tauri::command]
#[specta::specta]
pub async fn pick_attachment_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let path = app
        .dialog()
        .file()
        .add_filter("图片", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
        .blocking_pick_file();
    Ok(path.map(|p| p.to_string()))
}
```

- [ ] **Step 2: 在 lib.rs 注册 Command**

```rust
commands::attachment::list_attachments,
commands::attachment::add_attachment_from_file,
commands::attachment::add_attachment_from_bytes,
commands::attachment::delete_attachment,
commands::attachment::pick_attachment_file,
```

- [ ] **Step 3: 扩展 delete_work_order command**

在 `delete_work_order` 中，DB 删除之前调用：

```rust
attachment_service::delete_all_for_work_order(&conn, &state.data_dir, id)?;
```

- [ ] **Step 4: 扩展 delete_progress_log command**

在 `delete_progress_log` 中，DB 删除之前调用：

```rust
attachment_service::delete_all_for_owner(
    &conn,
    &state.data_dir,
    OwnerType::ProgressLog,
    log_id,
)?;
```

- [ ] **Step 5: 配置 asset 协议 scope**

在 `capabilities/default.json` 的 permissions 中追加（允许 convertFileSrc 加载附件目录）：

```json
{
  "identifier": "core:default",
  "allow": [
    { "path": "$APPDATA/**" },
    { "path": "$HOME/**" },
    { "path": "$DESKTOP/**" },
    { "path": "**/data/**" },
    { "path": "**/attachments/**" }
  ]
}
```

若 Tauri 2 schema 要求不同格式，以 `gen/schemas/desktop-schema.json` 为准调整；目标是允许加载 `data_dir/attachments/` 下文件。

- [ ] **Step 6: 生成 bindings 并验证编译**

Run: `npm run bindings`  
Run: `cargo check --manifest-path src-tauri/Cargo.toml`  
Expected: `src/bindings.ts` 含新 Command 类型，编译通过

---

### Task 5: 前端 API 层

**Files:**
- Create: `src/api/attachments.ts`
- Modify: `src/types.ts`

- [ ] **Step 1: 创建 api/attachments.ts**

```typescript
import { commands } from "../bindings";
import type { Attachment, OwnerType } from "../types";

export async function listAttachments(
  ownerType: OwnerType,
  ownerId: number,
): Promise<Attachment[]> {
  return commands.listAttachments(ownerType, ownerId);
}

export async function addAttachmentFromFile(
  ownerType: OwnerType,
  ownerId: number,
  sourcePath: string,
): Promise<Attachment> {
  return commands.addAttachmentFromFile(ownerType, ownerId, sourcePath);
}

export async function addAttachmentFromBytes(
  ownerType: OwnerType,
  ownerId: number,
  fileName: string,
  mimeType: string,
  data: Uint8Array,
): Promise<Attachment> {
  return commands.addAttachmentFromBytes(
    ownerType,
    ownerId,
    fileName,
    mimeType,
    Array.from(data),
  );
}

export async function deleteAttachment(id: number): Promise<void> {
  return commands.deleteAttachment(id);
}

export async function pickAttachmentFile(): Promise<string | null> {
  return commands.pickAttachmentFile();
}
```

- [ ] **Step 2: 在 types.ts 导出 Attachment / OwnerType**

从 `bindings.ts` re-export：

```typescript
export type { Attachment, OwnerType } from "../bindings";
```

---

### Task 6: AttachmentGallery 组件

**Files:**
- Create: `src/components/AttachmentGallery.vue`
- Modify: `src/styles/main.css`

- [ ] **Step 1: 创建 AttachmentGallery.vue**

**Props:**
- `ownerType: OwnerType`
- `ownerId?: number` — 未定义时为暂存模式
- `readonly?: boolean` — 默认 false

**Emits:**
- `staged-change(files: File[])` — 暂存模式下列表变化

**功能实现要点：**

```vue
<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { onMounted, ref, watch } from "vue";
import { useMessage } from "naive-ui";
import * as attachmentApi from "../api/attachments";
import type { Attachment, OwnerType } from "../types";

// attachments: 已上传列表
// staged: { file: File; previewUrl: string }[] 暂存列表

async function load() {
  if (props.ownerId == null) return;
  attachments.value = await attachmentApi.listAttachments(props.ownerType, props.ownerId);
}

async function handlePickFile() {
  const path = await attachmentApi.pickAttachmentFile();
  if (!path) return;
  if (props.ownerId == null) {
    // 暂存：通过 @tauri-apps/api 无法直接读路径为 File，
    // 调用 addAttachmentFromFile 需要 ownerId；
    // 方案：用 fetch(convertFileSrc(path)) 读为 blob 再暂存
    const url = convertFileSrc(path);
    const resp = await fetch(url);
    const blob = await resp.blob();
    const file = new File([blob], path.split(/[/\\]/).pop() ?? "image.png", { type: blob.type });
    addStaged(file);
  } else {
    await attachmentApi.addAttachmentFromFile(props.ownerType, props.ownerId, path);
    await load();
  }
}

async function handlePaste(e: ClipboardEvent) {
  const items = e.clipboardData?.items;
  if (!items) return;
  for (const item of items) {
    if (!item.type.startsWith("image/")) continue;
    e.preventDefault();
    const file = item.getAsFile();
    if (!file) continue;
    if (props.ownerId == null) {
      addStaged(file);
    } else {
      const buf = new Uint8Array(await file.arrayBuffer());
      await attachmentApi.addAttachmentFromBytes(
        props.ownerType, props.ownerId, file.name, file.type, buf,
      );
      await load();
    }
    break;
  }
}

// expose: uploadStaged(ownerId) — 父组件保存后调用
defineExpose({ uploadStaged });
</script>
```

**模板结构：**
- 外层 `div.attachment-gallery` 监听 `@paste`
- 缩略图网格 `.attachment-grid`
- 每张图：`<img :src="thumbSrc(att)">` + 删除按钮
- 「添加图片」按钮
- 提示文字

- [ ] **Step 2: 添加 CSS 到 main.css**

```css
.attachment-gallery { margin-top: 8px; }
.attachment-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 8px;
}
.attachment-thumb {
  position: relative;
  width: 80px;
  height: 80px;
  border-radius: 4px;
  overflow: hidden;
  border: 1px solid #e0e0e0;
}
.attachment-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  cursor: pointer;
}
.attachment-thumb .delete-btn {
  position: absolute;
  top: 2px;
  right: 2px;
}
.attachment-hint {
  color: #999;
  font-size: 12px;
}
```

- [ ] **Step 3: 验证 TypeScript**

Run: `npm run build`  
Expected: 无类型错误（组件尚未被引用时可能需临时 import 验证）

---

### Task 7: 集成到 WorkOrderDetail

**Files:**
- Modify: `src/views/WorkOrderDetail.vue`

- [ ] **Step 1: 导入组件与 ref**

```typescript
import AttachmentGallery from "../components/AttachmentGallery.vue";
import type { OwnerType } from "../types";

const workOrderGalleryRef = ref<InstanceType<typeof AttachmentGallery> | null>(null);
const progressGalleryRef = ref<InstanceType<typeof AttachmentGallery> | null>(null);
```

- [ ] **Step 2: 在代办描述下方嵌入画廊**

```vue
<n-form-item label="图片">
  <AttachmentGallery
    ref="workOrderGalleryRef"
    owner-type="work_order"
    :owner-id="workOrderId"
  />
</n-form-item>
```

- [ ] **Step 3: 在过程时间线展开区只读展示**

```vue
<AttachmentGallery
  owner-type="progress_log"
  :owner-id="log.id!"
  readonly
/>
```

- [ ] **Step 4: 在追加/编辑过程表单嵌入可编辑画廊**

```vue
<n-form-item label="图片">
  <AttachmentGallery
    ref="progressGalleryRef"
    owner-type="progress_log"
    :owner-id="editingLogId ?? undefined"
  />
</n-form-item>
```

- [ ] **Step 5: 修改 save() — 新建代办后上传暂存图片**

```typescript
if (isNew.value) {
  const created = await workOrderApi.createWorkOrder(input);
  workOrderId.value = created.id ?? undefined;
  if (workOrderId.value != null) {
    await workOrderGalleryRef.value?.uploadStaged(workOrderId.value);
  }
  // ...existing flush/load...
}
```

- [ ] **Step 6: 修改 saveProgress() — 新建过程后上传暂存图片**

```typescript
} else {
  const created = await progressLogApi.addProgressLog(workOrderId.value, input);
  const logId = created.id;
  if (logId != null) {
    await progressGalleryRef.value?.uploadStaged(logId);
  }
  clearProgressForm();
}
```

- [ ] **Step 7: 修改 clearProgressForm() — 重置过程画廊暂存**

确保 `progressGalleryRef.value?.clearStaged()` 在取消编辑时调用。

- [ ] **Step 8: 修改 startEdit(log) — 切换编辑目标时重置画廊**

编辑已有过程时 `ownerId` 有值，走即时上传模式；`clearStaged` 清理上一轮暂存。

---

### Task 8: 文档与最终验证

**Files:**
- Modify: `docs/api/commands.md`

- [ ] **Step 1: 补充附件 Command 文档**

为 `list_attachments`、`add_attachment_from_file`、`add_attachment_from_bytes`、`delete_attachment`、`pick_attachment_file` 各写一段：参数、返回值、错误说明。

- [ ] **Step 2: 运行全部 Rust 测试**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`  
Expected: 全部 PASS

- [ ] **Step 3: 运行前端构建**

Run: `npm run build`  
Expected: 无错误

- [ ] **Step 4: 手动冒烟测试清单**

1. 编辑已有代办 → 选文件添加 → 缩略图显示 → 点击预览
2. Ctrl+V 粘贴截图 → 成功添加
3. 删除单张 → 消失
4. 新建代办 → 暂存图片 → 保存 → 图片关联正确
5. 追加过程 → 暂存 → 保存 → 时间线可见
6. 删除代办 → 附件目录清理
7. 删除过程 → 该过程附件清理

---

## Spec Coverage Checklist

| Spec 要求 | 对应 Task |
|-----------|-----------|
| attachment 表 | Task 1 |
| 文件系统存储 attachments/ | Task 3 |
| list/add/delete Commands | Task 4 |
| 文件选择器 + 粘贴 | Task 4 + Task 6 |
| 逐张删除 | Task 3 + Task 6 |
| 代办 + 过程双场景 | Task 7 |
| 未保存暂存模式 | Task 6 + Task 7 |
| 级联删除 | Task 3 + Task 4 |
| MIME/大小/魔数校验 | Task 3 |
| convertFileSrc 预览 | Task 4 + Task 6 |
| 数据目录迁移兼容 | 无需改动（copy_data_dir 已递归） |
