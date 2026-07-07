# workOrder 设置 — 数据备份与恢复设计

**日期：** 2026-07-07  
**状态：** 已批准（方案 A — ZIP 单文件备份）  
**相关文档：** [settings-data-dir-design](2026-07-07-settings-data-dir-design.md)

---

## 1. 目标

在设置页提供 **备份 / 恢复** 功能，将当前数据目录（`workorder.db` + `attachments/`）打包为单个 ZIP 文件，用于换机或灾难恢复。恢复时整体替换当前数据，完成后重启应用。

## 2. 备份包格式

```
workorder-backup-YYYYMMDD-HHmmss.zip
├── manifest.json
├── workorder.db
└── attachments/
    └── ...
```

`manifest.json`：`format_version`、`app_version`、`exported_at`。

## 3. 流程

**备份：** WAL checkpoint → 打包 data_dir → 写入 manifest → 保存至用户选择路径。

**恢复：** 选 ZIP → 二次确认 → 解压至临时目录 → 校验 → 写入 `settings.json` 的 `pending_restore_from` → 重启 → 启动时替换 data_dir 内容 → 清除 pending → 删除临时目录。

## 4. UI

设置弹窗「数据存储位置」下方新增「数据备份」区块：「备份...」「恢复...」（危险操作）。

## 5. 非目标

- JSON/CSV 结构化导出（v2.1）
- 备份加密
- 导入前自动备份当前数据
- 合并式恢复
