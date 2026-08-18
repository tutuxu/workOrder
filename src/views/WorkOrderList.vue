<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useMessage, type DropdownOption } from "naive-ui";
import { VueDraggable } from "vue-draggable-plus";
import * as workOrderApi from "../api/workOrders";
import { formatLocalDateTime, formatServerDateTime } from "../utils/datetime";
import { formatServiceError } from "../utils/serviceError";
import { useWorkOrders } from "../composables/useWorkOrders";
import { useStatusConfig } from "../composables/useStatusConfig";
import { useTagConfig } from "../composables/useTagConfig";
import { registerShortcut, unregisterShortcut } from "../composables/useShortcuts";
import { useListViewMode, type ListViewMode } from "../composables/useListViewMode";
import { useTrashConfirm } from "../composables/useTrashConfirm";
import { rowStyleForStatus, tagStyleForTag } from "../utils/statusColors";

const emit = defineEmits<{
  openDetail: [order: import("../types").WorkOrder | null];
  openSettings: [];
  openRecycleBin: [];
}>();

const message = useMessage();
const { confirmMoveToTrash, confirmPermanentDelete } = useTrashConfirm();
const { statusOptions, statusLabel, statusColor, load: loadStatusConfig } = useStatusConfig();
const { tagOptions, tagLabel, tagColor, load: loadTagConfig } = useTagConfig();
const { viewMode, setViewMode } = useListViewMode();
const cardMultiSelect = ref(false);

const {
  items,
  loading,
  selectedStatuses,
  selectedTags,
  tagMatchMode,
  searchQuery,
  refresh,
  isOverdue,
  reorder,
} = useWorkOrders();

const localItems = ref<import("../types").WorkOrder[]>([]);
const selectedIds = ref<Set<number>>(new Set());
let searchTimer: ReturnType<typeof setTimeout> | undefined;

const ctxMenuShow = ref(false);
const ctxMenuX = ref(0);
const ctxMenuY = ref(0);
const ctxMenuOrder = ref<import("../types").WorkOrder | null>(null);

const selectableIds = computed(() =>
  localItems.value
    .map((item) => item.id)
    .filter((id): id is number => id != null),
);

const allSelected = computed(
  () =>
    selectableIds.value.length > 0 &&
    selectableIds.value.every((id) => selectedIds.value.has(id)),
);

const someSelected = computed(
  () => selectedIds.value.size > 0 && !allSelected.value,
);

const selectedCount = computed(() => selectedIds.value.size);

const emptyDescription = computed(() =>
  searchQuery.value.trim() ? "无匹配代办" : "暂无代办",
);

const ctxMenuOptions = computed<DropdownOption[]>(() => {
  const inMulti = cardMultiSelect.value;
  const count = selectedCount.value;
  const actionDisabled = inMulti && count === 0;
  return [
    { label: "打开", key: "open" },
    { label: "多选", key: "multi" },
    {
      label: "移入回收站",
      key: "trash",
      disabled: actionDisabled,
    },
    {
      label: "彻底删除",
      key: "permanent",
      disabled: actionDisabled,
    },
  ];
});

onMounted(async () => {
  try {
    await Promise.all([loadStatusConfig(), loadTagConfig()]);
    await refresh();
    localItems.value = [...items.value];
  } catch (error) {
    message.error(`加载失败：${error}`);
  }

  registerShortcut("list.new", {
    handler: () => openNew(),
    enabled: () => true,
  });
  registerShortcut("list.settings", {
    handler: () => emit("openSettings"),
    enabled: () => true,
  });
  registerShortcut("list.deleteSelected", {
    handler: () => {
      openTrashConfirm([...selectedIds.value]);
    },
    enabled: () => selectedCount.value > 0,
  });
});

onUnmounted(() => {
  unregisterShortcut("list.new");
  unregisterShortcut("list.settings");
  unregisterShortcut("list.deleteSelected");
});

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
  if (key === "trash") {
    const ids = contextActionIds(order);
    if (ids.length === 0) return;
    openTrashConfirm(ids, !cardMultiSelect.value);
    return;
  }
  if (key === "permanent") {
    const ids = contextActionIds(order);
    if (ids.length === 0) return;
    confirmPermanentDelete({
      count: ids.length,
      onConfirm: () => {
        void permanentlyDeleteIds(ids);
      },
    });
  }
}

function contextActionIds(order: import("../types").WorkOrder): number[] {
  if (cardMultiSelect.value) {
    return [...selectedIds.value];
  }
  return order.id != null ? [order.id] : [];
}

function openTrashConfirm(ids: number[], singular = false) {
  if (ids.length === 0) return;
  confirmMoveToTrash({
    count: ids.length,
    singular,
    onTrash: () => {
      void trashIds(ids);
    },
    onPermanent: () => {
      void permanentlyDeleteIds(ids);
    },
  });
}

async function trashIds(ids: number[]) {
  try {
    await workOrderApi.trashWorkOrders(ids);
    message.success(`已移入回收站 ${ids.length} 条`);
    clearSelection();
    await syncLocalItems();
  } catch (error) {
    message.error(`移入回收站失败：${formatServiceError(error)}`);
  }
}

async function permanentlyDeleteIds(ids: number[]) {
  try {
    await workOrderApi.permanentlyDeleteWorkOrders(ids);
    message.success(`已彻底删除 ${ids.length} 条`);
    clearSelection();
    await syncLocalItems();
  } catch (error) {
    message.error(`彻底删除失败：${formatServiceError(error)}`);
  }
}

function clearSelection() {
  selectedIds.value = new Set();
}

async function syncLocalItems() {
  await refresh();
  localItems.value = [...items.value];
  const visibleIds = new Set(selectableIds.value);
  selectedIds.value = new Set(
    [...selectedIds.value].filter((id) => visibleIds.has(id)),
  );
}

async function onFilterChange() {
  await syncLocalItems();
}

async function onTagFilterChange() {
  await syncLocalItems();
}

function onMatchAllChange(checked: boolean) {
  tagMatchMode.value = checked ? "all" : "any";
  void onTagFilterChange();
}

function onSearchInput(value: string) {
  searchQuery.value = value;
  if (searchTimer) {
    clearTimeout(searchTimer);
  }
  searchTimer = setTimeout(() => {
    syncLocalItems().catch((error) => {
      message.error(`搜索失败：${error}`);
    });
  }, 300);
}

async function onDragEnd() {
  const orderedIds = localItems.value
    .map((item) => item.id)
    .filter((id): id is number => id != null);
  await reorder(orderedIds);
  localItems.value = [...items.value];
}

function openNew() {
  emit("openDetail", null);
}

function openExisting(order: import("../types").WorkOrder) {
  emit("openDetail", order);
}

function rowStyle(order: import("../types").WorkOrder) {
  return rowStyleForStatus(statusColor(order.status), isOverdue(order));
}

function isSelected(id: number | null | undefined) {
  return id != null && selectedIds.value.has(id);
}

function toggleSelect(id: number | null | undefined, checked: boolean) {
  if (id == null) return;
  const next = new Set(selectedIds.value);
  if (checked) {
    next.add(id);
  } else {
    next.delete(id);
  }
  selectedIds.value = next;
}

function toggleSelectAll(checked: boolean) {
  selectedIds.value = checked ? new Set(selectableIds.value) : new Set();
}

async function deleteSelected() {
  openTrashConfirm([...selectedIds.value]);
}

async function reload() {
  await Promise.all([loadStatusConfig(true), loadTagConfig(true)]);
  await refresh();
  localItems.value = [...items.value];
}

defineExpose({ reload });
</script>

<template>
  <div class="work-order-list">
    <div class="toolbar">
      <div class="toolbar-row">
        <n-button type="primary" @click="openNew">新建</n-button>
        <n-button type="error" :disabled="selectedCount === 0" @click="deleteSelected">
          删除选中{{ selectedCount > 0 ? ` (${selectedCount})` : "" }}
        </n-button>
        <n-button
          v-if="viewMode === 'card' && cardMultiSelect"
          @click="exitCardMultiSelect"
        >
          取消多选
        </n-button>
        <n-button quaternary @click="emit('openRecycleBin')">回收站</n-button>
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
          class="search-input"
          :value="searchQuery"
          clearable
          placeholder="搜索标题、描述、状态字段"
          @update:value="onSearchInput"
        />
        <div class="status-filters">
          <span>状态筛选</span>
          <n-checkbox-group v-model:value="selectedStatuses" @update:value="onFilterChange">
            <n-space>
              <n-checkbox
                v-for="opt in statusOptions"
                :key="opt.value"
                :value="opt.value"
                :label="opt.label"
              />
            </n-space>
          </n-checkbox-group>
        </div>
      </div>
      <div class="toolbar-row tag-filters">
        <span>标签筛选</span>
        <n-checkbox
          :checked="tagMatchMode === 'all'"
          label="同时满足"
          @update:checked="onMatchAllChange"
        />
        <n-checkbox-group v-model:value="selectedTags" @update:value="onTagFilterChange">
          <n-space>
            <n-checkbox
              v-for="opt in tagOptions"
              :key="opt.value"
              :value="opt.value"
              :label="opt.label"
            />
          </n-space>
        </n-checkbox-group>
      </div>
    </div>

    <div class="list-container">
      <n-spin :show="loading">
        <div class="list-scroll">
          <template v-if="viewMode === 'list'">
            <div class="list-row list-header">
              <n-checkbox
                :checked="allSelected"
                :indeterminate="someSelected"
                @update:checked="toggleSelectAll"
              />
              <span>标题</span>
              <span>标签</span>
              <span>状态</span>
              <span>计划完成时间</span>
              <span>最后更新</span>
            </div>

            <VueDraggable
              v-model="localItems"
              :animation="150"
              handle=".drag-handle"
              @end="onDragEnd"
            >
              <div
                v-for="item in localItems"
                :key="item.id ?? item.updatedAt"
                class="list-row"
                :class="{
                  'overdue-row': isOverdue(item),
                  'selected-row': isSelected(item.id),
                }"
                :style="rowStyle(item)"
              >
                <n-checkbox
                  :checked="isSelected(item.id)"
                  @update:checked="(checked: boolean) => toggleSelect(item.id, checked)"
                  @click.stop
                />
                <div class="list-row-main drag-handle" @click="openExisting(item)">
                  <span class="title-text">{{ item.title }}</span>
                  <div class="tag-cell">
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
                  </div>
                  <span>{{ statusLabel(item.status) }}</span>
                  <span>{{ formatLocalDateTime(item.dueDate) }}</span>
                  <span>{{ formatServerDateTime(item.updatedAt) }}</span>
                </div>
              </div>
            </VueDraggable>

            <n-empty v-if="!loading && localItems.length === 0" :description="emptyDescription" />
          </template>
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
                  'has-checkbox': cardMultiSelect,
                }"
                :style="rowStyle(item)"
                @click="openExisting(item)"
                @contextmenu="openCardContextMenu($event, item)"
              >
                <n-checkbox
                  v-if="cardMultiSelect"
                  class="card-checkbox"
                  :checked="isSelected(item.id)"
                  @update:checked="(checked: boolean) => toggleSelect(item.id, checked)"
                  @click.stop
                />
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
        </div>
      </n-spin>
    </div>

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
  </div>
</template>
