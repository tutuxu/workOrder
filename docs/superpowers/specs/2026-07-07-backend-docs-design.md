# 后端文档方案设计

**日期：** 2026-07-07  
**状态：** 已实施  
**方案：** D + tauri-specta

---

## 目标

为 Rust 后端引入完整文档体系：

1. **Markdown 架构文档** — `docs/backend.md`
2. **Command API 参考** — `docs/api/commands.md`
3. **Rust 内联文档** — `///` / `//!` + `cargo doc`
4. **类型自动生成** — tauri-specta → `src/bindings.ts`

## 技术选型

| 组件 | 选型 | 说明 |
|------|------|------|
| 类型绑定 | tauri-specta 2.0.0-rc.25 | 从 Rust Command 生成 TS |
| Specta | 2.0.0-rc.25 + chrono | Model 派生 `Type` |
| 错误模式 | `ErrorHandlingMode::Throw` | 与现有 invoke 行为一致 |
| i64 映射 | `dangerously_cast_bigints_to_number` | SQLite id 在安全整数范围内 |

## 不在范围

- OpenAPI / Swagger（无 HTTP API）
- cargo doc CI 自动化

## 验证

- `npm run bindings` 成功生成 `src/bindings.ts`
- `npx vue-tsc --noEmit` 通过
- `cargo doc --no-deps` 无错误
