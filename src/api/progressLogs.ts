import { commands } from "../bindings";
import type { ProgressLog, ProgressLogInput } from "../bindings";

export type { ProgressLog, ProgressLogInput };

export function listProgressLogs(workOrderId: number): Promise<ProgressLog[]> {
  return commands.listProgressLogs(workOrderId);
}

export function addProgressLog(
  workOrderId: number,
  input: ProgressLogInput,
): Promise<ProgressLog> {
  return commands.addProgressLog(workOrderId, input);
}

export function updateProgressLog(
  logId: number,
  workOrderId: number,
  input: ProgressLogInput,
): Promise<ProgressLog> {
  return commands.updateProgressLog(logId, workOrderId, input);
}

export function deleteProgressLog(
  logId: number,
  workOrderId: number,
): Promise<void> {
  return commands.deleteProgressLog(logId, workOrderId).then(() => undefined);
}
