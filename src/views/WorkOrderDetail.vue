<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, type Ref } from "vue";
import dayjs from "dayjs";
import { useMessage } from "naive-ui";
import { formatServerDateTime } from "../utils/datetime";
import * as workOrderApi from "../api/workOrders";
import * as progressLogApi from "../api/progressLogs";
import AttachmentGallery from "../components/AttachmentGallery.vue";
import { useStatusConfig } from "../composables/useStatusConfig";
import type {
  ProgressLog,
  ProgressLogInput,
  StatusField,
  WorkOrder,
  WorkOrderInput,
} from "../types";
import {
  cycleOption,
  focusNextElement,
  insertTextIndent,
  isCapsLockKey,
  resolveTextInput,
} from "../utils/keyboard";
import { tagStyleForStatus } from "../utils/statusColors";

const props = defineProps<{
  workOrder: WorkOrder | null;
}>();

const emit = defineEmits<{
  saved: [];
  closed: [];
}>();

const message = useMessage();
const {
  statusOptions,
  statusLabel,
  statusColor,
  fieldsForStatus,
  defaultStatus,
  load: loadStatusConfig,
} = useStatusConfig();

const show = ref(true);
const saving = ref(false);

const title = ref("");
const description = ref("");
const status = ref("NOT_STARTED");
const dueDate = ref<number | null>(null);
const extraFieldValues = ref<Record<string, string>>({});

const logs = ref<ProgressLog[]>([]);
const expandedLogIds = ref<Array<string | number>>([]);
const progressTitle = ref("");
const progressContent = ref("");
const progressStatus = ref("IN_PROGRESS");
const editingLogId = ref<number | null>(null);
const showProgressForm = ref(false);

const workOrderGalleryRef = ref<InstanceType<typeof AttachmentGallery> | null>(null);
const progressGalleryRef = ref<InstanceType<typeof AttachmentGallery> | null>(null);
const modalContainerRef = ref<HTMLElement | null>(null);

const workOrderId = ref<number | undefined>(props.workOrder?.id ?? undefined);
const isNew = computed(() => workOrderId.value == null);
const modalTitle = computed(() => (isNew.value ? "新建代办" : "编辑代办"));
const activeFields = computed(() => fieldsForStatus(status.value));

function bindForm(order: WorkOrder) {
  title.value = order.title ?? "";
  description.value = order.description ?? "";
  status.value = order.status ?? defaultStatus();
  dueDate.value = order.dueDate ? dayjs(order.dueDate).valueOf() : null;
  extraFieldValues.value = { ...(order.extraFields ?? {}) };
  workOrderId.value = order.id ?? undefined;
}

function resetForNew() {
  title.value = "";
  description.value = "";
  status.value = defaultStatus();
  dueDate.value = null;
  extraFieldValues.value = {};
  workOrderId.value = undefined;
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

function buildExtraFieldsPayload(): Record<string, string> | null {
  const payload: Record<string, string> = {};
  for (const field of activeFields.value) {
    const value = extraFieldValues.value[field.key]?.trim();
    if (value) {
      payload[field.key] = value;
    }
  }
  return Object.keys(payload).length > 0 ? payload : null;
}

function validateExtraFieldsClient(): boolean {
  for (const field of activeFields.value) {
    if (!field.required) continue;
    const value = extraFieldValues.value[field.key]?.trim();
    if (!value) {
      message.warning(`请填写${field.label}`);
      return false;
    }
  }
  return true;
}

function buildProgressInput(): ProgressLogInput {
  return {
    title: progressTitle.value.trim(),
    content: progressContent.value.trim() || null,
    status: progressStatus.value,
  };
}

function defaultProgressStatus(): string {
  const options = statusOptions.value;
  const inProgress = options.find((o) => o.value === "IN_PROGRESS");
  return inProgress?.value ?? options[0]?.value ?? defaultStatus();
}

function clearProgressForm() {
  editingLogId.value = null;
  progressTitle.value = "";
  progressContent.value = "";
  progressStatus.value = defaultProgressStatus();
  progressGalleryRef.value?.clearStaged();
  showProgressForm.value = false;
}

function openProgressForm() {
  editingLogId.value = null;
  progressTitle.value = "";
  progressContent.value = "";
  progressStatus.value = defaultProgressStatus();
  progressGalleryRef.value?.clearStaged();
  showProgressForm.value = true;
}

async function loadLogs() {
  if (workOrderId.value == null) {
    logs.value = [];
    return;
  }
  logs.value = await progressLogApi.listProgressLogs(workOrderId.value);
}

onMounted(async () => {
  document.addEventListener("keydown", onGlobalKeydown);
  await loadStatusConfig();
  if (props.workOrder) {
    bindForm(props.workOrder);
    await loadLogs();
  } else {
    resetForNew();
  }
});

watch(
  () => props.workOrder,
  async (order) => {
    if (order) {
      bindForm(order);
      await loadLogs();
    }
  },
);

function buildInput(): WorkOrderInput {
  return {
    title: title.value,
    description: description.value || null,
    status: status.value,
    extraFields: buildExtraFieldsPayload(),
    dueDate: dueDate.value ? dayjs(dueDate.value).format("YYYY-MM-DDTHH:mm:ss") : null,
  };
}

async function save() {
  if (!validateExtraFieldsClient()) return;
  saving.value = true;
  try {
    const input = buildInput();
    if (isNew.value) {
      const created = await workOrderApi.createWorkOrder(input);
      workOrderId.value = created.id ?? undefined;
      if (workOrderId.value != null) {
        await workOrderGalleryRef.value?.uploadStaged(workOrderId.value);
      }
      await flushPendingProgress();
      message.success("已保存");
      emit("saved");
      close();
    } else {
      await workOrderApi.updateWorkOrder(workOrderId.value!, input);
      await flushPendingProgress();
      message.success("已保存");
      emit("saved");
      close();
    }
  } catch (e) {
    message.error(String(e));
  } finally {
    saving.value = false;
  }
}

async function confirmDelete() {
  if (workOrderId.value == null) return;
  try {
    await workOrderApi.deleteWorkOrder(workOrderId.value);
    message.success("已删除");
    emit("saved");
    close();
  } catch (e) {
    message.error(String(e));
  }
}

function close() {
  show.value = false;
  emit("closed");
}

async function saveProgress() {
  if (workOrderId.value == null) {
    message.warning("请先保存代办事项");
    return;
  }
  if (!progressTitle.value.trim()) {
    message.warning("请填写过程标题");
    return;
  }
  try {
    const input = buildProgressInput();
    if (editingLogId.value != null) {
      await progressLogApi.updateProgressLog(
        editingLogId.value,
        workOrderId.value,
        input,
      );
      clearProgressForm();
    } else {
      const created = await progressLogApi.addProgressLog(workOrderId.value, input);
      const logId = created.id;
      if (logId != null) {
        await progressGalleryRef.value?.uploadStaged(logId);
      }
      clearProgressForm();
    }
    await loadLogs();
  } catch (e) {
    message.error(String(e));
  }
}

async function flushPendingProgress() {
  if (!progressTitle.value.trim() || workOrderId.value == null) return;
  try {
    const input = buildProgressInput();
    if (editingLogId.value != null) {
      await progressLogApi.updateProgressLog(
        editingLogId.value,
        workOrderId.value,
        input,
      );
      clearProgressForm();
    } else {
      await progressLogApi.addProgressLog(workOrderId.value, input);
      clearProgressForm();
    }
  } catch (e) {
    message.error(String(e));
  }
}

function startEdit(log: ProgressLog) {
  progressGalleryRef.value?.clearStaged();
  editingLogId.value = log.id ?? null;
  progressTitle.value = log.title;
  progressContent.value = log.content ?? "";
  progressStatus.value = log.status;
  showProgressForm.value = true;
  if (log.id != null && !expandedLogIds.value.includes(log.id)) {
    expandedLogIds.value = [...expandedLogIds.value, log.id];
  }
}

async function deleteProgress(log: ProgressLog) {
  if (workOrderId.value == null || log.id == null) return;
  try {
    await progressLogApi.deleteProgressLog(log.id, workOrderId.value);
    if (editingLogId.value === log.id) {
      clearProgressForm();
    }
    expandedLogIds.value = expandedLogIds.value.filter((id) => id !== log.id);
    await loadLogs();
  } catch (e) {
    message.error(String(e));
  }
}

function logKey(log: ProgressLog): string | number {
  return log.id ?? log.createdAt;
}

function onTextKeydown(e: KeyboardEvent, valueRef: Ref<string>) {
  insertTextIndent(valueRef, e);
}

function onTitleKeydown(e: KeyboardEvent) {
  onTextKeydown(e, title);
}

function onDescriptionKeydown(e: KeyboardEvent) {
  onTextKeydown(e, description);
}

function onProgressTitleKeydown(e: KeyboardEvent) {
  onTextKeydown(e, progressTitle);
}

function onProgressContentKeydown(e: KeyboardEvent) {
  onTextKeydown(e, progressContent);
}

function onFormKeydown(e: KeyboardEvent) {
  if (!isCapsLockKey(e)) return;
  if (resolveTextInput(e.target)) return;
  e.preventDefault();

  const target = e.target as HTMLElement | null;
  const radioGroup = target?.closest(".n-radio-group") as HTMLElement | null;
  if (radioGroup?.contains(document.activeElement)) {
    if (radioGroup.dataset.field === "status") {
      status.value = cycleOption(status.value, statusOptions.value);
    } else if (radioGroup.dataset.field === "progressStatus") {
      progressStatus.value = cycleOption(progressStatus.value, statusOptions.value);
    }
    return;
  }

  const container = modalContainerRef.value;
  if (container) {
    focusNextElement(container, e.shiftKey);
  }
}

function onGlobalKeydown(e: KeyboardEvent) {
  if (!show.value || saving.value) return;
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
    e.preventDefault();
    void save();
  }
}

onUnmounted(() => {
  document.removeEventListener("keydown", onGlobalKeydown);
});

function fieldInputType(field: StatusField): "text" | "textarea" {
  return field.type === "textarea" ? "textarea" : "text";
}
</script>

<template>
  <n-modal
    v-model:show="show"
    preset="card"
    :title="modalTitle"
    class="work-order-detail-modal"
    style="width: 640px"
    content-scrollable
    @after-leave="close"
  >
    <div ref="modalContainerRef" @keydown="onFormKeydown">
    <n-form label-placement="top">
      <n-form-item label="标题" required>
        <n-input v-model:value="title" @keydown="onTitleKeydown" />
      </n-form-item>
      <n-form-item label="描述">
        <n-input
          v-model:value="description"
          type="textarea"
          :rows="4"
          @keydown="onDescriptionKeydown"
        />
      </n-form-item>
      <n-form-item label="图片">
        <AttachmentGallery
          ref="workOrderGalleryRef"
          owner-type="work_order"
          :owner-id="workOrderId"
        />
      </n-form-item>
      <n-form-item label="状态">
        <n-radio-group v-model:value="status" data-field="status">
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
      <n-form-item label="计划完成时间">
        <n-date-picker v-model:value="dueDate" type="datetime" clearable style="width: 100%" />
      </n-form-item>
      <template v-for="field in activeFields" :key="field.key">
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
    </n-form>

    <h3>处置过程</h3>
    <div v-if="isNew" style="color: #999; margin-bottom: 12px">保存后可追加处置过程</div>
    <template v-else>
      <div v-if="logs.length === 0" style="color: #999; margin-bottom: 12px">暂无过程记录</div>
      <n-collapse v-else v-model:expanded-names="expandedLogIds">
        <n-collapse-item
          v-for="log in logs"
          :key="logKey(log)"
          :name="logKey(log)"
        >
          <template #header>
            <div class="progress-header">
              <span class="progress-title">{{ log.title }}</span>
              <n-tag
                size="small"
                :bordered="false"
                :style="tagStyleForStatus(statusColor(log.status))"
              >
                {{ statusLabel(log.status) }}
              </n-tag>
              <span class="progress-time">{{ formatServerDateTime(log.createdAt) }}</span>
            </div>
          </template>
          <div class="progress-body">
            <p v-if="log.content" class="progress-content">{{ log.content }}</p>
            <p v-else class="progress-content progress-empty">暂无详细内容</p>
            <AttachmentGallery
              v-if="log.id != null"
              owner-type="progress_log"
              :owner-id="log.id"
              readonly
            />
            <n-space>
              <n-button text type="primary" @click="startEdit(log)">编辑</n-button>
              <n-popconfirm @positive-click="deleteProgress(log)">
                <template #trigger>
                  <n-button text type="error">删除</n-button>
                </template>
                确定删除该过程记录吗？
              </n-popconfirm>
            </n-space>
          </div>
        </n-collapse-item>
      </n-collapse>

      <n-button
        v-if="!showProgressForm"
        type="primary"
        style="margin-top: 12px"
        @click="openProgressForm"
      >
        添加过程
      </n-button>
    </template>

    <n-card
      v-if="!isNew && showProgressForm"
      size="small"
      :title="editingLogId != null ? '编辑过程' : '添加过程'"
      style="margin-top: 12px"
    >
      <n-form label-placement="top">
        <n-form-item label="标题" required>
          <n-input
            v-model:value="progressTitle"
            placeholder="过程标题"
            @keydown="onProgressTitleKeydown"
          />
        </n-form-item>
        <n-form-item label="状态">
          <n-radio-group v-model:value="progressStatus" data-field="progressStatus">
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
        <n-form-item label="详细内容">
          <n-input
            v-model:value="progressContent"
            type="textarea"
            :rows="3"
            placeholder="可选，展开后可见"
            @keydown="onProgressContentKeydown"
          />
        </n-form-item>
        <n-form-item label="图片">
          <AttachmentGallery
            ref="progressGalleryRef"
            owner-type="progress_log"
            :owner-id="editingLogId ?? undefined"
          />
        </n-form-item>
      </n-form>
      <n-space>
        <n-button type="primary" :keyboard="false" @click="saveProgress">
          {{ editingLogId != null ? "保存修改" : "保存过程" }}
        </n-button>
        <n-button @click="clearProgressForm">取消</n-button>
      </n-space>
    </n-card>
    </div>

    <template #footer>
      <n-space justify="end">
        <n-button type="primary" :loading="saving" @click="save">保存 (Ctrl+S)</n-button>
        <n-popconfirm v-if="!isNew" @positive-click="confirmDelete">
          <template #trigger>
            <n-button type="error">删除</n-button>
          </template>
          确定删除该代办事项吗？
        </n-popconfirm>
      </n-space>
    </template>
  </n-modal>
</template>
