import type {
  StatusConfig,
  StatusDefinition,
  StatusField,
  TagConfig,
  TagDefinition,
} from "./bindings";

export type {
  Attachment,
  OwnerType,
  StatusConfig,
  StatusDefinition,
  StatusField,
  StatusFieldType,
  TagConfig,
  TagDefinition,
  TagMatchMode,
  WorkOrder,
  WorkOrderInput,
  ProgressLog,
  ProgressLogInput,
} from "./bindings";

export function statusLabelFromConfig(
  config: StatusConfig | null,
  statusId: string,
): string {
  if (!config) return statusId;
  const found = config.statuses.find((s) => s.id === statusId);
  return found?.label ?? `未知状态 (${statusId})`;
}

export function sortedStatuses(config: StatusConfig): StatusDefinition[] {
  return [...config.statuses].sort((a, b) => a.order - b.order);
}

export function fieldsForStatus(
  config: StatusConfig | null,
  statusId: string,
): StatusField[] {
  if (!config) return [];
  return config.statuses.find((s) => s.id === statusId)?.fields ?? [];
}

export function defaultStatusId(config: StatusConfig | null): string {
  if (!config || config.statuses.length === 0) return "NOT_STARTED";
  const sorted = sortedStatuses(config);
  return sorted[0]?.id ?? "NOT_STARTED";
}

export function sortedTags(config: TagConfig): TagDefinition[] {
  return [...config.tags].sort((a, b) => a.order - b.order);
}

export function tagLabelFromConfig(
  config: TagConfig | null,
  tagId: string,
): string {
  if (!config) return tagId;
  return config.tags.find((t) => t.id === tagId)?.label ?? tagId;
}
