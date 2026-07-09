import { ref } from "vue";
import dayjs from "dayjs";
import type { TagMatchMode, WorkOrder } from "../types";
import * as api from "../api/workOrders";

export function useWorkOrders() {
  const items = ref<WorkOrder[]>([]);
  const loading = ref(false);
  const selectedStatuses = ref<string[]>([]);
  const selectedTags = ref<string[]>([]);
  const tagMatchMode = ref<TagMatchMode>("any");
  const searchQuery = ref("");

  async function refresh() {
    loading.value = true;
    try {
      items.value = await api.listWorkOrders(
        selectedStatuses.value,
        selectedTags.value,
        tagMatchMode.value,
        searchQuery.value,
      );
    } catch (error) {
      console.error("listWorkOrders failed", error);
      items.value = [];
      throw error;
    } finally {
      loading.value = false;
    }
  }

  function isOverdue(item: WorkOrder): boolean {
    if (!item.dueDate) {
      return false;
    }
    return dayjs(item.dueDate).isBefore(dayjs());
  }

  async function reorder(orderedIds: number[]) {
    await api.updatePriorities(orderedIds);
    await refresh();
  }

  return {
    items,
    loading,
    selectedStatuses,
    selectedTags,
    tagMatchMode,
    searchQuery,
    refresh,
    isOverdue,
    reorder,
  };
}
