<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useDialog, useMessage } from "naive-ui";
import { VueDraggable } from "vue-draggable-plus";
import type {
  StatusConfig,
  StatusDefinition,
  StatusField,
  StatusFieldType,
} from "../types";
import { sortedStatuses } from "../types";
import * as statusConfigApi from "../api/statusConfig";
import { useStatusConfig } from "../composables/useStatusConfig";
import StatusColorPicker from "./StatusColorPicker.vue";
import { DEFAULT_STATUS_COLOR, normalizeStatusColor } from "../utils/statusColors";

const message = useMessage();
const dialog = useDialog();
const { load: reloadGlobalConfig } = useStatusConfig();

const loading = ref(false);
const saving = ref(false);
const backingUp = ref(false);
const restoring = ref(false);
const draft = ref<StatusConfig | null>(null);
const selectedStatusId = ref<string | null>(null);

const statusList = computed({
  get: () => (draft.value ? sortedStatuses(draft.value) : []),
  set: (next: StatusDefinition[]) => {
    if (!draft.value) return;
    draft.value = {
      ...draft.value,
      statuses: next.map((item, index) => ({ ...item, order: index })),
    };
  },
});

const selectedStatus = computed(() =>
  statusList.value.find((s) => s.id === selectedStatusId.value) ?? null,
);

const fieldTypeOptions: { label: string; value: StatusFieldType }[] = [
  { label: "单行文本", value: "text" },
  { label: "多行文本", value: "textarea" },
  { label: "日期时间", value: "date" },
];

function toPlainConfig(base: StatusConfig): StatusConfig {
  return {
    version: base.version,
    statuses: sortedStatuses(base).map((s, index) => ({
      id: s.id,
      label: s.label,
      order: index,
      color: normalizeStatusColor(s.color),
      fields: s.fields.map((f) => ({
        key: f.key,
        label: f.label,
        type: f.type,
        required: f.required,
      })),
    })),
  };
}

onMounted(async () => {
  loading.value = true;
  try {
    draft.value = toPlainConfig(await statusConfigApi.getStatusConfig());
    selectedStatusId.value = draft.value.statuses[0]?.id ?? null;
  } catch (error) {
    message.error(`加载代办状态失败：${error}`);
  } finally {
    loading.value = false;
  }
});

function selectStatus(id: string) {
  selectedStatusId.value = id;
}

function addStatus() {
  if (!draft.value) return;
  const baseId = `STATUS_${Date.now()}`;
  const next: StatusDefinition = {
    id: baseId,
    label: "新状态",
    order: draft.value.statuses.length,
    color: DEFAULT_STATUS_COLOR,
    fields: [],
  };
  draft.value = {
    ...draft.value,
    statuses: [...draft.value.statuses, next],
  };
  selectedStatusId.value = baseId;
}

function removeStatus(id: string) {
  if (!draft.value || draft.value.statuses.length <= 1) {
    message.warning("至少保留一个状态");
    return;
  }
  dialog.warning({
    title: "删除状态",
    content: "已有工单使用此状态时将显示为未知状态。确定删除？",
    positiveText: "删除",
    negativeText: "取消",
    onPositiveClick: () => {
      if (!draft.value) return;
      draft.value = {
        ...draft.value,
        statuses: draft.value.statuses.filter((s) => s.id !== id),
      };
      if (selectedStatusId.value === id) {
        selectedStatusId.value = draft.value.statuses[0]?.id ?? null;
      }
    },
  });
}

function updateSelectedStatus(patch: Partial<StatusDefinition>) {
  if (!draft.value || !selectedStatus.value) return;
  const nextPatch = { ...patch };
  if (nextPatch.color !== undefined) {
    nextPatch.color = normalizeStatusColor(nextPatch.color);
  }
  draft.value = {
    ...draft.value,
    statuses: draft.value.statuses.map((s) =>
      s.id === selectedStatus.value!.id ? { ...s, ...nextPatch } : s,
    ),
  };
}

function addField() {
  if (!selectedStatus.value) return;
  const key = `field_${Date.now()}`;
  const field: StatusField = {
    key,
    label: "新字段",
    type: "text",
    required: false,
  };
  updateSelectedStatus({
    fields: [...selectedStatus.value.fields, field],
  });
}

function updateField(index: number, patch: Partial<StatusField>) {
  if (!selectedStatus.value) return;
  const fields = selectedStatus.value.fields.map((f, i) =>
    i === index ? { ...f, ...patch } : f,
  );
  updateSelectedStatus({ fields });
}

function removeField(index: number) {
  if (!selectedStatus.value) return;
  updateSelectedStatus({
    fields: selectedStatus.value.fields.filter((_, i) => i !== index),
  });
}

async function save() {
  if (!draft.value) return;
  saving.value = true;
  try {
    const normalized = toPlainConfig(draft.value);
    await statusConfigApi.saveStatusConfig(normalized);
    draft.value = toPlainConfig(normalized);
    await reloadGlobalConfig(true);
    message.success("代办状态已保存");
  } catch (error) {
    message.error(`保存失败：${error}`);
  } finally {
    saving.value = false;
  }
}

async function backup() {
  backingUp.value = true;
  try {
    const savePath = await statusConfigApi.pickStatusConfigSavePath();
    if (!savePath) return;
    const result = await statusConfigApi.exportStatusConfig(savePath);
    message.success(`备份已保存：${result.filePath}`);
  } catch (error) {
    message.error(`备份失败：${error}`);
  } finally {
    backingUp.value = false;
  }
}

function restore() {
  dialog.warning({
    title: "确认恢复",
    content:
      "将用备份文件替换当前代办状态配置，未保存的修改将丢失。是否继续？",
    positiveText: "继续恢复",
    negativeText: "取消",
    onPositiveClick: () => {
      void doRestore();
    },
  });
}

async function doRestore() {
  restoring.value = true;
  try {
    const filePath = await statusConfigApi.pickStatusConfigFile();
    if (!filePath) return;
    const config = await statusConfigApi.importStatusConfig(filePath);
    draft.value = toPlainConfig(config);
    selectedStatusId.value = draft.value.statuses[0]?.id ?? null;
    await reloadGlobalConfig(true);
    message.success("代办状态已恢复");
  } catch (error) {
    message.error(`恢复失败：${error}`);
  } finally {
    restoring.value = false;
  }
}
</script>

<template>
  <n-spin :show="loading">
    <n-form label-placement="top">
      <n-text depth="3" style="display: block; font-size: 13px; margin-bottom: 12px">
        配置代办状态及每个状态下需要填写的信息栏。保存后立即生效。
      </n-text>

      <n-grid :cols="2" :x-gap="16">
        <n-gi>
          <n-text strong>状态列表</n-text>
          <VueDraggable
            v-model="statusList"
            :animation="150"
            handle=".status-drag-row"
            style="margin-top: 8px"
          >
            <div
              v-for="item in statusList"
              :key="item.id"
              class="status-drag-row"
              :class="{ 'status-selected': item.id === selectedStatusId }"
              @click="selectStatus(item.id)"
            >
              <span class="status-label-row">
                <span
                  class="status-color-dot"
                  :style="{ backgroundColor: item.color ?? DEFAULT_STATUS_COLOR }"
                />
                {{ item.label }}
              </span>
              <n-button
                text
                type="error"
                size="tiny"
                @click.stop="removeStatus(item.id)"
              >
                删除
              </n-button>
            </div>
          </VueDraggable>
          <n-button style="margin-top: 8px" @click="addStatus">添加状态</n-button>
        </n-gi>

        <n-gi>
          <template v-if="selectedStatus">
            <n-text strong>编辑状态</n-text>
            <n-grid :cols="2" :x-gap="12" style="margin-top: 8px">
              <n-gi>
                <n-form-item label="显示名称">
                  <n-input
                    :value="selectedStatus.label"
                    @update:value="(v: string) => updateSelectedStatus({ label: v })"
                  />
                </n-form-item>
              </n-gi>
              <n-gi>
                <n-form-item label="状态 ID">
                  <n-input :value="selectedStatus.id" readonly />
                </n-form-item>
              </n-gi>
            </n-grid>

            <n-form-item label="列表颜色" style="margin-top: 8px">
              <StatusColorPicker
                :value="selectedStatus.color ?? DEFAULT_STATUS_COLOR"
                @update:value="(v: string) => updateSelectedStatus({ color: v })"
              />
            </n-form-item>

            <n-divider />

            <div class="fields-header">
              <n-text strong>信息栏</n-text>
              <n-button size="small" @click="addField">添加字段</n-button>
            </div>
            <div
              v-for="(field, index) in selectedStatus.fields"
              :key="field.key"
              class="field-editor"
            >
              <n-grid :cols="2" :x-gap="12">
                <n-gi>
                  <n-form-item label="标签">
                    <n-input
                      :value="field.label"
                      @update:value="(v: string) => updateField(index, { label: v })"
                    />
                  </n-form-item>
                </n-gi>
                <n-gi>
                  <n-form-item label="字段 key">
                    <n-input :value="field.key" readonly />
                  </n-form-item>
                </n-gi>
              </n-grid>
              <n-grid :cols="2" :x-gap="12">
                <n-gi>
                  <n-form-item label="类型">
                    <n-select
                      :value="field.type"
                      :options="fieldTypeOptions"
                      @update:value="(v: StatusFieldType) => updateField(index, { type: v })"
                    />
                  </n-form-item>
                </n-gi>
                <n-gi>
                  <n-form-item label="必填">
                    <n-switch
                      :value="field.required"
                      @update:value="(v: boolean) => updateField(index, { required: v })"
                    />
                  </n-form-item>
                </n-gi>
              </n-grid>
              <n-button text type="error" @click="removeField(index)">删除字段</n-button>
              <n-divider />
            </div>
          </template>
        </n-gi>
      </n-grid>

      <div class="footer-actions">
        <n-space>
          <n-button :loading="backingUp" @click="backup">备份...</n-button>
          <n-button type="error" :loading="restoring" @click="restore">恢复...</n-button>
        </n-space>
        <n-button type="primary" :loading="saving" @click="save">保存代办状态</n-button>
      </div>
    </n-form>
  </n-spin>
</template>

<style scoped>
.status-drag-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  margin-bottom: 6px;
  cursor: pointer;
}

.status-selected {
  border-color: var(--n-primary-color);
  background: rgba(24, 160, 88, 0.08);
}

.status-label-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-color-dot {
  width: 14px;
  height: 14px;
  border-radius: 4px;
  flex-shrink: 0;
  box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.1);
}

.fields-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 8px;
}

.field-editor {
  margin-top: 8px;
}

.footer-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 16px;
}
</style>
