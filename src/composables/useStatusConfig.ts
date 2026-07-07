import { computed, ref } from "vue";
import { commands } from "../bindings";
import type { StatusConfig } from "../bindings";
import {
  defaultStatusId,
  sortedStatuses,
  statusLabelFromConfig,
} from "../types";
import {
  statusColorFromConfig,
} from "../utils/statusColors";

const config = ref<StatusConfig | null>(null);
let loadPromise: Promise<StatusConfig> | null = null;

export function useStatusConfig() {
  const statusOptions = computed(() => {
    if (!config.value) return [];
    return sortedStatuses(config.value).map((s) => ({
      value: s.id,
      label: s.label,
    }));
  });

  async function load(force = false): Promise<StatusConfig> {
    if (!force && config.value) return config.value;
    if (!force && loadPromise) return loadPromise;
    loadPromise = commands.getStatusConfig().then((loaded) => {
      config.value = loaded;
      return loaded;
    });
    return loadPromise;
  }

  async function save(next: StatusConfig): Promise<void> {
    await commands.saveStatusConfig(next);
    config.value = next;
  }

  function statusLabel(statusId: string): string {
    return statusLabelFromConfig(config.value, statusId);
  }

  function statusColor(statusId: string): string {
    return statusColorFromConfig(config.value, statusId);
  }

  function fieldsForStatus(statusId: string) {
    if (!config.value) return [];
    return config.value.statuses.find((s) => s.id === statusId)?.fields ?? [];
  }

  function defaultStatus(): string {
    return defaultStatusId(config.value);
  }

  return {
    config,
    statusOptions,
    load,
    save,
    statusLabel,
    statusColor,
    fieldsForStatus,
    defaultStatus,
  };
}
