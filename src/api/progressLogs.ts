import { commands } from "../bindings";
import type { ProgressLog } from "../bindings";

export type { ProgressLog };

export function listProgressLogs(workOrderId: number): Promise<ProgressLog[]> {
  return commands.listProgressLogs(workOrderId);
}

export function addProgressLog(
  workOrderId: number,
  content: string,
): Promise<ProgressLog> {
  return commands.addProgressLog(workOrderId, content);
}

export function updateProgressLog(
  logId: number,
  workOrderId: number,
  content: string,
): Promise<ProgressLog> {
  return commands.updateProgressLog(logId, workOrderId, content);
}

export function deleteProgressLog(
  logId: number,
  workOrderId: number,
): Promise<void> {
  return commands.deleteProgressLog(logId, workOrderId).then(() => undefined);
}
