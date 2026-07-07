<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { onMounted, ref, watch } from "vue";
import { useMessage } from "naive-ui";
import * as attachmentApi from "../api/attachments";
import type { Attachment, OwnerType } from "../types";

const props = withDefaults(
  defineProps<{
    ownerType: OwnerType;
    ownerId?: number;
    readonly?: boolean;
  }>(),
  { readonly: false },
);

const emit = defineEmits<{
  "staged-change": [files: File[]];
}>();

const message = useMessage();
const attachments = ref<Attachment[]>([]);
const staged = ref<{ file: File; previewUrl: string }[]>([]);

function thumbSrc(att: Attachment): string {
  return convertFileSrc(att.filePath);
}

function stagedSrc(item: { previewUrl: string }): string {
  return item.previewUrl;
}

async function load() {
  if (props.ownerId == null) return;
  attachments.value = await attachmentApi.listAttachments(
    props.ownerType,
    props.ownerId,
  );
}

function emitStagedChange() {
  emit(
    "staged-change",
    staged.value.map((s) => s.file),
  );
}

function addStaged(file: File) {
  const previewUrl = URL.createObjectURL(file);
  staged.value = [...staged.value, { file, previewUrl }];
  emitStagedChange();
}

function clearStaged() {
  for (const item of staged.value) {
    URL.revokeObjectURL(item.previewUrl);
  }
  staged.value = [];
  emitStagedChange();
}

async function handlePickFile() {
  try {
    const path = await attachmentApi.pickAttachmentFile();
    if (!path) return;
    if (props.ownerId == null) {
      const url = convertFileSrc(path);
      const resp = await fetch(url);
      const blob = await resp.blob();
      const name = path.split(/[/\\]/).pop() ?? "image.png";
      const file = new File([blob], name, { type: blob.type || "image/png" });
      addStaged(file);
    } else {
      await attachmentApi.addAttachmentFromFile(
        props.ownerType,
        props.ownerId,
        path,
      );
      await load();
    }
  } catch (e) {
    message.error(String(e));
  }
}

async function handlePaste(e: ClipboardEvent) {
  if (props.readonly) return;
  const items = e.clipboardData?.items;
  if (!items) return;
  for (const item of items) {
    if (!item.type.startsWith("image/")) continue;
    e.preventDefault();
    const file = item.getAsFile();
    if (!file) continue;
    try {
      if (props.ownerId == null) {
        addStaged(file);
      } else {
        const buf = new Uint8Array(await file.arrayBuffer());
        await attachmentApi.addAttachmentFromBytes(
          props.ownerType,
          props.ownerId,
          file.name,
          file.type,
          buf,
        );
        await load();
      }
    } catch (err) {
      message.error(String(err));
    }
    break;
  }
}

async function handleDeleteAttachment(id: number) {
  try {
    await attachmentApi.deleteAttachment(id);
    await load();
  } catch (e) {
    message.error(String(e));
  }
}

function removeStaged(index: number) {
  const item = staged.value[index];
  if (item) {
    URL.revokeObjectURL(item.previewUrl);
  }
  staged.value = staged.value.filter((_, i) => i !== index);
  emitStagedChange();
}

async function uploadStaged(ownerId: number) {
  for (const item of staged.value) {
    const buf = new Uint8Array(await item.file.arrayBuffer());
    await attachmentApi.addAttachmentFromBytes(
      props.ownerType,
      ownerId,
      item.file.name,
      item.file.type || "image/png",
      buf,
    );
  }
  clearStaged();
  await load();
}

onMounted(load);

watch(
  () => props.ownerId,
  async () => {
    clearStaged();
    await load();
  },
);

defineExpose({ uploadStaged, clearStaged });
</script>

<template>
  <div class="attachment-gallery" tabindex="0" @paste="handlePaste">
    <div v-if="attachments.length > 0 || staged.length > 0" class="attachment-grid">
      <div v-for="att in attachments" :key="att.id ?? att.fileName" class="attachment-thumb">
        <n-image :src="thumbSrc(att)" width="80" height="80" object-fit="cover" />
        <n-popconfirm
          v-if="!readonly"
          @positive-click="att.id != null && handleDeleteAttachment(att.id)"
        >
          <template #trigger>
            <n-button class="delete-btn" size="tiny" type="error" circle>×</n-button>
          </template>
          确定删除该图片吗？
        </n-popconfirm>
      </div>
      <div v-for="(item, index) in staged" :key="'staged-' + index" class="attachment-thumb">
        <n-image :src="stagedSrc(item)" width="80" height="80" object-fit="cover" />
        <n-popconfirm v-if="!readonly" @positive-click="removeStaged(index)">
          <template #trigger>
            <n-button class="delete-btn" size="tiny" type="error" circle>×</n-button>
          </template>
          确定删除该图片吗？
        </n-popconfirm>
      </div>
    </div>
    <n-space v-if="!readonly" align="center">
      <n-button size="small" @click="handlePickFile">添加图片</n-button>
      <span class="attachment-hint">支持选择图片或 Ctrl+V 粘贴</span>
    </n-space>
  </div>
</template>
