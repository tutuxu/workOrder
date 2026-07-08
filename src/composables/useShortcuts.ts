import { ref } from "vue";
import * as settingsApi from "../api/settings";
import { isTextInputElement } from "../utils/keyboard";
import {
  bindingsEqual,
  eventToBinding,
  isModifierOnlyEvent,
  normalizeBinding,
} from "../utils/shortcutFormat";
import {
  actionsForContext,
  SHORTCUT_ACTIONS,
  type ShortcutActionDef,
  type ShortcutActionId,
  type ShortcutContext,
} from "../types/shortcuts";

export type ShortcutHandler = (event: KeyboardEvent) => void | Promise<void>;

interface Registration {
  handler: ShortcutHandler;
  enabled: () => boolean;
}

const bindings = ref<Record<string, string>>({});
const handlers = new Map<ShortcutActionId, Registration>();
let loadPromise: Promise<void> | null = null;

export const shortcutUiState = {
  progressFormVisible: ref(false),
  isRecording: ref(false),
};

export type AppShortcutContext = {
  settingsOpen: boolean;
  detailOpen: boolean;
  progressFormVisible: boolean;
};

function resolveActiveContext(state: AppShortcutContext): ShortcutContext {
  if (state.settingsOpen) return "settings";
  if (state.detailOpen && state.progressFormVisible) return "detail.progressForm";
  if (state.detailOpen) return "detail";
  return "list";
}

function contextsToSearch(
  activeContext: ShortcutContext,
): ShortcutContext[] {
  if (activeContext === "detail.progressForm") {
    return ["detail.progressForm", "detail"];
  }
  return [activeContext];
}

function isCapsLockAction(actionId: ShortcutActionId): boolean {
  return actionId === "detail.focusNext" || actionId === "detail.textIndent";
}

function shouldRunInTextInput(actionId: ShortcutActionId): boolean {
  return (
    actionId === "detail.textIndent" ||
    actionId === "detail.save" ||
    actionId === "detail.saveProgress" ||
    actionId === "detail.cancelProgress"
  );
}

function pickCapsLockAction(inTextInput: boolean): ShortcutActionId {
  return inTextInput ? "detail.textIndent" : "detail.focusNext";
}

function canUseDetailActionDuringProgressForm(actionId: ShortcutActionId): boolean {
  return (
    isCapsLockAction(actionId) ||
    actionId === "detail.textIndent" ||
    actionId === "detail.focusNext"
  );
}

export function getEffectiveBinding(
  actionId: ShortcutActionId,
  customBindings?: Record<string, string>,
): string | undefined {
  const source = customBindings ?? bindings.value;
  if (Object.prototype.hasOwnProperty.call(source, actionId)) {
    const custom = source[actionId];
    return custom ? normalizeBinding(custom) : undefined;
  }
  const action = SHORTCUT_ACTIONS.find((item) => item.id === actionId);
  return action?.defaultBinding
    ? normalizeBinding(action.defaultBinding)
    : undefined;
}

export function getEffectiveBindings(
  customBindings?: Record<string, string>,
): Record<string, string> {
  const merged: Record<string, string> = {};
  for (const action of SHORTCUT_ACTIONS) {
    const binding = getEffectiveBinding(action.id, customBindings);
    if (binding) merged[action.id] = binding;
  }
  return merged;
}

export function getDefaultBindingsRecord(): Record<string, string> {
  const result: Record<string, string> = {};
  for (const action of SHORTCUT_ACTIONS) {
    if (action.defaultBinding) {
      result[action.id] = normalizeBinding(action.defaultBinding);
    }
  }
  return result;
}

export async function loadShortcutBindings(force = false): Promise<void> {
  if (loadPromise && !force) {
    await loadPromise;
    return;
  }
  loadPromise = (async () => {
    const payload = await settingsApi.getShortcutBindings();
    bindings.value = Object.fromEntries(
      Object.entries(payload.bindings).map(([key, value]) => [
        key,
        normalizeBinding(String(value)),
      ]),
    );
  })();
  await loadPromise;
}

export async function saveShortcutBindingsToServer(
  newBindings: Record<string, string>,
): Promise<void> {
  const normalized = Object.fromEntries(
    Object.entries(newBindings).map(([key, value]) => [
      key,
      value === "" ? "" : normalizeBinding(value),
    ]),
  );
  await settingsApi.saveShortcutBindings(normalized);
  bindings.value = normalized;
}

export function registerShortcut(
  actionId: ShortcutActionId,
  registration: Registration,
) {
  handlers.set(actionId, registration);
}

export function unregisterShortcut(actionId: ShortcutActionId) {
  handlers.delete(actionId);
}

function matchActionInContext(
  action: ShortcutActionDef,
  pressed: string,
  activeContext: ShortcutContext,
  searchingContext: ShortcutContext,
): ShortcutActionId | null {
  const binding = getEffectiveBinding(action.id);
  if (!binding || !bindingsEqual(binding, pressed)) return null;

  if (
    activeContext === "detail.progressForm" &&
    searchingContext === "detail" &&
    !canUseDetailActionDuringProgressForm(action.id)
  ) {
    return null;
  }

  const inTextInput = isTextInputElement(document.activeElement);

  if (isCapsLockAction(action.id)) {
    if (activeContext !== "detail" && activeContext !== "detail.progressForm") {
      return null;
    }
    return pickCapsLockAction(inTextInput);
  }

  if (inTextInput && !shouldRunInTextInput(action.id)) return null;
  if (!inTextInput && action.id === "detail.textIndent") return null;

  return action.id;
}

function matchAction(
  event: KeyboardEvent,
  activeContext: ShortcutContext,
): ShortcutActionId | null {
  const pressed = eventToBinding(event);
  if (!pressed) return null;

  for (const context of contextsToSearch(activeContext)) {
    for (const action of actionsForContext(context)) {
      const matched = matchActionInContext(
        action,
        pressed,
        activeContext,
        context,
      );
      if (matched) return matched;
    }
  }

  return null;
}

export function handleShortcutKeydown(
  event: KeyboardEvent,
  appContext: AppShortcutContext,
): boolean {
  if (shortcutUiState.isRecording.value) return false;
  if (isModifierOnlyEvent(event) && event.type === "keydown") return false;

  const activeContext = resolveActiveContext(appContext);
  const actionId = matchAction(event, activeContext);
  if (!actionId) return false;

  const registration = handlers.get(actionId);
  if (!registration || !registration.enabled()) return false;

  event.preventDefault();
  event.stopPropagation();
  void registration.handler(event);
  return true;
}

export function findBindingConflict(
  draft: Record<string, string>,
  actionId: ShortcutActionId,
  binding: string,
): ShortcutActionDef | null {
  const normalized = normalizeBinding(binding);
  const action = SHORTCUT_ACTIONS.find((item) => item.id === actionId);
  if (!action) return null;

  for (const candidate of SHORTCUT_ACTIONS) {
    if (candidate.id === actionId) continue;
    if (candidate.context !== action.context) continue;
    if (candidate.linkedIds?.includes(actionId)) continue;
    if (action.linkedIds?.includes(candidate.id)) continue;

    const existing = draft[candidate.id] ?? candidate.defaultBinding;
    if (existing && bindingsEqual(normalizeBinding(existing), normalized)) {
      return candidate;
    }
  }

  return null;
}

export function applyLinkedBindings(
  draft: Record<string, string>,
  actionId: ShortcutActionId,
  binding: string,
): Record<string, string> {
  const next = { ...draft, [actionId]: binding };
  const action = SHORTCUT_ACTIONS.find((item) => item.id === actionId);
  for (const linkedId of action?.linkedIds ?? []) {
    next[linkedId] = binding;
  }
  return next;
}

export function useShortcutBindings() {
  return {
    bindings,
    loadShortcutBindings,
    saveShortcutBindingsToServer,
    getEffectiveBinding,
    getEffectiveBindings,
  };
}
