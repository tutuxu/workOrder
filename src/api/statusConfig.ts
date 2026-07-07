import { commands } from "../bindings";
import type { StatusConfig } from "../bindings";

export function getStatusConfig(): Promise<StatusConfig> {
  return commands.getStatusConfig();
}

export function saveStatusConfig(config: StatusConfig): Promise<void> {
  return commands.saveStatusConfig(config).then(() => undefined);
}
