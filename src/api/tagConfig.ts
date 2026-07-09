import { commands } from "../bindings";
import type { ExportTagConfigResult, TagConfig } from "../bindings";

export type { ExportTagConfigResult };

export function getTagConfig(): Promise<TagConfig> {
  return commands.getTagConfig();
}

export function saveTagConfig(config: TagConfig): Promise<void> {
  return commands.saveTagConfig(config).then(() => undefined);
}

export function pickTagConfigSavePath(): Promise<string | null> {
  return commands.pickTagConfigSavePath();
}

export function pickTagConfigFile(): Promise<string | null> {
  return commands.pickTagConfigFile();
}

export function exportTagConfig(savePath: string): Promise<ExportTagConfigResult> {
  return commands.exportTagConfig(savePath);
}

export function importTagConfig(filePath: string): Promise<TagConfig> {
  return commands.importTagConfig(filePath);
}

export function countWorkOrdersByTag(tagId: string): Promise<number> {
  return commands.countWorkOrdersByTag(tagId);
}
