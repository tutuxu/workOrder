import { invoke } from "@tauri-apps/api/core";
import type { WorkOrder, WorkOrderInput } from "../types";

export function listWorkOrders(
  statuses: string[],
  includeCompleted: boolean,
): Promise<WorkOrder[]> {
  return invoke<WorkOrder[]>("list_work_orders", { statuses, includeCompleted });
}

export function getWorkOrder(id: number): Promise<WorkOrder> {
  return invoke<WorkOrder>("get_work_order", { id });
}

export function createWorkOrder(input: WorkOrderInput): Promise<WorkOrder> {
  return invoke<WorkOrder>("create_work_order", { input });
}

export function updateWorkOrder(
  id: number,
  input: WorkOrderInput,
): Promise<WorkOrder> {
  return invoke<WorkOrder>("update_work_order", { id, input });
}

export function deleteWorkOrder(id: number): Promise<void> {
  return invoke<void>("delete_work_order", { id });
}

export function updatePriorities(orderedIds: number[]): Promise<void> {
  return invoke<void>("update_priorities", { orderedIds });
}

export function isWorkOrderOverdue(workOrder: WorkOrder): Promise<boolean> {
  return invoke<boolean>("is_work_order_overdue", { workOrder });
}
