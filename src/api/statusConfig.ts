import { commands } from "../bindings";
import type { ExportStatusConfigResult, StatusConfig } from "../bindings";

export type { ExportStatusConfigResult };

export function getStatusConfig(): Promise<StatusConfig> {
  return commands.getStatusConfig();
}

export function saveStatusConfig(config: StatusConfig): Promise<void> {
  return commands.saveStatusConfig(config).then(() => undefined);
}

export function pickStatusConfigSavePath(): Promise<string | null> {
  return commands.pickStatusConfigSavePath();
}

export function pickStatusConfigFile(): Promise<string | null> {
  return commands.pickStatusConfigFile();
}

export function exportStatusConfig(savePath: string): Promise<ExportStatusConfigResult> {
  return commands.exportStatusConfig(savePath);
}

export function importStatusConfig(filePath: string): Promise<StatusConfig> {
  return commands.importStatusConfig(filePath);
}
