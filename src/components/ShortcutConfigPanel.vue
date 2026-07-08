<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useMessage } from "naive-ui";
import {
  applyLinkedBindings,
  findBindingConflict,
  getDefaultBindingsRecord,
  getEffectiveBindings,
  loadShortcutBindings,
  saveShortcutBindingsToServer,
  shortcutUiState,
  useShortcutBindings,
} from "../composables/useShortcuts";
import {
  SHORTCUT_ACTIONS,
  SHORTCUT_CONTEXT_ORDER,
  actionsForContext,
  type ShortcutActionId,
} from "../types/shortcuts";
import {
  eventToBinding,
  formatBindingDisplay,
  isModifierOnlyEvent,
  normalizeBinding,
} from "../utils/shortcutFormat";

const message = useMessage();
const { bindings } = useShortcutBindings();

const loading = ref(false);
const saving = ref(false);
const draft = ref<Record<string, string>>({});
const recordingActionId = ref<ShortcutActionId | null>(null);

const groupedActions = computed(() =>
  SHORTCUT_CONTEXT_ORDER.map((context) => ({
    context,
    label: actionsForContext(context)[0]?.contextLabel ?? context,
    actions: actionsForContext(context),
  })).filter((group) => group.actions.length > 0),
);

const hasChanges = computed(() => {
  const effectiveSaved = getEffectiveBindings();
  const keys = new Set([
    ...Object.keys(effectiveSaved),
    ...Object.keys(draft.value),
    ...SHORTCUT_ACTIONS.map((action) => action.id),
  ]);
  for (const key of keys) {
    const saved = effectiveSaved[key as ShortcutActionId];
    const current = draft.value[key as ShortcutActionId] ?? saved;
    if ((saved ?? "") !== (current ?? "")) return true;
  }
  return false;
});

function buildDraftFromSaved(): Record<string, string> {
  const draftState = { ...getDefaultBindingsRecord() };
  for (const action of SHORTCUT_ACTIONS) {
    if (Object.prototype.hasOwnProperty.call(bindings.value, action.id)) {
      draftState[action.id] = bindings.value[action.id] ?? "";
    }
  }
  return draftState;
}

onMounted(async () => {
  loading.value = true;
  try {
    await loadShortcutBindings();
    draft.value = buildDraftFromSaved();
  } catch (error) {
    message.error(`加载快捷键失败：${error}`);
  } finally {
    loading.value = false;
  }
});

function displayBinding(actionId: ShortcutActionId): string {
  if (Object.prototype.hasOwnProperty.call(draft.value, actionId)) {
    const binding = draft.value[actionId];
    return binding ? formatBindingDisplay(binding) : "未设置";
  }
  const defaults = getDefaultBindingsRecord();
  return formatBindingDisplay(defaults[actionId]);
}

function startRecording(actionId: ShortcutActionId) {
  recordingActionId.value = actionId;
  shortcutUiState.isRecording.value = true;
}

function stopRecording() {
  recordingActionId.value = null;
  shortcutUiState.isRecording.value = false;
}

function onRecordKeydown(event: KeyboardEvent) {
  if (!recordingActionId.value) return;

  if (event.key === "Escape") {
    event.preventDefault();
    stopRecording();
    return;
  }

  if (isModifierOnlyEvent(event)) return;

  const binding = eventToBinding(event);
  if (!binding) return;

  event.preventDefault();
  event.stopPropagation();

  const actionId = recordingActionId.value;
  const conflict = findBindingConflict(draft.value, actionId, binding);
  if (conflict) {
    message.warning(`与「${conflict.label}」冲突，请使用其他快捷键`);
    return;
  }

  draft.value = applyLinkedBindings(draft.value, actionId, normalizeBinding(binding));
  stopRecording();
}

function clearBinding(actionId: ShortcutActionId) {
  const next = { ...draft.value, [actionId]: "" };
  const action = SHORTCUT_ACTIONS.find((item) => item.id === actionId);
  for (const linkedId of action?.linkedIds ?? []) {
    next[linkedId] = "";
  }
  draft.value = next;
}

function restoreDefaults() {
  draft.value = { ...getDefaultBindingsRecord() };
}

async function save() {
  saving.value = true;
  try {
    const defaults = getDefaultBindingsRecord();
    const toSave: Record<string, string> = {};
    for (const action of SHORTCUT_ACTIONS) {
      if (!Object.prototype.hasOwnProperty.call(draft.value, action.id)) continue;
      const value = draft.value[action.id] ?? "";
      const defaultValue = defaults[action.id] ?? "";
      if (value !== defaultValue) {
        toSave[action.id] = value;
      }
    }
    await saveShortcutBindingsToServer(toSave);
    draft.value = buildDraftFromSaved();
    message.success("快捷键已保存");
  } catch (error) {
    message.error(`保存快捷键失败：${error}`);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="shortcut-config-panel" @keydown.capture="onRecordKeydown">
    <n-spin :show="loading">
      <n-text depth="3" style="display: block; font-size: 13px; margin-bottom: 12px">
        快捷键仅在对应界面生效。带确认的操作（删除）会弹出确认对话框。录制时按 Esc 取消。
      </n-text>

      <div v-for="group in groupedActions" :key="group.context" class="shortcut-group">
        <n-text strong style="display: block; margin-bottom: 8px">{{ group.label }}</n-text>
        <div
          v-for="action in group.actions"
          :key="action.id"
          class="shortcut-row"
        >
          <span class="shortcut-label">{{ action.label }}</span>
          <n-tag
            size="small"
            :type="recordingActionId === action.id ? 'warning' : 'default'"
            :bordered="false"
          >
            {{
              recordingActionId === action.id
                ? "请按下快捷键…"
                : displayBinding(action.id)
            }}
          </n-tag>
          <n-space size="small">
            <n-button
              size="tiny"
              :type="recordingActionId === action.id ? 'warning' : 'default'"
              @click="startRecording(action.id)"
            >
              录制
            </n-button>
            <n-button size="tiny" @click="clearBinding(action.id)">清除</n-button>
          </n-space>
        </div>
      </div>

      <n-space style="margin-top: 12px">
        <n-button :loading="saving" type="primary" :disabled="!hasChanges" @click="save">
          保存快捷键
        </n-button>
        <n-button @click="restoreDefaults">恢复默认</n-button>
      </n-space>
    </n-spin>
  </div>
</template>

<style scoped>
.shortcut-group + .shortcut-group {
  margin-top: 16px;
}

.shortcut-row {
  display: grid;
  grid-template-columns: 120px 1fr auto;
  gap: 8px;
  align-items: center;
  margin-bottom: 8px;
}

.shortcut-label {
  font-size: 13px;
}
</style>
