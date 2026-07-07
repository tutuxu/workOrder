# workOrder 图片附件设计

**日期：** 2026-07-07  
**状态：** 已批准（方案 A）  
**相关文档：** [数据目录设计](2026-07-07-settings-data-dir-design.md)、[backend.md](../../backend.md)

---

## 1. 背景与目标

### 1.1 需求

用户希望在以下两个场景中添加图片资源：

1. **代办（WorkOrder）**：创建/编辑代办时可附加多张图片（如需求截图、参考图）。
2. **处置过程（ProgressLog）**：追加或编辑过程记录时可附加多张图片（如调试截图、沟通记录）。

### 1.2 已确认决策

| 决策项 | 选择 |
|--------|------|
| 添加方式 | 本地文件选择器 + 剪贴板粘贴（Ctrl+V） |
| 数量限制 | 每条记录多张，不设上限 |
| 删除能力 | 支持逐张删除 |
| 存储方案 | 方案 A：文件系统 + 附件元数据表 |
| 附件范围 | 仅图片（jpeg / png / gif / webp / bmp） |

### 1.3 非目标（本期）

- 列表页缩略图预览
- 拖拽上传
- 图片编辑/标注
- 非图片附件（PDF、文档等）
- 云端同步

---

## 2. 方案对比

### 方案 A：文件系统 + 附件元数据表（采用）

图片存储在 `{data_dir}/attachments/`，数据库记录元数据。

| 优点 | 缺点 |
|------|------|
| 与现有 `data_dir` 及目录迁移设计一致 | 删除记录时需同步清理磁盘文件 |
| 数据库体积小、查询性能好 | 未保存时需前端暂存图片 |
| 支持逐张删除、预览、扩展 | |

### 方案 B：Base64 存入数据库

| 优点 | 缺点 |
|------|------|
| 备份只需拷贝 db | 库迅速膨胀，性能差 |
| 无文件路径管理 | 与 `attachments/` 约定冲突 |

### 方案 C：JSON 路径数组挂在主表

| 优点 | 缺点 |
|------|------|
| 改动最小 | 难维护元数据，逐张删除麻烦 |
| | 不符合分层架构 |

---

## 3. 架构

### 3.1 模块划分

```
src-tauri/src/
├── models/
│   └── attachment.rs          # Attachment 模型、OwnerType 枚举
├── services/
│   └── attachment_service.rs  # 增删查、文件读写、级联清理
├── commands/
│   └── attachment.rs          # Tauri Command 层
└── db/
    ├── schema.sql             # 新增 attachment 表
    └── migrate.rs             # 存量库迁移

src/
├── api/
│   └── attachments.ts         # 前端 API 封装
├── components/
│   └── AttachmentGallery.vue  # 可复用图片画廊组件
└── views/
    └── WorkOrderDetail.vue    # 嵌入画廊（代办 + 过程）
```

### 3.2 数据流

```
用户操作（选文件 / Ctrl+V）
    ↓
AttachmentGallery.vue
    ↓ invoke
commands/attachment.rs
    ↓
attachment_service.rs
    ├── 校验 mime / 大小
    ├── 写入 {data_dir}/attachments/{owner_type}/{owner_id}/{uuid}.ext
    └── INSERT attachment 表
    ↓
返回 Attachment 元数据
    ↓
前端用 convertFileSrc 显示缩略图
```

### 3.3 磁盘目录结构

```
{data_dir}/
├── workorder.db
└── attachments/
    ├── work_order/
    │   └── {work_order_id}/
    │       ├── a1b2c3d4.png
    │       └── e5f6g7h8.jpg
    └── progress_log/
        └── {progress_log_id}/
            └── ...
```

文件名使用 UUID + 原始扩展名，避免冲突与路径注入。

---

## 4. 数据模型

### 4.1 attachment 表

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

### 4.2 OwnerType 枚举

| 值 | 含义 |
|----|------|
| `work_order` | 代办事项附件 |
| `progress_log` | 处置过程附件 |

### 4.3 Attachment 结构体（Rust / TS 绑定）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `number \| null` | 主键 |
| `ownerType` | `OwnerType` | 归属类型 |
| `ownerId` | `number` | 归属记录 ID |
| `fileName` | `string` | 存储文件名（uuid.ext） |
| `originalName` | `string \| null` | 用户原始文件名 |
| `mimeType` | `string` | MIME 类型 |
| `fileSize` | `number` | 字节大小 |
| `createdAt` | `string` | ISO 时间戳 |
| `filePath` | `string` | 完整磁盘路径（供前端 `convertFileSrc` 使用） |

`filePath` 为派生字段，查询时由 `data_dir` + 相对路径拼接，不存入数据库。

---

## 5. 后端 API

### 5.1 Commands

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| `list_attachments` | `ownerType`, `ownerId` | `Attachment[]` | 按归属查询，按 `created_at` 升序 |
| `add_attachment_from_file` | `ownerType`, `ownerId`, `sourcePath` | `Attachment` | 从本地路径复制（文件选择器） |
| `add_attachment_from_bytes` | `ownerType`, `ownerId`, `fileName`, `mimeType`, `data` | `Attachment` | 从字节写入（剪贴板粘贴） |
| `delete_attachment` | `id` | `void` | 删除 DB 记录 + 磁盘文件 |

所有 Command 通过 `AppState.data_dir` 解析附件目录，复用现有 `ServiceError` 错误类型。

### 5.2 校验规则

| 规则 | 值 |
|------|-----|
| 允许 MIME | `image/jpeg`, `image/png`, `image/gif`, `image/webp`, `image/bmp` |
| 单文件大小上限 | 20 MB |
| 数量上限 | 无 |

校验失败返回 `ServiceError::Validation` 及中文提示。

### 5.3 级联删除

在现有 service 层扩展，不依赖数据库外键：

| 触发操作 | 清理行为 |
|----------|----------|
| `delete_work_order` | 删除该工单全部 `work_order` 附件 + 移除 `attachments/work_order/{id}/` 目录 |
| `delete_progress_log` | 删除该过程全部 `progress_log` 附件 + 移除 `attachments/progress_log/{id}/` 目录 |
| `delete_attachment` | 删除单条记录及对应文件；若目录为空则移除目录 |

### 5.4 数据目录迁移

现有 `copy_data_dir` 递归复制整个 `data_dir`，`attachments/` 子目录会随迁移一并复制，无需额外改动。`verify_migration` 仍以 `workorder.db` 完整性为准。

---

## 6. 前端 UI

### 6.1 AttachmentGallery 组件

**Props：**

| Prop | 类型 | 说明 |
|------|------|------|
| `ownerType` | `'work_order' \| 'progress_log'` | 归属类型 |
| `ownerId` | `number \| undefined` | 归属 ID；`undefined` 时为暂存模式 |
| `readonly` | `boolean` | 只读（仅展示，无添加/删除） |

**Events：**

| Event | 说明 |
|-------|------|
| `staged-change` | 暂存模式下，暂存图片列表变化 |

**功能：**

- 缩略图网格（固定高度，等比裁剪，`object-fit: cover`）
- 点击缩略图 → `n-image` 预览大图
- 「添加图片」按钮 → 调 `pick_attachment_file`（封装 dialog，filter 图片）
- 区域监听 `@paste` → 读取 `clipboardData.items` 中 `image/*`，调用 `add_attachment_from_bytes`
- 每张缩略图右上角删除按钮（`n-popconfirm` 确认）
- 底部提示：「支持选择图片或 Ctrl+V 粘贴」

**暂存模式（`ownerId` 为 `undefined`）：**

- 选文件/粘贴后保存在组件内 `stagedFiles: { file: File, previewUrl: string }[]`
- 父组件在保存拿到 ID 后，遍历暂存列表调用 `add_attachment_from_bytes`
- 上传完成后清空暂存并 reload 列表

### 6.2 嵌入位置

| 位置 | 模式 | 说明 |
|------|------|------|
| 代办表单 · 描述下方 | 新建暂存 / 已保存即时上传 | `ownerType=work_order` |
| 过程时间线 · 展开内容区 | 只读展示 | `readonly=true` |
| 追加/编辑过程表单 · 详细内容下方 | 新建暂存 / 编辑即时上传 | `ownerType=progress_log` |

### 6.3 图片显示

使用 Tauri `convertFileSrc(attachment.filePath)` 生成可加载 URL，配合 `<img>` 或 Naive UI `n-image`。

### 6.4 与现有保存流程的配合

| 场景 | 行为 |
|------|------|
| 新建代办 | 先 `createWorkOrder` 获 ID → 上传暂存图片 → 完成 |
| 编辑代办 | 添加后立即上传 |
| 新建过程 | 先 `addProgressLog` 获 ID → 上传暂存图片 |
| 编辑过程 | 添加后立即上传；切换编辑目标时清空过程表单暂存 |

---

## 7. 错误处理

| 场景 | 处理 |
|------|------|
| 非图片格式 | 提示「仅支持图片文件（JPEG、PNG、GIF、WebP、BMP）」 |
| 文件超过 20MB | 提示「图片大小不能超过 20MB」 |
| 磁盘写入失败 | 回滚已插入的 DB 记录（事务 + 删除已写文件） |
| 归属记录不存在 | `ServiceError::NotFound` |
| 删除失败 | 提示具体错误，不静默失败 |

---

## 8. 安全考量

- 文件名消毒：存储名仅用 UUID + 白名单扩展名，不使用用户提供的文件名作为磁盘名
- 路径校验：拼接路径时确认结果在 `attachments/` 目录内，防止路径穿越
- MIME 校验：除扩展名外，读取文件头魔数二次校验（防止伪装扩展名）

---

## 9. 测试计划

### 9.1 Rust 单元测试

- `attachment_service`：添加、列表、删除单条
- MIME / 大小校验拒绝非法输入
- 级联删除：删工单后附件记录与目录均不存在
- 路径穿越防护

### 9.2 手动测试

1. 编辑已有代办 → 选文件添加图片 → 缩略图显示 → 点击预览
2. 在图片区域 Ctrl+V 粘贴截图 → 成功添加
3. 删除单张图片 → 缩略图消失、磁盘文件删除
4. 新建代办 → 暂存 2 张图 → 保存 → 图片正确关联
5. 追加过程 → 暂存图片 → 追加成功 → 时间线展开可见
6. 删除代办 → 其附件目录被清理
7. 数据目录迁移 → 附件图片在新目录可正常预览

---

## 10. 实施清单（供实现计划引用）

1. 新增 `attachment` 表与 `migrate.rs` 迁移
2. 新增 `models/attachment.rs`
3. 新增 `services/attachment_service.rs`
4. 新增 `commands/attachment.rs` 并注册 specta
5. 扩展 `work_order_service` / `progress_log_service` 级联删除
6. 运行 `npm run bindings` 更新 `bindings.ts`
7. 新增 `src/api/attachments.ts`
8. 新增 `src/components/AttachmentGallery.vue`
9. 修改 `WorkOrderDetail.vue` 嵌入画廊
10. 补充 `docs/api/commands.md` 附件 API 说明
