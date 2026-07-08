export type ShortcutContext = "list" | "detail" | "detail.progressForm" | "settings";

export type ShortcutActionId =
  | "list.new"
  | "list.deleteSelected"
  | "list.settings"
  | "detail.save"
  | "detail.delete"
  | "detail.close"
  | "detail.addProgress"
  | "detail.saveProgress"
  | "detail.cancelProgress"
  | "detail.focusNext"
  | "detail.textIndent"
  | "settings.close";

export interface ShortcutActionDef {
  id: ShortcutActionId;
  label: string;
  context: ShortcutContext;
  contextLabel: string;
  defaultBinding?: string;
  /** CapsLock form assist actions share one physical key. */
  linkedIds?: ShortcutActionId[];
}

export const SHORTCUT_ACTIONS: ShortcutActionDef[] = [
  {
    id: "list.new",
    label: "新建",
    context: "list",
    contextLabel: "列表",
    defaultBinding: "Ctrl+N",
  },
  {
    id: "list.deleteSelected",
    label: "删除选中",
    context: "list",
    contextLabel: "列表",
    defaultBinding: "Delete",
  },
  {
    id: "list.settings",
    label: "设置",
    context: "list",
    contextLabel: "列表",
    defaultBinding: "Ctrl+,",
  },
  {
    id: "detail.save",
    label: "保存",
    context: "detail",
    contextLabel: "详情",
    defaultBinding: "Ctrl+S",
  },
  {
    id: "detail.delete",
    label: "删除",
    context: "detail",
    contextLabel: "详情",
  },
  {
    id: "detail.close",
    label: "关闭",
    context: "detail",
    contextLabel: "详情",
    defaultBinding: "Escape",
  },
  {
    id: "detail.addProgress",
    label: "添加过程",
    context: "detail",
    contextLabel: "详情",
  },
  {
    id: "detail.saveProgress",
    label: "保存过程",
    context: "detail.progressForm",
    contextLabel: "详情 · 过程表单",
  },
  {
    id: "detail.cancelProgress",
    label: "取消过程",
    context: "detail.progressForm",
    contextLabel: "详情 · 过程表单",
    defaultBinding: "Escape",
  },
  {
    id: "detail.focusNext",
    label: "聚焦下一字段",
    context: "detail",
    contextLabel: "详情",
    defaultBinding: "CapsLock",
    linkedIds: ["detail.textIndent"],
  },
  {
    id: "detail.textIndent",
    label: "文本缩进",
    context: "detail",
    contextLabel: "详情",
    defaultBinding: "CapsLock",
    linkedIds: ["detail.focusNext"],
  },
  {
    id: "settings.close",
    label: "关闭设置",
    context: "settings",
    contextLabel: "设置",
    defaultBinding: "Escape",
  },
];

export const SHORTCUT_CONTEXT_ORDER: ShortcutContext[] = [
  "list",
  "detail",
  "detail.progressForm",
  "settings",
];

export function actionsForContext(context: ShortcutContext): ShortcutActionDef[] {
  return SHORTCUT_ACTIONS.filter((action) => action.context === context);
}
