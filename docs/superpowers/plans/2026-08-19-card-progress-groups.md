# 卡片过程分组展示与拖拽修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 卡片视图按状态分组展示处置过程（仅标题+状态，整卡最多 6 条，未展全组标「还有更多」）；列表 API 附带过程摘要；修复代办列表与状态配置拖拽，并为状态配置增加上移/下移。

**Architecture:** `WorkOrder` 增加 `progressSummaries`；主清单 `find_by_filters` 批量挂载精简过程。前端 `groupCardProgress` 按状态 `order` 分组并截断到 6 条。Tauri 关闭 `dragDropEnabled` 以恢复 HTML5 DnD；状态列表保留拖拽并加上下移。

**Tech Stack:** Rust / rusqlite / Tauri 2 / specta、Vue 3、Naive UI、vue-draggable-plus、TypeScript

**Spec:** [2026-08-19-card-progress-groups-design.md](../specs/2026-08-19-card-progress-groups-design.md)

## Global Constraints

- 过程行只显示标题 + 状态；整卡最多 **6** 条；未展全文案固定为「还有更多」（无数字）
- 截断与分组只在前端；后端返回该工单下**全部**摘要（字段精简）
- 主清单必须填充 `progressSummaries`；`getWorkOrder` / 回收站列表可为 `[]`
- 列表模式不展示过程区；回收站不依赖摘要
- 关闭窗口 `dragDropEnabled` 可接受（附件用选文件/粘贴）
- 本仓库无前端单测框架：前端 Task 用 `npm run build`（含 `vue-tsc --noEmit`）+ 文内手工检查
- **不要执行 git commit / push**（用户要求不做提交）；计划中无 Commit 步骤

---

## File Structure

| 文件 | 职责 |
|------|------|
| `src-tauri/src/models/progress_log.rs` | 新增 `ProgressLogSummary` |
| `src-tauri/src/models/work_order.rs` | `WorkOrder.progress_summaries` |
| `src-tauri/src/services/work_order_service.rs` | `row_to_work_order` 默认 `[]`；批量 `attach_progress_summaries`；`find_by_filters` 挂载 |
| `src/bindings.ts` | specta 重新导出（`npm run bindings`） |
| `src/types.ts` | 如需 re-export `ProgressLogSummary` |
| `src/utils/cardProgressGroups.ts` | 分组 + 截断纯函数 |
| `src/views/WorkOrderList.vue` | 卡片过程区 |
| `src/styles/main.css` | 过程区分组样式 |
| `src/components/StatusConfigPanel.vue` | 上移 / 下移 |
| `src-tauri/tauri.conf.json` | `dragDropEnabled: false` |

---

## 任务清单

- [x] Task 1: `ProgressLogSummary` + `WorkOrder.progress_summaries` 模型
- [x] Task 2: 主清单批量挂载过程摘要（含 cargo 测试）
- [x] Task 3: 导出 bindings
- [x] Task 4: `groupCardProgress` 前端纯函数
- [x] Task 5: 卡片过程区 UI + CSS
- [x] Task 6: 状态配置上移 / 下移
- [x] Task 7: 修复拖拽（`dragDropEnabled` + 验收）
- [x] Task 8: 收尾验证

---

### Task 1: 模型字段

**Files:**
- Modify: `src-tauri/src/models/progress_log.rs`
- Modify: `src-tauri/src/models/work_order.rs`
- Modify: `src-tauri/src/services/work_order_service.rs`（仅 `row_to_work_order` 编译通过）

**Interfaces:**
- Produces:
  - `ProgressLogSummary { id, title, status, created_at }`（serde camelCase；specta Type；**不含** content / work_order_id）
  - `WorkOrder.progress_summaries: Vec<ProgressLogSummary>`，`#[serde(default)]`

- [ ] **Step 1: 在 `progress_log.rs` 增加摘要类型**

```rust
/// 列表卡片用的过程摘要（无正文 / 扩展字段 / work_order_id）。
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProgressLogSummary {
    pub id: Option<i64>,
    pub title: String,
    pub status: String,
    pub created_at: NaiveDateTime,
}
```

- [ ] **Step 2: 扩展 `WorkOrder`**

在 `work_order.rs` 的 `WorkOrder` 中增加：

```rust
    #[serde(default)]
    pub progress_summaries: Vec<crate::models::progress_log::ProgressLogSummary>,
```

（若该文件已 `use` progress 类型，可写成短名 `ProgressLogSummary`。）

- [ ] **Step 3: 更新 `row_to_work_order`**

在 `work_order_service.rs` 的构造中增加：

```rust
        progress_summaries: vec![],
```

凡手写 `WorkOrder { ... }` 的测试/代码同样补上 `progress_summaries: vec![]`（或依赖 Default——本结构体无 Default 则显式补字段）。

- [ ] **Step 4: 编译检查**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`  
Expected: 通过（或仅提示未使用类型，无 error）。

---

### Task 2: 主清单批量挂载摘要

**Files:**
- Modify: `src-tauri/src/services/work_order_service.rs`

**Interfaces:**
- Consumes: `ProgressLogSummary`；`progress_log` 表
- Produces:
  - `fn attach_progress_summaries(conn, &mut [WorkOrder]) -> Result<(), ServiceError>`
  - `find_by_filters` 返回的每条工单（主清单）带齐该工单下全部摘要，按 `created_at DESC`
  - `get_required` / `list_trashed` **不要求**填充（保持 `[]`）

- [ ] **Step 1: Write the failing test**

在 `work_order_service.rs` 的 `mod tests` 末尾增加：

```rust
    #[test]
    fn find_by_statuses_includes_progress_summaries_ordered_desc() {
        let (conn, dir) = temp_db();
        let cfg = config();
        let tags = tag_config();
        let a = create(&conn, input("A"), &cfg, &tags).unwrap();
        let b = create(&conn, input("B"), &cfg, &tags).unwrap();
        let a_id = a.id.unwrap();
        let b_id = b.id.unwrap();

        crate::services::progress_log_service::add_log(
            &conn,
            a_id,
            &crate::models::progress_log::ProgressLogInput {
                title: "old".into(),
                content: Some("secret".into()),
                status: "NOT_STARTED".into(),
                extra_fields: None,
            },
            &cfg,
        )
        .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        crate::services::progress_log_service::add_log(
            &conn,
            a_id,
            &crate::models::progress_log::ProgressLogInput {
                title: "new".into(),
                content: None,
                status: "IN_PROGRESS".into(),
                extra_fields: None,
            },
            &cfg,
        )
        .unwrap();
        crate::services::progress_log_service::add_log(
            &conn,
            b_id,
            &crate::models::progress_log::ProgressLogInput {
                title: "b-only".into(),
                content: None,
                status: "COMPLETED".into(),
                extra_fields: None,
            },
            &cfg,
        )
        .unwrap();

        let list = find_by_statuses(&conn, &[], None).unwrap();
        let wo_a = list.iter().find(|w| w.id == Some(a_id)).unwrap();
        let wo_b = list.iter().find(|w| w.id == Some(b_id)).unwrap();
        assert_eq!(wo_a.progress_summaries.len(), 2);
        assert_eq!(wo_a.progress_summaries[0].title, "new");
        assert_eq!(wo_a.progress_summaries[1].title, "old");
        assert_eq!(wo_b.progress_summaries.len(), 1);
        assert_eq!(wo_b.progress_summaries[0].title, "b-only");

        let single = get_required(&conn, a_id).unwrap();
        assert!(single.progress_summaries.is_empty());

        drop(conn);
        let _ = std::fs::remove_dir_all(dir);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml find_by_statuses_includes_progress_summaries_ordered_desc -- --nocapture`  
Expected: FAIL（`progress_summaries` 仍为空）。

- [ ] **Step 3: 实现批量挂载**

在 `work_order_service.rs` 增加（可放在 `enrich_with_tags` 附近）：

```rust
fn attach_progress_summaries(
    conn: &Connection,
    orders: &mut [WorkOrder],
) -> Result<(), ServiceError> {
    let ids: Vec<i64> = orders.iter().filter_map(|w| w.id).collect();
    if ids.is_empty() {
        return Ok(());
    }
    let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "SELECT id, work_order_id, title, status, created_at FROM progress_log \
         WHERE work_order_id IN ({}) ORDER BY created_at DESC",
        placeholders.join(", ")
    );
    let mut stmt = conn.prepare(&sql)?;
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = ids
        .iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .collect();
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        let work_order_id: i64 = row.get("work_order_id")?;
        Ok((
            work_order_id,
            crate::models::progress_log::ProgressLogSummary {
                id: Some(row.get("id")?),
                title: row.get("title")?,
                status: row.get("status")?,
                created_at: read_datetime_column(row, "created_at")?,
            },
        ))
    })?;

    let mut by_wo: HashMap<i64, Vec<crate::models::progress_log::ProgressLogSummary>> =
        HashMap::new();
    for row in rows {
        let (work_order_id, s) = row?;
        by_wo.entry(work_order_id).or_default().push(s);
    }
    for wo in orders.iter_mut() {
        if let Some(id) = wo.id {
            wo.progress_summaries = by_wo.remove(&id).unwrap_or_default();
        }
    }
    Ok(())
}
```

在 `find_by_filters` 末尾，由：

```rust
    for row in rows {
        result.push(enrich_with_tags(conn, row?)?);
    }
    Ok(result)
```

改为：

```rust
    for row in rows {
        result.push(enrich_with_tags(conn, row?)?);
    }
    attach_progress_summaries(conn, &mut result)?;
    Ok(result)
```

注意：`params_refs` 对 `i64` 的写法若编译报错，改为 `rusqlite::params_from_iter(ids.iter())` 或按项目现有 IN 查询写法调整。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml find_by_statuses_includes_progress_summaries_ordered_desc -- --nocapture`  
Expected: PASS。

- [ ] **Step 5: 回归相关测试**

Run: `cargo test --manifest-path src-tauri/Cargo.toml work_order_service -- --nocapture`  
Expected: PASS。

---

### Task 3: 导出 bindings

**Files:**
- Modify: `src/bindings.ts`（由工具生成，勿手改大段）
- Optional: `src/types.ts` re-export

**Interfaces:**
- Produces: TS `ProgressLogSummary`；`WorkOrder.progressSummaries?: ProgressLogSummary[]`（或必填数组，与 serde default 对齐）

- [ ] **Step 1: 重新导出**

Run: `npm run bindings`  
Expected: `src/bindings.ts` 出现 `ProgressLogSummary` 与 `progressSummaries`。

- [ ] **Step 2: 如需 re-export**

在 `src/types.ts` 的 `export type { ... }` 中加入 `ProgressLogSummary`。

- [ ] **Step 3: 类型检查**

Run: `npm run build`  
Expected: `vue-tsc` 通过（若别处解构 `WorkOrder` 未破坏）。

---

### Task 4: `groupCardProgress` 纯函数

**Files:**
- Create: `src/utils/cardProgressGroups.ts`

**Interfaces:**
- Consumes: `ProgressLogSummary`；有序状态 id 列表（`order` 已排好）
- Produces:
  - `export type CardProgressGroup = { statusId: string; items: ProgressLogSummary[]; truncated: boolean }`
  - `export function groupCardProgress(summaries: ProgressLogSummary[], orderedStatusIds: string[], maxVisible?: number): CardProgressGroup[]`
  - 默认 `maxVisible = 6`

- [ ] **Step 1: 实现函数（完整逻辑）**

```ts
import type { ProgressLogSummary } from "../bindings";

export type CardProgressGroup = {
  statusId: string;
  items: ProgressLogSummary[];
  truncated: boolean;
};

const DEFAULT_MAX = 6;

export function groupCardProgress(
  summaries: ProgressLogSummary[],
  orderedStatusIds: string[],
  maxVisible: number = DEFAULT_MAX,
): CardProgressGroup[] {
  if (!summaries.length) return [];

  const byStatus = new Map<string, ProgressLogSummary[]>();
  for (const s of summaries) {
    const list = byStatus.get(s.status) ?? [];
    list.push(s);
    byStatus.set(s.status, list);
  }
  for (const list of byStatus.values()) {
    list.sort((a, b) => (a.createdAt < b.createdAt ? 1 : a.createdAt > b.createdAt ? -1 : 0));
  }

  const known = new Set(orderedStatusIds);
  const statusOrder = [
    ...orderedStatusIds.filter((id) => byStatus.has(id)),
    ...[...byStatus.keys()].filter((id) => !known.has(id)).sort(),
  ];

  let remaining = maxVisible;
  const groups: CardProgressGroup[] = [];

  for (const statusId of statusOrder) {
    const all = byStatus.get(statusId) ?? [];
    if (all.length === 0) continue;
    const take = Math.min(remaining, all.length);
    const items = all.slice(0, take);
    remaining -= take;
    groups.push({
      statusId,
      items,
      truncated: items.length < all.length,
    });
  }

  return groups;
}
```

说明：名额用尽后，`take === 0` 且 `all.length > 0` 的组仍会进入循环并 push（`items: []`, `truncated: true`），满足 spec「0 可见仍显示组 + 还有更多」。

- [ ] **Step 2: 手工核对用例（在注释或本地临时断言）**

| 输入 | 期望 |
|------|------|
| `summaries=[]` | `[]` |
| 单状态 8 条，`max=6` | 1 组，`items.length===6`，`truncated===true` |
| 状态序 [A,B]，A 有 5、B 有 3 | A 5 条 + B 1 条；B `truncated===true` |
| 状态序 [A,B]，A 有 6、B 有 2 | A 6 条；B `items=[]` 且 `truncated===true` |
| 未知状态夹杂 | 已知状态先按 `orderedStatusIds`，未知在后 |

- [ ] **Step 3: `npm run build`**

Expected: PASS。

---

### Task 5: 卡片过程区 UI

**Files:**
- Modify: `src/views/WorkOrderList.vue`
- Modify: `src/styles/main.css`

**Interfaces:**
- Consumes: `groupCardProgress`；`statusOptions` / `statusLabel` / `statusColor`；`tagStyleForStatus`
- Produces: 卡片模板中的过程区

- [ ] **Step 1: script 引入与辅助**

```ts
import { groupCardProgress } from "../utils/cardProgressGroups";
import { tagStyleForStatus } from "../utils/statusColors";
import type { ProgressLogSummary } from "../types"; // 或 bindings

function orderedStatusIds(): string[] {
  return statusOptions.value.map((o) => o.value);
}

function cardProgressGroups(item: { progressSummaries?: ProgressLogSummary[] }) {
  return groupCardProgress(item.progressSummaries ?? [], orderedStatusIds());
}
```

（`statusOptions` 已由 `useStatusConfig` 提供；若 option 字段名是 `value`/`label`，与现有筛选一致。）

- [ ] **Step 2: 卡片模板 — 在标签与 `card-meta` 之间插入**

```vue
                <div class="card-progress">
                  <template v-if="!(item.progressSummaries?.length)">
                    <div class="card-progress-empty">暂无过程</div>
                  </template>
                  <template v-else>
                    <div
                      v-for="group in cardProgressGroups(item)"
                      :key="group.statusId"
                      class="card-progress-group"
                    >
                      <div class="card-progress-group-title">
                        {{ statusLabel(group.statusId) }}
                      </div>
                      <div
                        v-for="(log, idx) in group.items"
                        :key="log.id ?? `${group.statusId}-${idx}`"
                        class="card-progress-row"
                      >
                        <span class="card-progress-row-title">{{ log.title }}</span>
                        <n-tag
                          size="small"
                          :bordered="false"
                          :style="tagStyleForStatus(statusColor(log.status))"
                        >
                          {{ statusLabel(log.status) }}
                        </n-tag>
                      </div>
                      <div v-if="group.truncated" class="card-progress-more">还有更多</div>
                    </div>
                  </template>
                </div>
```

- [ ] **Step 3: CSS（`main.css`）**

```css
.work-order-card .card-progress {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 12px;
}

.work-order-card .card-progress-empty {
  color: #999;
}

.work-order-card .card-progress-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.work-order-card .card-progress-group-title {
  font-weight: 600;
  color: #555;
}

.work-order-card .card-progress-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-width: 0;
}

.work-order-card .card-progress-row-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #333;
}

.work-order-card .card-progress-more {
  color: #999;
}
```

- [ ] **Step 4: `npm run build` + 手工**

- 卡片：无过程 →「暂无过程」
- 多状态过程 → 分组；超过 6 →「还有更多」
- 列表模式无过程区
- 点击过程区仍打开详情

---

### Task 6: 状态配置上移 / 下移

**Files:**
- Modify: `src/components/StatusConfigPanel.vue`

**Interfaces:**
- Consumes: 现有 `statusList` computed setter（已按 index 写 `order`）
- Produces: `moveStatus(id, delta: -1 | 1)`

- [ ] **Step 1: 增加移动函数**

```ts
function moveStatus(id: string, delta: -1 | 1) {
  const list = [...statusList.value];
  const index = list.findIndex((s) => s.id === id);
  if (index < 0) return;
  const target = index + delta;
  if (target < 0 || target >= list.length) return;
  const next = [...list];
  const [item] = next.splice(index, 1);
  next.splice(target, 0, item);
  statusList.value = next;
}
```

- [ ] **Step 2: 行内按钮（删除旁）**

```vue
              <n-space :size="4" @click.stop>
                <n-button
                  text
                  size="tiny"
                  :disabled="statusList[0]?.id === item.id"
                  @click="moveStatus(item.id, -1)"
                >
                  上移
                </n-button>
                <n-button
                  text
                  size="tiny"
                  :disabled="statusList[statusList.length - 1]?.id === item.id"
                  @click="moveStatus(item.id, 1)"
                >
                  下移
                </n-button>
                <n-button
                  text
                  type="error"
                  size="tiny"
                  @click.stop="removeStatus(item.id)"
                >
                  删除
                </n-button>
              </n-space>
```

（替换原单独删除按钮；保持 `@click.stop`。）

- [ ] **Step 3: `npm run build` + 手工**

- 上移/下移后 draft `order` 连续；保存后筛选与卡片分组顺序变化
- 首项上移、末项下移禁用

---

### Task 7: 修复拖拽

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Possibly: `src/views/WorkOrderList.vue`、`src/components/StatusConfigPanel.vue`（若关闭 DnD 后仍失效）

**Interfaces:**
- Produces: Tauri 窗口 `dragDropEnabled: false`；两处拖拽可用

- [ ] **Step 1: 关闭 Tauri 文件拖放拦截**

在 `tauri.conf.json` 的 `app.windows[0]` 增加：

```json
        "dragDropEnabled": false
```

完整窗口对象示例：

```json
      {
        "title": "workOrder",
        "width": 1024,
        "height": 768,
        "resizable": true,
        "dragDropEnabled": false
      }
```

- [ ] **Step 2: 用 `tauri:dev` 验证**

Run: `npm run tauri:dev`  
手工：

1. 代办**列表**模式：拖 `.drag-handle` 区域改序，刷新后顺序仍在  
2. 代办**卡片**模式：拖 `⋮⋮` 改序，持久化成功  
3. 设置 → 代办状态：拖状态行改序，保存后 `order` 正确  

- [ ] **Step 3: 若仍失效 — 次要排查（按需）**

- 列表：整行 `list-row-main` 同时是 `drag-handle` 与 `@click` 打开详情；可改为独立窄把手（如卡片 `⋮⋮`），主行只负责点击打开  
- 状态：`handle=".status-drag-row"` 与整行 click 选中冲突时，改为独立 `.status-drag-handle`  
- 确认未在父级 `preventDefault` 掉 `dragstart`

修到 Step 2 三条均通过为止。

---

### Task 8: 收尾验证

**Files:** 无新文件

- [ ] **Step 1: 后端测试**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -- --nocapture`  
Expected: PASS。

- [ ] **Step 2: 前端构建**

Run: `npm run build`  
Expected: PASS。

- [ ] **Step 3: Spec 对照清单**

| Spec 项 | 验证 |
|---------|------|
| 卡片仅标题+状态 | ✓ |
| 按状态 order 分组 | ✓ |
| 最多 6 条 | ✓ |
| 「还有更多」无数字 | ✓ |
| 0 可见组仍显示标题+标记 | ✓ |
| 无过程「暂无过程」 | ✓ |
| 列表无过程区 | ✓ |
| 主清单含摘要 | cargo 测 ✓ |
| 状态上移/下移 | ✓ |
| 列表+状态拖拽 | Tauri 手工 ✓ |
| 回收站不依赖摘要 | ✓ |

---

## Spec coverage（自检）

| Spec | Task |
|------|------|
| §3.1 ProgressLogSummary / WorkOrder 字段 | 1 |
| §3.2 批量挂载、主清单填充 | 2 |
| bindings | 3 |
| §3.3 前端截断算法 | 4 |
| §4.1–4.2 卡片 UI | 5 |
| §4.3 上移/下移 | 6 |
| §4.4 拖拽修复 | 7 |
| §5–6 边界与测试 | 2、8 |

无 TBD；类型名 `progressSummaries` / `ProgressLogSummary` / `groupCardProgress` 前后一致。
