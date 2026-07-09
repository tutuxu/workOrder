import { commands } from "../bindings";
import type { TagMatchMode, WorkOrder, WorkOrderInput } from "../bindings";

export type { WorkOrder, WorkOrderInput };

export function listWorkOrders(
  statuses: string[],
  tags: string[],
  tagMatchMode: TagMatchMode,
  query = "",
): Promise<WorkOrder[]> {
  return commands.listWorkOrders(statuses, tags, tagMatchMode, query.trim());
}

export function getWorkOrder(id: number): Promise<WorkOrder> {
  return commands.getWorkOrder(id);
}

export function createWorkOrder(input: WorkOrderInput): Promise<WorkOrder> {
  return commands.createWorkOrder(input);
}

export function updateWorkOrder(
  id: number,
  input: WorkOrderInput,
): Promise<WorkOrder> {
  return commands.updateWorkOrder(id, input);
}

export function deleteWorkOrder(id: number): Promise<void> {
  return commands.deleteWorkOrder(id).then(() => undefined);
}

export function updatePriorities(orderedIds: number[]): Promise<void> {
  return commands.updatePriorities(orderedIds).then(() => undefined);
}

export function isWorkOrderOverdue(workOrder: WorkOrder): Promise<boolean> {
  return commands.isWorkOrderOverdue(workOrder);
}
