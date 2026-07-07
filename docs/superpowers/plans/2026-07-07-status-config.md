# 代办状态与自定义字段 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 用户可在设置页「代办状态」中完全自定义状态及 per-status 字段；配置随 data_dir 备份；旧版数据自动迁移。

**Architecture:** `status_config.json` 存配置；`work_order.extra_fields` JSON 存字段值；启动时 DB/文件迁移；前端 radio/checkbox 展示不变。

**Tech Stack:** Rust (Tauri/rusqlite)、Vue 3、Naive UI、vue-draggable-plus

**Spec:** [2026-07-07-status-config-design.md](../specs/2026-07-07-status-config-design.md)

---

## 任务清单

- [x] Task 1: `StatusConfig` 模型 + `status_config_service` + 校验/默认配置
- [x] Task 2: `migrate_extra_fields` + schema `extra_fields` 列
- [x] Task 3: `WorkOrder`/`ProgressLog` status 改 String + `extra_fields`
- [x] Task 4: `work_order_service` 重构（移除 includeCompleted、待回复自动进度、Completed 免逾期）
- [x] Task 5: Tauri commands `get_status_config` / `save_status_config`
- [x] Task 6: CLI `workorder-migrate --data-dir`
- [x] Task 7: 前端 `useStatusConfig` + 列表/详情动态渲染
- [x] Task 8: 设置页「代办状态」`StatusConfigPanel`
- [x] Task 9: 导出 bindings + `npm run build` + `cargo build`

## 验证

```bash
cd src-tauri && cargo build
cd src-tauri && cargo run --bin export_bindings
cd .. && npm run build
cd src-tauri && cargo run --bin workorder-migrate -- --data-dir ./data
```

## 迁移脚本

```bash
cargo run --bin workorder-migrate -- --data-dir "D:\path\to\data"
```
