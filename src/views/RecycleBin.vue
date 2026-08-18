<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useMessage } from "naive-ui";
import * as workOrderApi from "../api/workOrders";
import { formatServerDateTime } from "../utils/datetime";
import { formatServiceError } from "../utils/serviceError";
import { useStatusConfig } from "../composables/useStatusConfig";
import { useTagConfig } from "../composables/useTagConfig";
import { useTrashConfirm } from "../composables/useTrashConfirm";
import { rowStyleForStatus, tagStyleForTag } from "../utils/statusColors";
import type { WorkOrder } from "../types";

const emit = defineEmits<{
  closed: [];
  openDetail: [order: WorkOrder];
}>();

const message = useMessage();
const { confirmPermanentDelete } = useTrashConfirm();
const { statusLabel, statusColor, load: loadStatusConfig } = useStatusConfig();
const { tagLabel, tagColor, load: loadTagConfig } = useTagConfig();

const show = ref(true);
const loading = ref(false);
const items = ref<WorkOrder[]>([]);
const selectedIds = ref<Set<number>>(new Set());

const selectableIds = computed(() =>
  items.value.map((item) => item.id).filter((id): id is number => id != null),
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

function rowStyle(order: WorkOrder) {
  return rowStyleForStatus(statusColor(order.status), false);
}

function removeIds(ids: number[]) {
  const removed = new Set(ids);
  items.value = items.value.filter((item) => item.id == null || !removed.has(item.id));
  selectedIds.value = new Set([...selectedIds.value].filter((id) => !removed.has(id)));
}

async function loadItems() {
  loading.value = true;
  try {
    items.value = await workOrderApi.listTrashedWorkOrders();
    const visible = new Set(selectableIds.value);
    selectedIds.value = new Set([...selectedIds.value].filter((id) => visible.has(id)));
  } catch (error) {
    message.error(`加载回收站失败：${formatServiceError(error)}`);
  } finally {
    loading.value = false;
  }
}

async function restoreSelected() {
  const ids = [...selectedIds.value];
  if (ids.length === 0) return;
  try {
    await workOrderApi.restoreWorkOrders(ids);
    message.success(`已还原 ${ids.length} 条`);
    removeIds(ids);
  } catch (error) {
    message.error(`还原失败：${formatServiceError(error)}`);
  }
}

function requestPermanentDelete() {
  const ids = [...selectedIds.value];
  if (ids.length === 0) return;
  confirmPermanentDelete({
    count: ids.length,
    onConfirm: () => {
      void permanentlyDeleteSelected(ids);
    },
  });
}

async function permanentlyDeleteSelected(ids: number[]) {
  try {
    await workOrderApi.permanentlyDeleteWorkOrders(ids);
    message.success(`已彻底删除 ${ids.length} 条`);
    removeIds(ids);
  } catch (error) {
    message.error(`彻底删除失败：${formatServiceError(error)}`);
  }
}

function openReadonly(order: WorkOrder) {
  emit("openDetail", order);
}

function close() {
  emit("closed");
}

onMounted(async () => {
  await Promise.all([loadStatusConfig(), loadTagConfig()]);
  await loadItems();
});
</script>

<template>
  <n-modal
    v-model:show="show"
    preset="card"
    title="回收站"
    class="recycle-bin-modal"
    style="width: 840px"
    content-scrollable
    @after-leave="close"
  >
    <div class="recycle-bin">
      <div class="toolbar">
        <n-button :disabled="selectedCount === 0" @click="restoreSelected">还原</n-button>
        <n-button
          type="error"
          :disabled="selectedCount === 0"
          @click="requestPermanentDelete"
        >
          彻底删除{{ selectedCount > 0 ? ` (${selectedCount})` : "" }}
        </n-button>
      </div>

      <n-spin :show="loading">
        <div class="list-scroll recycle-bin-list">
          <div class="list-row list-header">
            <n-checkbox
              :checked="allSelected"
              :indeterminate="someSelected"
              @update:checked="toggleSelectAll"
            />
            <span>标题</span>
            <span>标签</span>
            <span>状态</span>
            <span>删除时间</span>
          </div>

          <div
            v-for="item in items"
            :key="item.id ?? item.updatedAt"
            class="list-row"
            :class="{ 'selected-row': isSelected(item.id) }"
            :style="rowStyle(item)"
          >
            <n-checkbox
              :checked="isSelected(item.id)"
              @update:checked="(checked: boolean) => toggleSelect(item.id, checked)"
              @click.stop
            />
            <div class="list-row-main" @click="openReadonly(item)">
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
              <span>{{ formatServerDateTime(item.deletedAt) }}</span>
            </div>
          </div>

          <n-empty v-if="!loading && items.length === 0" description="回收站为空" />
        </div>
      </n-spin>
    </div>
  </n-modal>
</template>
