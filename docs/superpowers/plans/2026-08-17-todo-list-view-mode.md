# 代办列表视图切换（列表 / 卡片）Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在代办列表页增加列表/卡片视图切换，卡片为多列网格；卡片右键进入多选，「取消多选」退出；视图偏好经 `localStorage` 持久化。

**Architecture:** 在 `WorkOrderList.vue` 内用 `viewMode` 双模板渲染；`useListViewMode` 负责读写 `workOrder.listViewMode`；卡片多选用 `cardMultiSelect` + 现有 `selectedIds`；右键用 Naive UI `n-dropdown` manual 模式。不改后端 / settings API。

**Tech Stack:** Vue 3、Naive UI、vue-draggable-plus、TypeScript、Vite

**Spec:** [2026-08-17-todo-list-view-mode-design.md](../specs/2026-08-17-todo-list-view-mode-design.md)

## Global Constraints

- 不改 `bindings` / Rust settings / 备份逻辑
- 右键菜单仅卡片模式；退出多选仅工具栏「取消多选」
- localStorage 键名必须为 `workOrder.listViewMode`，合法值 `list` | `card`，非法则默认 `list`
- 卡片模式不提供「全选」
- 本仓库无前端单测框架：每个 Task 用 `npm run build`（含 `vue-tsc --noEmit`）+ 文内手工检查代替 TDD
- **不要执行 git commit / push**（用户明确要求不做 git 操作）；计划中无 Commit 步骤

---

## File Structure

| 文件 | 职责 |
|------|------|
| `src/composables/useListViewMode.ts` | `viewMode` 读写 localStorage；`setViewMode` |
| `src/views/WorkOrderList.vue` | 切换控件、列表/卡片模板、`cardMultiSelect`、右键菜单、取消多选 |
| `src/styles/main.css` | `.card-grid` / `.work-order-card` 等样式 |

---

## 任务清单

- [x] Task 1: `useListViewMode` composable
- [x] Task 2: 工具栏视图切换 + 列表/卡片分支骨架
- [x] Task 3: 卡片网格 UI + CSS + 拖拽
- [x] Task 4: 卡片多选状态、「取消多选」、勾选显示
- [x] Task 5: 卡片右键菜单（打开 / 多选 / 删除）
- [x] Task 6: 视图切换时选中衔接 + 收尾验证

---

### Task 1: `useListViewMode` composable

**Files:**
- Create: `src/composables/useListViewMode.ts`

**Interfaces:**
- Consumes: `localStorage`（浏览器 / Tauri WebView）
- Produces:
  - `export type ListViewMode = "list" | "card"`
  - `export const LIST_VIEW_MODE_KEY = "workOrder.listViewMode"`
  - `export function readListViewMode(): ListViewMode`
  - `export function writeListViewMode(mode: ListViewMode): void`
  - `export function useListViewMode(): { viewMode: Ref<ListViewMode>; setViewMode: (mode: ListViewMode) => void }`

- [ ] **Step 1: 创建 composable**

```ts
import { ref, type Ref } from "vue";

export type ListViewMode = "list" | "card";

export const LIST_VIEW_MODE_KEY = "workOrder.listViewMode";

export function readListViewMode(): ListViewMode {
  try {
    const raw = localStorage.getItem(LIST_VIEW_MODE_KEY);
    if (raw === "list" || raw === "card") return raw;
  } catch {
    // ignore quota / privacy mode
  }
  return "list";
}

export function writeListViewMode(mode: ListViewMode): void {
  try {
    localStorage.setItem(LIST_VIEW_MODE_KEY, mode);
  } catch {
    // ignore
  }
}

export function useListViewMode(): {
  viewMode: Ref<ListViewMode>;
  setViewMode: (mode: ListViewMode) => void;
} {
  const viewMode = ref<ListViewMode>(readListViewMode());

  function setViewMode(mode: ListViewMode) {
    viewMode.value = mode;
    writeListViewMode(mode);
  }

  return { viewMode, setViewMode };
}
```

- [ ] **Step 2: 类型检查**

Run: `npm run build`  
Expected: 通过（或仅有与本文件无关的既有错误）；新文件无 TS 报错。

- [ ] **Step 3: 手工快速核验（可选，DevTools Console）**

在应用 WebView / 浏览器控制台：

```js
localStorage.setItem("workOrder.listViewMode", "card");
localStorage.getItem("workOrder.listViewMode"); // "card"
localStorage.setItem("workOrder.listViewMode", "nope");
// 重新进入页面后 viewMode 应为 list（由后续 Task 接入后验证）
```

本 Task 完成标准：文件存在且导出签名与上表一致。

---

### Task 2: 工具栏视图切换 + 列表/卡片分支骨架

**Files:**
- Modify: `src/views/WorkOrderList.vue`

**Interfaces:**
- Consumes: `useListViewMode`（Task 1）
- Produces: 模板内 `viewMode`；`onViewModeUpdate`；列表模板包在 `v-if="viewMode === 'list'"`；卡片占位 `v-else`

- [ ] **Step 1: script 引入与切换处理**

在 `WorkOrderList.vue` script 顶部增加 import，并在现有 state 旁加入：

```ts
import { useListViewMode, type ListViewMode } from "../composables/useListViewMode";

const { viewMode, setViewMode } = useListViewMode();
const cardMultiSelect = ref(false);

function onViewModeUpdate(mode: ListViewMode) {
  const prev = viewMode.value;
  setViewMode(mode);
  if (mode === "list") {
    cardMultiSelect.value = false;
    return;
  }
  // list → card
  if (prev === "list") {
    cardMultiSelect.value = selectedIds.value.size > 0;
  }
}
```

（`cardMultiSelect` 在本 Task 先声明，勾选 UI 在 Task 4 接上。）

- [ ] **Step 2: 工具栏插入分段控件**

在「设置」按钮与搜索框之间插入：

```vue
        <n-button quaternary @click="emit('openSettings')">设置</n-button>
        <n-radio-group
          :value="viewMode"
          size="small"
          @update:value="onViewModeUpdate"
        >
          <n-radio-button value="list">列表</n-radio-button>
          <n-radio-button value="card">卡片</n-radio-button>
        </n-radio-group>
        <n-input
```

- [ ] **Step 3: 用 `v-if` / `v-else` 包住现有列表与卡片占位**

将 `.list-scroll` 内部改为：

```vue
        <div class="list-scroll">
          <template v-if="viewMode === 'list'">
            <!-- 原有 list-header + VueDraggable 列表行 + empty：原样保留 -->
          </template>
          <template v-else>
            <div class="card-grid-placeholder">
              <n-empty
                v-if="!loading && localItems.length === 0"
                :description="emptyDescription"
              />
              <n-text v-else depth="3">卡片视图（构建中）</n-text>
            </div>
          </template>
        </div>
```

注意：列表分支内的 `n-empty` 保持在列表 `VueDraggable` 之后，与现状一致；卡片分支空状态单独处理，避免两套 empty 同时出现。

- [ ] **Step 4: 验证**

Run: `npm run build`  
Expected: PASS。

手工：`npm run tauri:dev`（或项目惯用启动方式）→ 工具栏可见「列表」「卡片」→ 点「卡片」出现占位文案 → 刷新后仍为卡片（localStorage）。

---

### Task 3: 卡片网格 UI + CSS + 拖拽

**Files:**
- Modify: `src/views/WorkOrderList.vue`（替换卡片占位）
- Modify: `src/styles/main.css`

**Interfaces:**
- Consumes: 现有 `localItems`、`rowStyle`、`isOverdue`、`isSelected`、`openExisting`、`onDragEnd`、`statusLabel`、`tagLabel`/`tagColor`/`tagStyleForTag`、时间格式化函数
- Produces: 可拖拽的多列卡片网格（尚无右键 / 多选勾选）

- [ ] **Step 1: 追加 CSS**

在 `src/styles/main.css` 末尾追加：

```css
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
  padding-bottom: 8px;
}

.work-order-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px;
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 8px;
  min-height: 120px;
  transition: border-color 0.15s, box-shadow 0.15s;
  cursor: pointer;
}

.work-order-card:hover {
  border-color: #18a058;
}

.work-order-card.selected-row {
  border-color: #18a058;
  background: #f6ffed;
}

.work-order-card .card-title {
  font-weight: 600;
  word-break: break-word;
}

.work-order-card .card-meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
  color: #666;
}

.work-order-card .card-drag-handle {
  position: absolute;
  top: 8px;
  right: 8px;
  padding: 2px 6px;
  font-size: 12px;
  color: #999;
  cursor: grab;
  user-select: none;
}

.work-order-card .card-checkbox {
  position: absolute;
  top: 8px;
  left: 8px;
}

.work-order-card.has-checkbox {
  padding-top: 36px;
}
```

- [ ] **Step 2: 替换卡片占位为真实网格**

```vue
          <template v-else>
            <VueDraggable
              v-model="localItems"
              class="card-grid"
              :animation="150"
              handle=".drag-handle"
              @end="onDragEnd"
            >
              <div
                v-for="item in localItems"
                :key="item.id ?? item.updatedAt"
                class="work-order-card"
                :class="{
                  'overdue-row': isOverdue(item),
                  'selected-row': isSelected(item.id),
                }"
                :style="rowStyle(item)"
                @click="openExisting(item)"
              >
                <span class="card-drag-handle drag-handle" @click.stop>⋮⋮</span>
                <div class="card-title">{{ item.title }}</div>
                <n-space v-if="item.tags?.length" size="small" class="tag-badges">
                  <n-tag
                    v-for="tagId in item.tags"
                    :key="tagId"
                    size="small"
                    :bordered="false"
                    :style="tagStyleForTag(tagColor(tagId))"
                  >
                    {{ tagLabel(tagId) }}
                  </n-tag>
                </n-space>
                <div class="card-meta">
                  <span>状态：{{ statusLabel(item.status) }}</span>
                  <span>计划完成：{{ formatLocalDateTime(item.dueDate) }}</span>
                  <span>最后更新：{{ formatServerDateTime(item.updatedAt) }}</span>
                </div>
              </div>
            </VueDraggable>
            <n-empty
              v-if="!loading && localItems.length === 0"
              :description="emptyDescription"
            />
          </template>
```

- [ ] **Step 3: 验证**

Run: `npm run build`  
Expected: PASS。

手工：卡片多列排布；字段齐全；拖拽把手可排序；点卡片打开详情；空列表显示空状态。

---

### Task 4: 卡片多选状态、「取消多选」、勾选显示

**Files:**
- Modify: `src/views/WorkOrderList.vue`

**Interfaces:**
- Consumes: `cardMultiSelect`（Task 2）、`selectedIds` / `toggleSelect` / `clearSelection` / `selectedCount`
- Produces: `exitCardMultiSelect()`；工具栏「取消多选」；卡片上条件勾选

- [ ] **Step 1: 增加退出函数**

```ts
function exitCardMultiSelect() {
  cardMultiSelect.value = false;
  clearSelection();
}

function enterCardMultiSelect(id: number | null | undefined) {
  cardMultiSelect.value = true;
  if (id != null) {
    toggleSelect(id, true);
  }
}
```

- [ ] **Step 2: 工具栏「取消多选」**

放在「删除选中」旁（或其后），仅卡片多选时显示：

```vue
        <n-button
          v-if="viewMode === 'card' && cardMultiSelect"
          @click="exitCardMultiSelect"
        >
          取消多选
        </n-button>
```

- [ ] **Step 3: 卡片勾选（仅 `cardMultiSelect`）**

在 `.work-order-card` 上增加 class，并在拖拽把手前插入 checkbox：

```vue
              <div
                v-for="item in localItems"
                :key="item.id ?? item.updatedAt"
                class="work-order-card"
                :class="{
                  'overdue-row': isOverdue(item),
                  'selected-row': isSelected(item.id),
                  'has-checkbox': cardMultiSelect,
                }"
                :style="rowStyle(item)"
                @click="openExisting(item)"
              >
                <n-checkbox
                  v-if="cardMultiSelect"
                  class="card-checkbox"
                  :checked="isSelected(item.id)"
                  @update:checked="(checked: boolean) => toggleSelect(item.id, checked)"
                  @click.stop
                />
                <span class="card-drag-handle drag-handle" @click.stop>⋮⋮</span>
                <!-- 其余字段不变 -->
```

本 Task 暂不接右键；可在 DevTools 临时 `cardMultiSelect = true` 测勾选，或先留到 Task 5 用右键验证。若需本 Task 可测：临时在工具栏加调试按钮并在合并前删除——**不要留下调试按钮**；优先用 Task 5 一并验证。

- [ ] **Step 4: 验证**

Run: `npm run build`  
Expected: PASS。

---

### Task 5: 卡片右键菜单（打开 / 多选 / 删除）

**Files:**
- Modify: `src/views/WorkOrderList.vue`

**Interfaces:**
- Consumes: `enterCardMultiSelect`、`openExisting`、`deleteSelected`、`workOrderApi.deleteWorkOrder`、`dialog` / `message`
- Produces: 卡片 `@contextmenu` + `n-dropdown`；单条删除 `deleteOne(id)`

- [ ] **Step 1: 右键菜单状态与选项**

```ts
import type { DropdownOption } from "naive-ui";

const ctxMenuShow = ref(false);
const ctxMenuX = ref(0);
const ctxMenuY = ref(0);
const ctxMenuOrder = ref<import("../types").WorkOrder | null>(null);

const ctxMenuOptions = computed<DropdownOption[]>(() => {
  const inMulti = cardMultiSelect.value;
  const count = selectedCount.value;
  const deleteLabel = inMulti ? "删除选中" : "删除";
  const deleteDisabled = inMulti && count === 0;
  return [
    { label: "打开", key: "open" },
    { label: "多选", key: "multi" },
    {
      label: deleteLabel,
      key: "delete",
      disabled: deleteDisabled,
    },
  ];
});

function openCardContextMenu(e: MouseEvent, order: import("../types").WorkOrder) {
  e.preventDefault();
  ctxMenuOrder.value = order;
  ctxMenuX.value = e.clientX;
  ctxMenuY.value = e.clientY;
  ctxMenuShow.value = true;
}

function onCtxMenuSelect(key: string | number) {
  const order = ctxMenuOrder.value;
  ctxMenuShow.value = false;
  if (!order) return;
  if (key === "open") {
    openExisting(order);
    return;
  }
  if (key === "multi") {
    enterCardMultiSelect(order.id);
    return;
  }
  if (key === "delete") {
    if (cardMultiSelect.value) {
      if (selectedCount.value === 0) return;
      dialog.warning({
        title: "确认删除",
        content: `确定删除选中的 ${selectedCount.value} 条代办事项吗？`,
        positiveText: "删除",
        negativeText: "取消",
        onPositiveClick: () => {
          void deleteSelected();
        },
      });
      return;
    }
    if (order.id == null) return;
    dialog.warning({
      title: "确认删除",
      content: "确定删除该代办事项吗？",
      positiveText: "删除",
      negativeText: "取消",
      onPositiveClick: () => {
        void deleteOne(order.id!);
      },
    });
  }
}

async function deleteOne(id: number) {
  try {
    await workOrderApi.deleteWorkOrder(id);
    message.success("已删除");
    await syncLocalItems();
  } catch (error) {
    message.error(`删除失败：${error}`);
  }
}
```

- [ ] **Step 2: 卡片绑定右键 + 页面级 dropdown**

在 `.work-order-card` 上增加：

```vue
                @contextmenu="openCardContextMenu($event, item)"
```

在 `.work-order-list` 根节点内（toolbar 外或末尾）增加：

```vue
    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      :x="ctxMenuX"
      :y="ctxMenuY"
      :options="ctxMenuOptions"
      :show="ctxMenuShow"
      :on-clickoutside="() => (ctxMenuShow = false)"
      @select="onCtxMenuSelect"
    />
```

若 Naive UI 版本对 `:x/:y` + `trigger="manual"` 的写法要求不同，以当前 `naive-ui` 文档为准；目标行为是指针位置弹出菜单。

- [ ] **Step 3: 验证**

Run: `npm run build`  
Expected: PASS。

手工（卡片模式）：

1. 右键 →「多选」→ 出现勾选且当前项已选；工具栏出现「取消多选」
2. 「取消多选」→ 勾选消失、选中清空
3. 非多选右键「删除」→ 删当前一条
4. 多选若干后右键「删除选中」→ 批量删除
5. 多选但全部取消勾选 →「删除选中」禁用
6. 「打开」与单击均打开详情

---

### Task 6: 视图切换衔接收尾 + 全量验证

**Files:**
- Modify: `src/views/WorkOrderList.vue`（仅当 Task 2 的 `onViewModeUpdate` 不完整时修补）

**Interfaces:**
- Consumes: Task 2–5 全部行为
- Produces: 与 spec §4.5–4.6 一致的切换语义

- [ ] **Step 1: 确认 `onViewModeUpdate` 完整**

最终函数必须为：

```ts
function onViewModeUpdate(mode: ListViewMode) {
  const prev = viewMode.value;
  setViewMode(mode);
  if (mode === "list") {
    cardMultiSelect.value = false;
    // 保留 selectedIds，供列表勾选使用
    return;
  }
  if (prev === "list") {
    cardMultiSelect.value = selectedIds.value.size > 0;
  }
}
```

切换视图时**不要**调用 `exitCardMultiSelect()`（那会清选中）；仅清 `cardMultiSelect` 标志。

- [ ] **Step 2: 类型检查与构建**

Run: `npm run build`  
Expected: PASS（`vue-tsc --noEmit` + vite build）。

- [ ] **Step 3: Spec 手工验收清单**

| # | 步骤 | 期望 |
|---|------|------|
| 1 | 切到卡片，刷新应用 | 仍为卡片 |
| 2 | localStorage 写成非法值后刷新 | 回退列表 |
| 3 | 列表勾选若干 → 切卡片 | 进入多选且勾选保留 |
| 4 | 卡片多选 → 切列表 | 列表勾选保留；无「取消多选」按钮 |
| 5 | 卡片「取消多选」→ 切列表 | 列表无选中 |
| 6 | 两种视图拖拽排序后刷新列表数据 | 顺序保持（现有 reorder） |
| 7 | 筛选掉已选项 | 选中集合去掉不可见 id |
| 8 | 列表模式右键 | 无自定义多选菜单（浏览器默认即可） |
| 9 | 卡片非多选无勾选 | 无 checkbox |

完成本清单即视为功能完成。

---

## Self-Review（对照 spec）

| Spec 项 | 对应 Task |
|---------|-----------|
| 分段控件 + localStorage | Task 1–2 |
| 多列卡片字段 | Task 3 |
| 拖拽两种视图 | Task 3（卡片）+ 现有列表 |
| 右键仅卡片；打开/多选/删除 | Task 5 |
| 取消多选唯一退出 | Task 4–5 |
| 视图切换选中衔接 | Task 2、6 |
| 不改后端 / 无全选卡片 | Global Constraints + 未实现全选 |
| 空状态 | Task 2–3 |

无 TBD / 占位步骤。类型名全程统一：`ListViewMode`、`cardMultiSelect`、`enterCardMultiSelect`、`exitCardMultiSelect`。
