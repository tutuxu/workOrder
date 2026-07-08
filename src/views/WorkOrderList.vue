<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useMessage } from "naive-ui";
import { VueDraggable } from "vue-draggable-plus";
import * as workOrderApi from "../api/workOrders";
import { formatLocalDateTime, formatServerDateTime } from "../utils/datetime";
import { useWorkOrders } from "../composables/useWorkOrders";
import { useStatusConfig } from "../composables/useStatusConfig";
import { rowStyleForStatus } from "../utils/statusColors";

const emit = defineEmits<{
  openDetail: [order: import("../types").WorkOrder | null];
  openSettings: [];
}>();

const message = useMessage();
const { statusOptions, statusLabel, statusColor, load: loadStatusConfig } = useStatusConfig();

const {
  items,
  loading,
  selectedStatuses,
  searchQuery,
  refresh,
  isOverdue,
  reorder,
} = useWorkOrders();

const localItems = ref<import("../types").WorkOrder[]>([]);
const selectedIds = ref<Set<number>>(new Set());
let searchTimer: ReturnType<typeof setTimeout> | undefined;

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

onMounted(async () => {
  try {
    await loadStatusConfig();
    await refresh();
    localItems.value = [...items.value];
  } catch (error) {
    message.error(`加载失败：${error}`);
  }
});

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
  const ids = [...selectedIds.value];
  if (ids.length === 0) return;
  try {
    await Promise.all(ids.map((id) => workOrderApi.deleteWorkOrder(id)));
    message.success(`已删除 ${ids.length} 条`);
    clearSelection();
    await syncLocalItems();
  } catch (error) {
    message.error(`删除失败：${error}`);
  }
}

async function reload() {
  await loadStatusConfig(true);
  await refresh();
  localItems.value = [...items.value];
}

defineExpose({ reload });
</script>

<template>
  <div class="work-order-list">
    <div class="toolbar">
      <n-button type="primary" @click="openNew">新建</n-button>
      <n-popconfirm @positive-click="deleteSelected">
        <template #trigger>
          <n-button type="error" :disabled="selectedCount === 0">
            删除选中{{ selectedCount > 0 ? ` (${selectedCount})` : "" }}
          </n-button>
        </template>
        确定删除选中的 {{ selectedCount }} 条代办事项吗？
      </n-popconfirm>
      <n-button quaternary @click="emit('openSettings')">设置</n-button>
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

    <n-spin :show="loading">
      <div class="list-row list-header">
        <n-checkbox
          :checked="allSelected"
          :indeterminate="someSelected"
          @update:checked="toggleSelectAll"
        />
        <span>标题</span>
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
            <span>{{ item.title }}</span>
            <span>{{ statusLabel(item.status) }}</span>
            <span>{{ formatLocalDateTime(item.dueDate) }}</span>
            <span>{{ formatServerDateTime(item.updatedAt) }}</span>
          </div>
        </div>
      </VueDraggable>

      <n-empty v-if="!loading && localItems.length === 0" :description="emptyDescription" />
    </n-spin>
  </div>
</template>
