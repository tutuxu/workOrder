# workOrder 代办状态与自定义字段设计

**日期：** 2026-07-07  
**状态：** 已批准  
**相关文档：** [backup-restore-design](2026-07-07-backup-restore-design.md)、[settings-data-dir-design](2026-07-07-settings-data-dir-design.md)

---

## 1. 目标

用户可在设置页 **「代办状态」** 中完全自定义工单状态及其对应的信息栏（字段），包括字段类型与必填规则。状态配置存储于数据目录，随现有 ZIP 备份/恢复自动包含。旧版导出的代办数据须能在新版中正常恢复，或通过独立迁移脚本完成升级。

## 2. 需求摘要

| 项 | 决定 |
|----|------|
| 状态自由度 | 完全自定义：增删改、调整排序 |
| 字段 | 可新增自定义字段；类型 `text` / `textarea` / `date` |
| 字段值存储 | `work_order.extra_fields` JSON 列 |
| 必填规则 | 逐字段配置 `required: true/false` |
| 状态特殊语义 | 无：所有状态一视同仁 |
| 移除行为 | 切到「待回复」不再自动写进度；「已完成」不再豁免逾期 |
| 列表筛选 | 移除「显示已完成」；仅保留状态多选框（不选 = 全部） |
| 设置页区块名 | **代办状态** |
| UI 展示形式 | 保持现有 radio / checkbox / tag 组件与布局 |
| 旧版兼容 | 恢复时自动 DB 迁移 + 补全配置；提供 CLI 迁移脚本 |

## 3. 配置模型

配置文件路径：`{data_dir}/status_config.json`

```json
{
  "version": 1,
  "statuses": [
    {
      "id": "NOT_STARTED",
      "label": "未处置",
      "order": 0,
      "fields": []
    },
    {
      "id": "WAITING_REPLY",
      "label": "待回复",
      "order": 2,
      "fields": [
        {
          "key": "waitingFor",
          "label": "等待对象",
          "type": "text",
          "required": true
        },
        {
          "key": "waitingReason",
          "label": "等待原因",
          "type": "textarea",
          "required": false
        }
      ]
    }
  ]
}
```

### 3.1 字段说明

- **status.id**：唯一标识，创建后不可修改；允许英文字母、数字、下划线。
- **status.label**：显示名称，可修改。
- **status.order**：排序序号，从 0 起连续递增。
- **field.key**：同一 status 内唯一；创建后不可修改。
- **field.type**：`text` | `textarea` | `date`。
- **field.required**：该状态下保存工单时是否必填。

### 3.2 校验规则

- status.id 不可重复。
- field.key 在同一 status 内不可重复。
- order 值允许间隙，渲染时按 order 升序排列。
- 删除 status 时不修改历史工单的 status 值。

### 3.3 未知状态展示

工单或进度记录的 status 值在配置中找不到时，显示为 **`未知状态 ({id})`**，其中 `{id}` 为数据库中的原始 status 字符串。

## 4. 数据存储

### 4.1 数据库变更

```sql
ALTER TABLE work_order ADD COLUMN extra_fields TEXT;
```

- `status` 列保持 `VARCHAR(50)`，存 status.id 字符串。
- `extra_fields` 存 JSON 对象，例如 `{"waitingFor":"联调方","waitingReason":"等待接口确认"}`。
- `date` 类型字段存 ISO 8601 字符串（与现有 `dueDate` 序列化方式一致）。
- `waiting_for` / `waiting_reason` 列保留只读兼容，新读写以 `extra_fields` 为准。

### 4.2 类型变更

- Rust：`WorkOrder.status` 由枚举改为 `String`；新增 `extra_fields: Option<serde_json::Value>`。
- 前端：`WorkOrderStatus` 联合类型改为 `string`；运行时从配置获取选项列表。

## 5. 行为变更

| 现有行为 | 变更 |
|----------|------|
| 4 个硬编码状态 | 读取 `status_config.json` |
| `WAITING_REPLY` 显示等待字段 | 按当前 status 的 fields 配置动态渲染 |
| 切到待回复自动写进度 | **移除** |
| 已完成不参与逾期 | **移除**：有 `dueDate` 且已过期即标红 |
| 「显示已完成」勾选 | **移除** |
| `includeCompleted` API 参数 | **移除** |

## 6. UI 设计

### 6.1 使用侧（保持现有形式）

| 位置 | 组件 | 数据源 |
|------|------|--------|
| 工单详情 · 状态 | `n-radio-group` + `n-radio` | 配置 statuses |
| 列表 · 状态筛选 | `n-checkbox-group` + `n-checkbox` | 配置 statuses |
| 进度记录 · 状态 | `n-radio-group` + `n-radio` | 配置 statuses |
| 列表行 · 状态 | 纯文本 | `statusLabel()` |
| 进度折叠头 · 状态 | `n-tag` | `statusLabel()` |
| 状态专属字段 | `n-form-item` 条件块 | 当前 status 的 fields |

字段组件映射：

- `text` → `n-input`
- `textarea` → `n-input type="textarea"`
- `date` → `n-date-picker type="datetime"`

### 6.2 设置页「代办状态」

在 `Settings.vue` 新增第三个区块（位于「数据备份」之后或之前）：

- 状态列表：增删、改 label、拖拽排序。
- 选中状态后管理其字段列表：增删、改 label、选类型、设必填。
- 保存后立即生效，无需重启。
- 删除状态前确认：「已有工单使用此状态时将显示为未知状态」。

## 7. 后端

### 7.1 新增模块

**`status_config_service.rs`**

- `load_config(data_dir)` / `save_config(data_dir, config)`
- `ensure_default_config(data_dir)` — 文件不存在时写入默认 4 状态配置
- 校验逻辑（id 唯一、field key 唯一等）

**Tauri 命令**

- `get_status_config` → `StatusConfig`
- `save_status_config(config)` → 校验后写入

### 7.2 工单服务变更

- 保存时按当前 status 的配置校验必填字段。
- 搜索扩展：匹配 `title`、`description`、`waiting_for`、`waiting_reason`（过渡期）及 `extra_fields` JSON 文本。
- 移除 `append_waiting_reply_progress_log` 及相关函数。
- `is_overdue`：仅判断 `due_date` 是否存在且早于当前时间。
- `find_by_statuses`：移除 `include_completed` 参数。

## 8. 旧版数据兼容与迁移

### 8.1 自动迁移（主路径）

在 `open_connection()` 中，于现有迁移之后执行：

```
open_connection
  → migrate_progress_log（已有）
  → migrate_attachment（已有）
  → migrate_extra_fields（新增）
```

应用启动、配置加载时：

```
ensure_status_config（新增，文件级）
```

**`migrate_extra_fields`**

1. 若 `extra_fields` 列不存在则 `ALTER TABLE` 添加。
2. 对 `extra_fields IS NULL OR extra_fields = ''` 的行，将非空的 `waiting_for` / `waiting_reason` 写入 JSON。
3. 幂等：已有 `extra_fields` 的行跳过。

**`ensure_status_config`**

- `status_config.json` 不存在时，写入默认配置（4 状态 + 待回复字段）。
- 不覆盖已有文件。

### 8.2 旧版 ZIP 备份恢复

旧版备份仅含 `workorder.db` + `attachments/`，无 `status_config.json`。

恢复流程不变：解压 → 校验 manifest + DB → 替换 data_dir → 重启。

重启后：

1. `migrate_extra_fields` 迁移等待字段至 JSON。
2. `ensure_status_config` 生成默认配置。
3. 工单 status 值（如 `NOT_STARTED`）与默认配置 id 匹配，数据完整可用。

备份校验 **不要求** `status_config.json` 存在。

### 8.3 独立迁移脚本（兜底）

供直接拷贝旧 data 目录（非 ZIP）时使用：

```bash
cargo run --bin workorder-migrate -- --data-dir "D:\path\to\data"
```

与启动时共用同一套 Rust 迁移函数，输出摘要（迁移行数、是否生成配置等）。

## 9. 备份 / 恢复

无需修改 ZIP 打包逻辑：`status_config.json` 位于 `data_dir` 内，自动包含。

恢复后若 config 随备份存在则直接使用；若缺失则 `ensure_status_config` 兜底。

`manifest.format_version` 可 bump 至 2 作为文档标记，不阻断旧版备份恢复。

## 10. 默认配置（首次安装 / 兜底）

与当前硬编码等价：

| id | label | fields |
|----|-------|--------|
| NOT_STARTED | 未处置 | — |
| IN_PROGRESS | 处置中 | — |
| WAITING_REPLY | 待回复 | waitingFor (text, 必填), waitingReason (textarea, 选填) |
| COMPLETED | 已完成 | — |

## 11. 非目标

- 状态下拉选项以外的展示形式变更（如改为 select）。
- 字段类型扩展（number、select、多选等）。
- 状态语义标记（终态、自动记进度等）。
- 删除 status 时批量改写历史工单 status。
- 配置版本 diff / 合并。

## 12. 测试要点

- 默认配置生成与读写 roundtrip。
- 自定义 status + field 保存校验（必填、类型）。
- 删除 status 后旧工单显示「未知状态」。
- 旧版 DB（无 extra_fields）迁移后字段值正确。
- 旧版 ZIP 备份恢复后工单与等待字段完整。
- CLI 迁移脚本与自动迁移结果一致。
- 备份 ZIP 含 status_config.json，恢复后配置保留。
- 移除 includeCompleted 后列表筛选行为正确。
- 逾期判定不再排除任何 status。
