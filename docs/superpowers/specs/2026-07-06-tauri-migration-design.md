# workOrder Tauri 迁移设计

**日期：** 2026-07-06  
**状态：** 待评审  
**相关文档**：[plan/技术选型.md](../plan/技术选型.md)、[plan/实现计划.md](../plan/实现计划.md)、[plan/版本规划.md](../plan/版本规划.md)

---

## 1. 背景与目标

### 1.1 问题

当前桌面端采用 **Spring Boot + Vaadin + JavaFX WebView + jpackage** 架构。启动时 CPU/内存负担大，主要原因：

- JVM 冷启动（Spring Boot、Hibernate、Flyway、Vaadin）
- JavaFX 运行时 + 内嵌 WebKit 与 Vaadin 形成双渲染引擎
- localhost HTTP 侧车模式带来额外开销

### 1.2 目标

| 指标 | 当前 | 目标 |
|------|------|------|
| 启动时间 | 5–15s | < 1s |
| 空闲内存 | 300–500MB | 50–80MB |
| 安装包体积 | ~100MB+（含 JRE） | ~10–20MB |
| 空闲 CPU | 可能持续偏高 | 接近 0 |

### 1.3 范围

**保留：**

- 业务功能（列表、筛选、拖拽排序、详情编辑、过程时间线）
- SQLite 表结构（`V1__init_schema.sql`）
- 现有 `data/workorder.db` 数据文件
- 数据目录约定（默认 `./data`）

**移除：**

- Spring Boot、Vaadin、JavaFX
- JPA / Hibernate / Flyway
- `DesktopLauncher`、`WebViewShell`、jpackage 打包流程
- 全部 `src/main/java/**` 与 `pom.xml`

**新增：**

- Tauri 2 桌面壳
- Vue 3 前端
- Rust Service 层 + rusqlite

---

## 2. 技术选型

| 层次 | 选型 | 说明 |
|------|------|------|
| 桌面壳 | Tauri 2 | 系统 WebView2，单进程 |
| 前端 | Vue 3 + Vite | 组件化 UI |
| UI 库 | Naive UI | Vue 3 原生，现代桌面风格 |
| 拖拽 | vue-draggable-plus | 列表行拖拽排序 |
| 日期 | dayjs | 格式化与解析 |
| 后端 | Rust | Tauri Commands + Service 层 |
| 数据库 | rusqlite（bundled） | 直接读写 SQLite 文件 |
| 序列化 | serde + chrono | 前后端数据传输 |
| 打包 | `npm run tauri build` | 生成 exe / MSI |

---

## 3. 架构

```
┌─────────────────────────────────────────┐
│              workOrder.exe              │
│  ┌─────────────┐    ┌────────────────┐  │
│  │  WebView2   │───►│   Vue 3 前端    │  │
│  └─────────────┘    └───────┬────────┘  │
│                             │ invoke    │
│                    ┌────────▼────────┐  │
│                    │ Tauri Commands  │  │
│                    └────────┬────────┘  │
│                    ┌────────▼────────┐  │
│                    │  Rust Services  │  │
│                    └────────┬────────┘  │
│                    ┌────────▼────────┐  │
│                    │    rusqlite     │  │
│                    └────────┬────────┘  │
└─────────────────────────────┼────────────┘
                              │
                    ┌─────────▼─────────┐
                    │  ./data/workorder.db │
                    └───────────────────┘
```

**关键设计：**

- 无 JVM、无 HTTP 服务、无端口轮询
- 前端通过 `invoke()` 调用 Rust Commands
- 业务逻辑集中在 Rust Service 层，Commands 仅做参数校验与转发
- 数据库连接在 Tauri 启动时初始化，通过 `manage()` 注入状态

---

## 4. 目录结构

迁移完成后仓库结构：

```
workOrder/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── work_order.rs
│   │   │   └── progress_log.rs
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── work_order_service.rs
│   │   │   └── progress_log_service.rs
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── connection.rs
│   │   │   └── schema.sql
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── work_order.rs
│   │   │   ├── progress_log.rs
│   │   │   └── work_order_status.rs
│   │   └── error.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── icons/
├── src/                          # Vue 3
│   ├── main.ts
│   ├── App.vue
│   ├── views/
│   │   ├── WorkOrderList.vue
│   │   └── WorkOrderDetail.vue
│   ├── components/
│   ├── composables/
│   │   └── useWorkOrders.ts
│   ├── api/
│   │   ├── workOrders.ts
│   │   └── progressLogs.ts
│   └── styles/
│       └── main.css
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
├── data/
│   └── workorder.db              # 开发数据（gitignore）
├── scripts/
│   └── package-windows.bat       # 改为调用 tauri build
└── docs/
    └── superpowers/specs/
        └── 2026-07-06-tauri-migration-design.md
```

**删除：**

- `pom.xml`
- `src/main/java/**`
- `src/test/java/**`
- `src/main/resources/application.properties`（配置迁入 Tauri）
- `META-INF/MANIFEST.MF`
- Java 相关 `.cursor/rules` 可保留或后续清理

---

## 5. 数据层

### 5.1 表结构

沿用 `V1__init_schema.sql`，在 `src-tauri/src/db/schema.sql` 中维护，启动时执行 `CREATE TABLE IF NOT EXISTS` 及索引创建。

### 5.2 数据文件路径

解析优先级：

1. 环境变量 `WORKORDER_DATA_DIR`
2. Tauri 配置中的 `workorder.data.dir`
3. 默认可执行文件同目录下的 `data/`（开发时为项目根 `data/`）

数据库文件：`{data_dir}/workorder.db`

### 5.3 日期时间

- Rust 使用 `chrono::NaiveDateTime`
- SQLite 存储为 ISO 8601 字符串（`YYYY-MM-DDTHH:MM:SS`）
- 与现有 Java 写入的数据兼容（Java `LocalDateTime` 默认格式可读）

### 5.4 连接管理

- 应用启动时打开单个 `rusqlite::Connection`
- 通过 `tauri::State<AppState>` 共享（内含 `Mutex<Connection>`）
- 桌面单用户场景，无需连接池

---

## 6. Rust 层设计

### 6.1 模型

```rust
// WorkOrderStatus
enum WorkOrderStatus { NotStarted, InProgress, WaitingReply, Completed }
// 序列化为 "NOT_STARTED" 等，与 DB 存量一致

// WorkOrder
struct WorkOrder {
    id: Option<i64>,
    title: String,
    description: Option<String>,
    status: WorkOrderStatus,
    priority: i32,
    waiting_for: Option<String>,
    waiting_reason: Option<String>,
    due_date: Option<NaiveDateTime>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

// ProgressLog
struct ProgressLog {
    id: Option<i64>,
    work_order_id: i64,
    content: String,
    created_at: NaiveDateTime,
}
```

### 6.2 Tauri Commands

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| `list_work_orders` | `statuses: Vec<String>`, `include_completed: bool` | `Vec<WorkOrder>` | 状态筛选列表 |
| `get_work_order` | `id: i64` | `WorkOrder` | 单条详情 |
| `create_work_order` | `input: WorkOrderInput` | `WorkOrder` | 新建 |
| `update_work_order` | `id: i64`, `input: WorkOrderInput` | `WorkOrder` | 更新 |
| `delete_work_order` | `id: i64` | `()` | 删除（级联删过程） |
| `update_priorities` | `ordered_ids: Vec<i64>` | `()` | 拖拽后批量更新 |
| `list_progress_logs` | `work_order_id: i64` | `Vec<ProgressLog>` | 时间线 |
| `add_progress_log` | `work_order_id: i64`, `content: String` | `ProgressLog` | 追加 |
| `update_progress_log` | `log_id: i64`, `work_order_id: i64`, `content: String` | `ProgressLog` | 编辑 |
| `delete_progress_log` | `log_id: i64`, `work_order_id: i64` | `()` | 删除 |

### 6.3 业务规则（从 Java 移植）

**WorkOrderService：**

- `validate_title`：标题非空
- `create`：默认状态 `NOT_STARTED`，priority = max + 1
- `update`：更新字段，刷新 `updated_at`
- `delete`：先删 progress_log，再删 work_order
- `find_by_statuses`：多状态 OR 筛选；`include_completed=false` 时排除 `COMPLETED`
- `update_priorities`：按传入 ID 顺序重写 priority（0, 1, 2, …）
- `is_overdue`：有 due_date、非 COMPLETED、due_date < now
- `append_waiting_reply_progress_log`：进入待回复或等待信息变更时自动写过程

**ProgressLogService：**

- `add_log` / `update_log`：内容非空
- `delete_log`：校验 log 属于指定 work_order
- `find_by_work_order_id`：按 `created_at DESC` 排序

### 6.4 错误处理

```rust
enum ServiceError {
    NotFound(String),
    Validation(String),
    Database(rusqlite::Error),
}
```

Commands 将 `ServiceError` 映射为前端可读的 `{ code, message }` 字符串。

---

## 7. Vue 3 前端设计

### 7.1 页面结构

**WorkOrderList.vue（主页面）**

- 工具栏：新建按钮、状态 Checkbox 筛选、显示已完成 Checkbox
- 列表：标题、状态、计划完成时间、最后更新
- 拖拽排序：vue-draggable-plus
- 过期行：`.overdue-row` 样式（红色背景）
- 点击行：打开详情模态框

**WorkOrderDetail.vue（模态框）**

- 表单：标题、描述、状态（Radio）、计划完成时间、等待对象/原因（待回复时显示）
- 处置过程时间线：时间 + 内容 + 编辑/删除
- 底部：追加过程输入框、保存、删除

### 7.2 API 封装

```typescript
// src/api/workOrders.ts
import { invoke } from '@tauri-apps/api/core';

export function listWorkOrders(statuses: string[], includeCompleted: boolean) {
  return invoke<WorkOrder[]>('list_work_orders', { statuses, includeCompleted });
}
// ... 其余 commands 同理
```

### 7.3 状态管理

- 列表页用 `ref` + composable `useWorkOrders`
- 无需 Pinia（功能规模小）
- 保存/删除后刷新列表

---

## 8. 功能对照表

| 功能 | 现 Java 实现 | 新实现 |
|------|-------------|--------|
| 列表展示 | Vaadin Grid | Naive UI Table / 自定义列表 |
| 状态筛选 | Checkbox 组 | n-checkbox-group |
| 拖拽排序 | Grid DragDrop | vue-draggable-plus |
| 过期高亮 | classNameGenerator | CSS class 绑定 |
| 详情编辑 | Dialog | n-modal |
| 状态选择 | RadioButtonGroup | n-radio-group |
| 过程时间线 | VerticalLayout | 列表 + 操作按钮 |
| 确认删除 | ConfirmDialog | n-popconfirm / n-dialog |
| 通知 | Notification | useMessage |

---

## 9. 构建与发布

### 9.1 开发环境要求

- Node.js 18+
- Rust（rustup）
- Windows：WebView2（Win10+ 通常已预装）
- Visual Studio Build Tools（Windows Rust 编译）

### 9.2 命令

```bash
# 安装依赖
npm install

# 开发（热重载）
npm run tauri dev

# 生产构建
npm run tauri build
```

### 9.3 输出

- `src-tauri/target/release/workorder.exe`
- `src-tauri/target/release/bundle/msi/workOrder_1.0.0_x64.msi`

### 9.4 package-windows.bat

替换为：

```bat
npm run tauri build
xcopy /E /I /Y "src-tauri\target\release\bundle\nsis\..." "release\"
```

---

## 10. 测试策略

### 10.1 Rust 单元测试

- `work_order_service` 测试：create、update、delete、find_by_statuses、update_priorities、is_overdue、waiting_reply 自动日志
- `progress_log_service` 测试：add、update、delete、归属校验
- 使用 `:memory:` SQLite 或临时文件

### 10.2 前端

- MVP 阶段手工验证
- 后续可加 Vitest 测 composable

### 10.3 数据兼容测试

- 用现有 `data/workorder.db` 启动新应用，验证列表、详情、过程记录正确显示

---

## 11. 迁移步骤（概要）

1. **脚手架**：`npm create tauri-app` 初始化 Tauri 2 + Vue 3 项目（原地，覆盖 Java 结构）
2. **Rust 数据层**：schema、connection、models
3. **Rust Service**：移植业务逻辑 + 单元测试
4. **Tauri Commands**：注册并连通 Service
5. **Vue 列表页**：筛选、展示、拖拽
6. **Vue 详情页**：表单、时间线、CRUD
7. **数据路径**：对齐 `./data` 约定
8. **删除 Java 代码**：pom.xml、src/main/java、旧打包脚本
9. **更新 .gitignore**：加入 `src-tauri/target/`、`node_modules/`
10. **验证**：`tauri build` 生成 exe，用现有 DB 测试

---

## 12. 风险与应对

| 风险 | 概率 | 应对 |
|------|------|------|
| Rust 学习曲线 | 中 | Service 层逻辑从 Java 直译；保持薄 Commands |
| 工具链安装 | 低 | 文档记录 rustup + Node 安装步骤 |
| 日期格式不兼容 | 低 | 启动时用现有 DB 验证；统一 ISO 8601 |
| WebView2 缺失 | 低 | Tauri NSIS 安装包捆绑 WebView2 bootstrapper |
| 迁移期间无法回退 | 中 | 迁移前 git tag 标记 Java 版本 |

---

## 13. 不在本次范围

- 多用户 / 网络同步
- REST API 暴露
- 自动更新（Tauri updater）
- macOS / Linux 打包（可后续扩展）

---

## 14. 决策记录

| 决策 | 选项 | 结论 |
|------|------|------|
| 迁移策略 | 原地替换 / 并行目录 / 分阶段 | **原地替换** |
| 后端语言 | Rust / TypeScript / Java 侧车 | **Rust** |
| 前端框架 | React / Vue / Svelte | **Vue 3** |
| UI 库 | Naive UI / Element Plus | **Naive UI** |
| 数据库 | rusqlite 直读 | **沿用现有 SQLite 文件** |
