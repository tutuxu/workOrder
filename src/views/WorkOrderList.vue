<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useMessage } from "naive-ui";
import { VueDraggable } from "vue-draggable-plus";
import { formatLocalDateTime, formatServerDateTime } from "../utils/datetime";
import { useWorkOrders } from "../composables/useWorkOrders";
import { STATUS_OPTIONS, statusLabel, type WorkOrder } from "../types";

const emit = defineEmits<{
  openDetail: [order: WorkOrder | null];
  openSettings: [];
}>();

const message = useMessage();

const {
  items,
  loading,
  selectedStatuses,
  includeCompleted,
  refresh,
  isOverdue,
  reorder,
} = useWorkOrders();

const localItems = ref<WorkOrder[]>([]);

onMounted(async () => {
  try {
    await refresh();
    localItems.value = [...items.value];
  } catch (error) {
    message.error(`加载失败：${error}`);
  }
});

async function onFilterChange() {
  await refresh();
  localItems.value = [...items.value];
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

function openExisting(order: WorkOrder) {
  emit("openDetail", order);
}

async function reload() {
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
      <div class="status-filters">
        <span>状态筛选</span>
        <n-checkbox-group v-model:value="selectedStatuses" @update:value="onFilterChange">
          <n-space>
            <n-checkbox
              v-for="opt in STATUS_OPTIONS"
              :key="opt.value"
              :value="opt.value"
              :label="opt.label"
            />
          </n-space>
        </n-checkbox-group>
      </div>
      <n-checkbox v-model:checked="includeCompleted" @update:checked="onFilterChange">
        显示已完成
      </n-checkbox>
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

      <n-empty v-if="!loading && localItems.length === 0" description="暂无代办" />
    </n-spin>
  </div>
</template>
