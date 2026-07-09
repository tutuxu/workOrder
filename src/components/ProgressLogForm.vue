<script setup lang="ts">
import { computed, ref, type Ref } from "vue";
import dayjs from "dayjs";
import AttachmentGallery from "./AttachmentGallery.vue";
import { useStatusConfig } from "../composables/useStatusConfig";
import { getEffectiveBinding } from "../composables/useShortcuts";
import { bindingsEqual, eventToBinding } from "../utils/shortcutFormat";
import { insertTextIndent } from "../utils/keyboard";
import type { StatusField } from "../types";

const props = defineProps<{
  editingLogId?: number | null;
  inline?: boolean;
}>();

const emit = defineEmits<{
  save: [];
  cancel: [];
}>();

const title = defineModel<string>("title", { required: true });
const status = defineModel<string>("status", { required: true });
const content = defineModel<string>("content", { required: true });
const extraFieldValues = defineModel<Record<string, string>>("extraFieldValues", {
  required: true,
});

const { statusOptions, fieldsForStatus } = useStatusConfig();

const progressActiveFields = computed(() => fieldsForStatus(status.value));
const isEdit = computed(() => props.editingLogId != null);

const galleryRef = ref<InstanceType<typeof AttachmentGallery> | null>(null);

function fieldInputType(field: StatusField): "text" | "textarea" {
  return field.type === "textarea" ? "textarea" : "text";
}

function getExtraFieldText(key: string): string {
  return extraFieldValues.value[key] ?? "";
}

function setExtraFieldText(key: string, value: string) {
  extraFieldValues.value = { ...extraFieldValues.value, [key]: value };
}

function getExtraFieldDate(key: string): number | null {
  const raw = extraFieldValues.value[key];
  if (!raw) return null;
  const parsed = dayjs(raw);
  return parsed.isValid() ? parsed.valueOf() : null;
}

function setExtraFieldDate(key: string, value: number | null) {
  const next = { ...extraFieldValues.value };
  if (value == null) {
    delete next[key];
  } else {
    next[key] = dayjs(value).format("YYYY-MM-DDTHH:mm:ss");
  }
  extraFieldValues.value = next;
}

function onExtraFieldKeydown(e: KeyboardEvent, key: string) {
  const fieldRef: Ref<string> = {
    get value() {
      return extraFieldValues.value[key] ?? "";
    },
    set value(v: string) {
      setExtraFieldText(key, v);
    },
  } as Ref<string>;
  insertTextIndent(fieldRef, e);
}

function onTextKeydown(e: KeyboardEvent, valueRef: Ref<string>) {
  const binding = getEffectiveBinding("detail.textIndent");
  const pressed = eventToBinding(e);
  if (binding && pressed && bindingsEqual(binding, pressed)) {
    insertTextIndent(valueRef, e);
  }
}

function onTitleKeydown(e: KeyboardEvent) {
  onTextKeydown(e, title);
}

function onContentKeydown(e: KeyboardEvent) {
  onTextKeydown(e, content);
}

defineExpose({
  clearStaged: () => galleryRef.value?.clearStaged(),
  uploadStaged: (ownerId: number) => galleryRef.value?.uploadStaged(ownerId),
});
</script>

<template>
  <component
    :is="inline ? 'div' : 'n-card'"
    v-bind="inline ? { class: 'progress-form-inline' } : { size: 'small', title: isEdit ? '编辑过程' : '添加过程', style: 'margin-top: 12px' }"
  >
    <n-form label-placement="top">
      <n-form-item label="标题" required>
        <n-input
          v-model:value="title"
          placeholder="过程标题"
          @keydown="onTitleKeydown"
        />
      </n-form-item>
      <n-form-item label="状态">
        <n-radio-group v-model:value="status" data-field="progressStatus">
          <n-space>
            <n-radio
              v-for="opt in statusOptions"
              :key="opt.value"
              :value="opt.value"
              :label="opt.label"
            />
          </n-space>
        </n-radio-group>
      </n-form-item>
      <template v-for="field in progressActiveFields" :key="field.key">
        <n-form-item :label="field.label" :required="field.required">
          <n-date-picker
            v-if="field.type === 'date'"
            :value="getExtraFieldDate(field.key)"
            type="datetime"
            clearable
            style="width: 100%"
            @update:value="(v: number | null) => setExtraFieldDate(field.key, v)"
          />
          <n-input
            v-else
            :value="getExtraFieldText(field.key)"
            :type="fieldInputType(field)"
            :rows="field.type === 'textarea' ? 3 : undefined"
            @update:value="(v: string) => setExtraFieldText(field.key, v)"
            @keydown="(e: KeyboardEvent) => onExtraFieldKeydown(e, field.key)"
          />
        </n-form-item>
      </template>
      <n-form-item label="详细内容">
        <n-input
          v-model:value="content"
          type="textarea"
          :rows="3"
          placeholder="可选，展开后可见"
          @keydown="onContentKeydown"
        />
      </n-form-item>
      <n-form-item label="图片">
        <AttachmentGallery
          ref="galleryRef"
          owner-type="progress_log"
          :owner-id="editingLogId ?? undefined"
        />
      </n-form-item>
    </n-form>
    <n-space>
      <n-button type="primary" :keyboard="false" @click="emit('save')">
        {{ isEdit ? "保存修改" : "保存过程" }}
      </n-button>
      <n-button @click="emit('cancel')">取消</n-button>
    </n-space>
  </component>
</template>
