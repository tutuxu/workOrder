export type WorkOrderStatus =
  | "NOT_STARTED"
  | "IN_PROGRESS"
  | "WAITING_REPLY"
  | "COMPLETED";

export interface WorkOrder {
  id?: number;
  title: string;
  description?: string | null;
  status: WorkOrderStatus;
  priority: number;
  waitingFor?: string | null;
  waitingReason?: string | null;
  dueDate?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface WorkOrderInput {
  title: string;
  description?: string | null;
  status: WorkOrderStatus;
  waitingFor?: string | null;
  waitingReason?: string | null;
  dueDate?: string | null;
}

export interface ProgressLog {
  id?: number;
  workOrderId: number;
  content: string;
  createdAt: string;
}

export const STATUS_OPTIONS: { value: WorkOrderStatus; label: string }[] = [
  { value: "NOT_STARTED", label: "未处置" },
  { value: "IN_PROGRESS", label: "处置中" },
  { value: "WAITING_REPLY", label: "待回复" },
  { value: "COMPLETED", label: "已完成" },
];

export function statusLabel(status: WorkOrderStatus): string {
  return STATUS_OPTIONS.find((o) => o.value === status)?.label ?? status;
}
