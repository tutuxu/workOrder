<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useMessage } from "naive-ui";
import { VueDraggable } from "vue-draggable-plus";
import { formatLocalDateTime, formatServerDateTime } from "../utils/datetime";
import { useWorkOrders } from "../composables/useWorkOrders";
import { useStatusConfig } from "../composables/useStatusConfig";

const emit = defineEmits<{
  openDetail: [order: import("../types").WorkOrder | null];
  openSettings: [];
}>();

const message = useMessage();
const { statusOptions, statusLabel, load: loadStatusConfig } = useStatusConfig();

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
let searchTimer: ReturnType<typeof setTimeout> | undefined;

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

async function syncLocalItems() {
  await refresh();
  localItems.value = [...items.value];
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
        <span>标题</span>
        <span>状态</span>
        <span>计划完成时间</span>
        <span>最后更新</span>
      </div>

      <VueDraggable
        v-model="localItems"
        :animation="150"
        handle=".list-row"
        @end="onDragEnd"
      >
        <div
          v-for="item in localItems"
          :key="item.id ?? item.updatedAt"
          class="list-row"
          :class="{ 'overdue-row': isOverdue(item) }"
          @click="openExisting(item)"
        >
          <span>{{ item.title }}</span>
          <span>{{ statusLabel(item.status) }}</span>
          <span>{{ formatLocalDateTime(item.dueDate) }}</span>
          <span>{{ formatServerDateTime(item.updatedAt) }}</span>
        </div>
      </VueDraggable>

      <n-empty v-if="!loading && localItems.length === 0" :description="emptyDescription" />
    </n-spin>
  </div>
</template>
