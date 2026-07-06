import { invoke } from "@tauri-apps/api/core";
import type { ProgressLog } from "../types";

export function listProgressLogs(workOrderId: number): Promise<ProgressLog[]> {
  return invoke<ProgressLog[]>("list_progress_logs", { workOrderId });
}

export function addProgressLog(
  workOrderId: number,
  content: string,
): Promise<ProgressLog> {
  return invoke<ProgressLog>("add_progress_log", { workOrderId, content });
}

export function updateProgressLog(
  logId: number,
  workOrderId: number,
  content: string,
): Promise<ProgressLog> {
  return invoke<ProgressLog>("update_progress_log", {
    logId,
    workOrderId,
    content,
  });
}

export function deleteProgressLog(
  logId: number,
  workOrderId: number,
): Promise<void> {
  return invoke<void>("delete_progress_log", { logId, workOrderId });
}
