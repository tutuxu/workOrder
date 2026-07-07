# Tauri Command API 参考

**自动生成类型：** [`src/bindings.ts`](../../src/bindings.ts)（tauri-specta）  
**手写封装：** [`src/api/workOrders.ts`](../../src/api/workOrders.ts)、[`src/api/progressLogs.ts`](../../src/api/progressLogs.ts)

> 本文档描述 IPC invoke 接口。workOrder **无 HTTP REST API**，因此不使用 Swagger/OpenAPI；类型安全由 tauri-specta 保证。

---

## 通用约定

### 命名

| Rust command | TypeScript 方法 | 说明 |
|--------------|-----------------|------|
| `snake_case` | `camelCase` | Specta 自动转换 |

### 错误

Command 返回 `Result<T, String>`。失败时 `invoke` 抛出，错误体为 JSON：

```json
{"code":"NOT_FOUND","message":"Work order not found: 42"}
```

| code | 含义 |
|------|------|
| `NOT_FOUND` | 资源不存在 |
| `VALIDATION` | 参数/业务校验失败 |
| `DATABASE` | SQLite 错误 |
| `IO` | 文件系统错误 |

### 共享类型

定义于 `bindings.ts`，与 Rust `models/` 一致：

```typescript
type WorkOrderStatus = "NOT_STARTED" | "IN_PROGRESS" | "WAITING_REPLY" | "COMPLETED";

type WorkOrder = {
  id: number | null;
  title: string;
  description: string | null;
  status: WorkOrderStatus;
  priority: number;
  waitingFor: string | null;
  waitingReason: string | null;
  dueDate: string | null;      // ISO 日期时间字符串
  createdAt: string;
  updatedAt: string;
};

type WorkOrderInput = {
  title: string;
  description: string | null;
  status: WorkOrderStatus;
  waitingFor: string | null;
  waitingReason: string | null;
  dueDate: string | null;
};

type ProgressLog = {
  id: number | null;
  workOrderId: number;
  content: string;
  createdAt: string;
};
```

---

## 工单 Command

### `list_work_orders`

按状态筛选工单列表。

| 项 | 说明 |
|----|------|
| **TS 方法** | `commands.listWorkOrders(statuses, includeCompleted)` |
| **参数** | `statuses: string[]` — 状态过滤，空数组表示全部<br>`includeCompleted: boolean` — `false` 时排除 `COMPLETED` |
| **返回** | `WorkOrder[]`，按 `priority` 升序、`updatedAt` 降序 |
| **错误** | `DATABASE` |

```typescript
await commands.listWorkOrders(["IN_PROGRESS"], false);
```

---

### `get_work_order`

| 项 | 说明 |
|----|------|
| **TS 方法** | `commands.getWorkOrder(id)` |
| **参数** | `id: number` |
| **返回** | `WorkOrder` |
| **错误** | `NOT_FOUND`, `DATABASE` |

---

### `create_work_order`

| 项 | 说明 |
|----|------|
| **TS 方法** | `commands.createWorkOrder(input)` |
| **参数** | `input: WorkOrderInput` — `title` 必填 |
| **返回** | 新建 `WorkOrder`（含自动分配的 `priority`） |
| **副作用** | 若状态为 `WAITING_REPLY`，自动追加进度日志 |
| **错误** | `VALIDATION`, `DATABASE` |

---

### `update_work_order`

| 项 | 说明 |
|----|------|
| **TS 方法** | `commands.updateWorkOrder(id, input)` |
| **参数** | `id: number`, `input: WorkOrderInput` |
| **返回** | 更新后的 `WorkOrder` |
| **副作用** | 「待回复」进入或 waiting 信息变更时自动追加进度日志 |
| **错误** | `NOT_FOUND`, `VALIDATION`, `DATABASE` |

---

### `delete_work_order`

| 项 | 说明 |
|----|------|
| **TS 方法** | `commands.deleteWorkOrder(id)` |
| **参数** | `id: number` |
| **返回** | `void` |
| **副作用** | 级联删除该工单全部进度日志 |
| **错误** | `NOT_FOUND`, `DATABASE` |

---

### `update_priorities`

拖拽排序后批量更新 priority。

| 项 | 说明 |
|----|------|
| **TS 方法** | `commands.updatePriorities(orderedIds)` |
| **参数** | `orderedIds: number[]` — 按新顺序排列的工单 id |
| **返回** | `void` |
| **规则** | 下标 0 对应 `priority = 0`，依次递增 |
| **错误** | `NOT_FOUND`（任一 id 无效）, `DATABASE` |

---

### `is_work_order_overdue`

纯计算，不访问数据库。

| 项 | 说明 |
|----|------|
| **TS 方法** | `commands.isWorkOrderOverdue(workOrder)` |
| **参数** | `workOrder: WorkOrder` |
| **返回** | `boolean` |
| **规则** | 无 `dueDate` 或已 `COMPLETED` → `false`；否则与当前 UTC 比较 |

---

## 进度日志 Command

### `list_progress_logs`

| 项 | 说明 |
|----|------|
| **TS 方法** | `commands.listProgressLogs(workOrderId)` |
| **参数** | `workOrderId: number` |
| **返回** | `ProgressLog[]`，按 `createdAt` 降序 |
| **错误** | `DATABASE` |

---

### `add_progress_log`

| 项 | 说明 |
|----|------|
| **TS 方法** | `commands.addProgressLog(workOrderId, content)` |
| **参数** | `workOrderId: number`, `content: string`（非空） |
| **返回** | 新建 `ProgressLog` |
| **错误** | `NOT_FOUND`, `VALIDATION`, `DATABASE` |

---

### `update_progress_log`

| 项 | 说明 |
|----|------|
| **TS 方法** | `commands.updateProgressLog(logId, workOrderId, content)` |
| **参数** | `logId`, `workOrderId`, `content`（非空） |
| **返回** | 更新后的 `ProgressLog` |
| **错误** | `NOT_FOUND`, `VALIDATION`, `DATABASE` |

---

### `delete_progress_log`

| 项 | 说明 |
|----|------|
| **TS 方法** | `commands.deleteProgressLog(logId, workOrderId)` |
| **参数** | `logId`, `workOrderId` |
| **返回** | `void` |
| **错误** | `NOT_FOUND`, `VALIDATION`, `DATABASE` |

---

## 维护说明

修改 Rust Command 或 Model 后，须重新生成绑定：

```bash
npm run bindings
```

并将更新后的 `src/bindings.ts` 一并提交。完整命令说明见 [开发者常用命令](../dev-commands.md)。
