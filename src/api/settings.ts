import { commands } from "../bindings";
import type {
  ChangeDataDirResult,
  ExportBackupResult,
  ImportBackupResult,
  SettingsInfo,
  ShortcutBindingsPayload,
} from "../bindings";

export type { ChangeDataDirResult, ExportBackupResult, ImportBackupResult, SettingsInfo, ShortcutBindingsPayload };

export function getSettings(): Promise<SettingsInfo> {
  return commands.getSettings();
}

export function pickDataDir(): Promise<string | null> {
  return commands.pickDataDir();
}

export function changeDataDir(newPath: string): Promise<ChangeDataDirResult> {
  return commands.changeDataDir(newPath);
}

export function pickBackupSavePath(): Promise<string | null> {
  return commands.pickBackupSavePath();
}

export function pickBackupFile(): Promise<string | null> {
  return commands.pickBackupFile();
}

export function exportBackup(savePath: string): Promise<ExportBackupResult> {
  return commands.exportBackup(savePath);
}

export function importBackup(zipPath: string): Promise<ImportBackupResult> {
  return commands.importBackup(zipPath);
}

export function restartApp(): Promise<null> {
  return commands.restartApp();
}

export function getShortcutBindings(): Promise<ShortcutBindingsPayload> {
  return commands.getShortcutBindings();
}

export function saveShortcutBindings(
  bindings: Record<string, string>,
): Promise<null> {
  return commands.saveShortcutBindings(bindings);
}
