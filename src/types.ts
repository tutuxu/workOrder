import type { WorkOrderStatus } from "./bindings";

export type {
  ProgressLog,
  ProgressLogInput,
  WorkOrder,
  WorkOrderInput,
  WorkOrderStatus,
} from "./bindings";

export const STATUS_OPTIONS: { value: WorkOrderStatus; label: string }[] = [
  { value: "NOT_STARTED", label: "未处置" },
  { value: "IN_PROGRESS", label: "处置中" },
  { value: "WAITING_REPLY", label: "待回复" },
  { value: "COMPLETED", label: "已完成" },
];

export function statusLabel(status: WorkOrderStatus): string {
  return STATUS_OPTIONS.find((o) => o.value === status)?.label ?? status;
}
