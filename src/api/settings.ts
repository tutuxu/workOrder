import { commands } from "../bindings";
import type { ChangeDataDirResult, SettingsInfo } from "../bindings";

export type { ChangeDataDirResult, SettingsInfo };

export function getSettings(): Promise<SettingsInfo> {
  return commands.getSettings();
}

export function pickDataDir(): Promise<string | null> {
  return commands.pickDataDir();
}

export function changeDataDir(newPath: string): Promise<ChangeDataDirResult> {
  return commands.changeDataDir(newPath);
}

export function restartApp(): Promise<null> {
  return commands.restartApp();
}
