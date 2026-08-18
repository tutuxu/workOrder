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
