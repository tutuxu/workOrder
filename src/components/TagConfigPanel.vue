<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useDialog, useMessage } from "naive-ui";
import { VueDraggable } from "vue-draggable-plus";
import type { TagConfig, TagDefinition } from "../types";
import { sortedTags } from "../types";
import * as tagConfigApi from "../api/tagConfig";
import { useTagConfig } from "../composables/useTagConfig";
import StatusColorPicker from "./StatusColorPicker.vue";
import { DEFAULT_STATUS_COLOR, normalizeStatusColor } from "../utils/statusColors";

const message = useMessage();
const dialog = useDialog();
const { load: reloadGlobalConfig } = useTagConfig();

const loading = ref(false);
const saving = ref(false);
const backingUp = ref(false);
const restoring = ref(false);
const draft = ref<TagConfig | null>(null);
const selectedTagId = ref<string | null>(null);

const tagList = computed({
  get: () => (draft.value ? sortedTags(draft.value) : []),
  set: (next: TagDefinition[]) => {
    if (!draft.value) return;
    draft.value = {
      ...draft.value,
      tags: next.map((item, index) => ({ ...item, order: index })),
    };
  },
});

const selectedTag = computed(
  () => tagList.value.find((t) => t.id === selectedTagId.value) ?? null,
);

function toPlainConfig(base: TagConfig): TagConfig {
  return {
    version: base.version,
    tags: sortedTags(base).map((t, index) => ({
      id: t.id,
      label: t.label,
      order: index,
      color: normalizeStatusColor(t.color),
    })),
  };
}

onMounted(async () => {
  loading.value = true;
  try {
    draft.value = toPlainConfig(await tagConfigApi.getTagConfig());
    selectedTagId.value = draft.value.tags[0]?.id ?? null;
  } catch (error) {
    message.error(`加载标签配置失败：${error}`);
  } finally {
    loading.value = false;
  }
});

function selectTag(id: string) {
  selectedTagId.value = id;
}

function addTag() {
  if (!draft.value) return;
  const baseId = `TAG_${Date.now()}`;
  const next: TagDefinition = {
    id: baseId,
    label: "新标签",
    order: draft.value.tags.length,
    color: DEFAULT_STATUS_COLOR,
  };
  draft.value = {
    ...draft.value,
    tags: [...draft.value.tags, next],
  };
  selectedTagId.value = baseId;
}

async function removeTag(id: string) {
  if (!draft.value) return;
  let affected = 0;
  try {
    affected = await tagConfigApi.countWorkOrdersByTag(id);
  } catch {
    affected = 0;
  }
  dialog.warning({
    title: "删除标签",
    content:
      affected > 0
        ? `将从 ${affected} 条代办中移除此标签。确定删除？`
        : "确定删除此标签？",
    positiveText: "删除",
    negativeText: "取消",
    onPositiveClick: () => {
      if (!draft.value) return;
      draft.value = {
        ...draft.value,
        tags: draft.value.tags.filter((t) => t.id !== id),
      };
      if (selectedTagId.value === id) {
        selectedTagId.value = draft.value.tags[0]?.id ?? null;
      }
    },
  });
}

function updateSelectedTag(patch: Partial<TagDefinition>) {
  if (!draft.value || !selectedTag.value) return;
  const nextPatch = { ...patch };
  if (nextPatch.color !== undefined) {
    nextPatch.color = normalizeStatusColor(nextPatch.color);
  }
  draft.value = {
    ...draft.value,
    tags: draft.value.tags.map((t) =>
      t.id === selectedTag.value!.id ? { ...t, ...nextPatch } : t,
    ),
  };
}

async function save() {
  if (!draft.value) return;
  saving.value = true;
  try {
    const normalized = toPlainConfig(draft.value);
    await tagConfigApi.saveTagConfig(normalized);
    draft.value = toPlainConfig(normalized);
    await reloadGlobalConfig(true);
    message.success("标签配置已保存");
  } catch (error) {
    message.error(`保存失败：${error}`);
  } finally {
    saving.value = false;
  }
}

async function backup() {
  backingUp.value = true;
  try {
    const savePath = await tagConfigApi.pickTagConfigSavePath();
    if (!savePath) return;
    const result = await tagConfigApi.exportTagConfig(savePath);
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
    content: "将用备份文件替换当前标签配置，未保存的修改将丢失。是否继续？",
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
    const filePath = await tagConfigApi.pickTagConfigFile();
    if (!filePath) return;
    const config = await tagConfigApi.importTagConfig(filePath);
    draft.value = toPlainConfig(config);
    selectedTagId.value = draft.value.tags[0]?.id ?? null;
    await reloadGlobalConfig(true);
    message.success("标签配置已恢复");
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
        管理代办标签：自定义名称、颜色与排序。保存后立即生效。
      </n-text>

      <n-grid :cols="2" :x-gap="16">
        <n-gi>
          <n-text strong>标签列表</n-text>
          <VueDraggable
            v-model="tagList"
            :animation="150"
            handle=".tag-drag-row"
            style="margin-top: 8px"
          >
            <div
              v-for="item in tagList"
              :key="item.id"
              class="tag-drag-row"
              :class="{ 'tag-selected': item.id === selectedTagId }"
              @click="selectTag(item.id)"
            >
              <span class="tag-label-row">
                <span
                  class="tag-color-dot"
                  :style="{ backgroundColor: item.color ?? DEFAULT_STATUS_COLOR }"
                />
                {{ item.label }}
              </span>
              <n-button text type="error" size="tiny" @click.stop="removeTag(item.id)">
                删除
              </n-button>
            </div>
          </VueDraggable>
          <n-button style="margin-top: 8px" @click="addTag">添加标签</n-button>
        </n-gi>

        <n-gi>
          <template v-if="selectedTag">
            <n-text strong>编辑标签</n-text>
            <n-grid :cols="2" :x-gap="12" style="margin-top: 8px">
              <n-gi>
                <n-form-item label="显示名称">
                  <n-input
                    :value="selectedTag.label"
                    @update:value="(v: string) => updateSelectedTag({ label: v })"
                  />
                </n-form-item>
              </n-gi>
              <n-gi>
                <n-form-item label="标签 ID">
                  <n-input :value="selectedTag.id" readonly />
                </n-form-item>
              </n-gi>
            </n-grid>

            <n-form-item label="徽章颜色" style="margin-top: 8px">
              <StatusColorPicker
                :value="selectedTag.color ?? DEFAULT_STATUS_COLOR"
                @update:value="(v: string) => updateSelectedTag({ color: v })"
              />
            </n-form-item>
          </template>
        </n-gi>
      </n-grid>

      <div class="footer-actions">
        <n-space>
          <n-button :loading="backingUp" @click="backup">备份...</n-button>
          <n-button type="error" :loading="restoring" @click="restore">恢复...</n-button>
        </n-space>
        <n-button type="primary" :loading="saving" @click="save">保存标签配置</n-button>
      </div>
    </n-form>
  </n-spin>
</template>

<style scoped>
.tag-drag-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  margin-bottom: 6px;
  cursor: pointer;
}

.tag-selected {
  border-color: var(--n-primary-color);
  background: rgba(24, 160, 88, 0.08);
}

.tag-label-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tag-color-dot {
  width: 14px;
  height: 14px;
  border-radius: 4px;
  flex-shrink: 0;
  box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.1);
}

.footer-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 16px;
}
</style>
