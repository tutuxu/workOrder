<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import dayjs from "dayjs";
import { useMessage } from "naive-ui";
import * as workOrderApi from "../api/workOrders";
import * as progressLogApi from "../api/progressLogs";
import {
  STATUS_OPTIONS,
  type ProgressLog,
  type WorkOrder,
  type WorkOrderInput,
  type WorkOrderStatus,
} from "../types";

const props = defineProps<{
  workOrder: WorkOrder | null;
}>();

const emit = defineEmits<{
  saved: [];
  closed: [];
}>();

const message = useMessage();
const show = ref(true);
const saving = ref(false);

const title = ref("");
const description = ref("");
const status = ref<WorkOrderStatus>("NOT_STARTED");
const dueDate = ref<number | null>(null);
const waitingFor = ref("");
const waitingReason = ref("");

const logs = ref<ProgressLog[]>([]);
const progressInput = ref("");
const editingLogId = ref<number | null>(null);

const workOrderId = ref<number | undefined>(props.workOrder?.id);
const isNew = computed(() => workOrderId.value == null);
const showWaitingFields = computed(() => status.value === "WAITING_REPLY");
const modalTitle = computed(() => (isNew.value ? "新建代办" : "编辑代办"));

function bindForm(order: WorkOrder) {
  title.value = order.title ?? "";
  description.value = order.description ?? "";
  status.value = order.status ?? "NOT_STARTED";
  dueDate.value = order.dueDate ? dayjs(order.dueDate).valueOf() : null;
  waitingFor.value = order.waitingFor ?? "";
  waitingReason.value = order.waitingReason ?? "";
  workOrderId.value = order.id;
}

async function loadLogs() {
  if (workOrderId.value == null) {
    logs.value = [];
    return;
  }
  logs.value = await progressLogApi.listProgressLogs(workOrderId.value);
}

onMounted(async () => {
  if (props.workOrder) {
    bindForm(props.workOrder);
    await loadLogs();
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
    waitingFor: waitingFor.value || null,
    waitingReason: waitingReason.value || null,
    dueDate: dueDate.value ? dayjs(dueDate.value).format("YYYY-MM-DDTHH:mm:ss") : null,
  };
}

async function save() {
  saving.value = true;
  try {
    const input = buildInput();
    if (isNew.value) {
      const created = await workOrderApi.createWorkOrder(input);
      workOrderId.value = created.id;
      await flushPendingProgress();
      await loadLogs();
      message.success("已保存");
      emit("saved");
    } else {
      await workOrderApi.updateWorkOrder(workOrderId.value!, input);
      await flushPendingProgress();
      await loadLogs();
      message.success("已保存");
      emit("saved");
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

function clearEditMode() {
  editingLogId.value = null;
  progressInput.value = "";
}

async function saveProgress() {
  if (workOrderId.value == null) {
    message.warning("请先保存代办事项");
    return;
  }
  try {
    if (editingLogId.value != null) {
      await progressLogApi.updateProgressLog(
        editingLogId.value,
        workOrderId.value,
        progressInput.value,
      );
      clearEditMode();
    } else {
      await progressLogApi.addProgressLog(workOrderId.value, progressInput.value);
      progressInput.value = "";
    }
    await loadLogs();
  } catch (e) {
    message.error(String(e));
  }
}

async function flushPendingProgress() {
  if (!progressInput.value.trim() || workOrderId.value == null) return;
  try {
    if (editingLogId.value != null) {
      await progressLogApi.updateProgressLog(
        editingLogId.value,
        workOrderId.value,
        progressInput.value,
      );
      clearEditMode();
    } else {
      await progressLogApi.addProgressLog(workOrderId.value, progressInput.value);
      progressInput.value = "";
    }
  } catch (e) {
    message.error(String(e));
  }
}

function startEdit(log: ProgressLog) {
  editingLogId.value = log.id ?? null;
  progressInput.value = log.content;
}

async function deleteProgress(log: ProgressLog) {
  if (workOrderId.value == null || log.id == null) return;
  try {
    await progressLogApi.deleteProgressLog(log.id, workOrderId.value);
    if (editingLogId.value === log.id) {
      clearEditMode();
    }
    await loadLogs();
  } catch (e) {
    message.error(String(e));
  }
}

function formatDate(value: string) {
  return dayjs(value).format("YYYY-MM-DD HH:mm");
}
</script>

<template>
  <n-modal
    v-model:show="show"
    preset="card"
    :title="modalTitle"
    style="width: 640px"
    @after-leave="close"
  >
    <n-form label-placement="top">
      <n-form-item label="标题" required>
        <n-input v-model:value="title" />
      </n-form-item>
      <n-form-item label="描述">
        <n-input v-model:value="description" type="textarea" :rows="4" />
      </n-form-item>
      <n-form-item label="状态">
        <n-radio-group v-model:value="status">
          <n-space>
            <n-radio
              v-for="opt in STATUS_OPTIONS"
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
      <template v-if="showWaitingFields">
        <n-form-item label="等待对象">
          <n-input v-model:value="waitingFor" />
        </n-form-item>
        <n-form-item label="等待原因">
          <n-input v-model:value="waitingReason" />
        </n-form-item>
      </template>
    </n-form>

    <h3>处置过程</h3>
    <div v-if="isNew" style="color: #999; margin-bottom: 12px">保存后可追加处置过程</div>
    <template v-else>
      <div v-if="logs.length === 0" style="color: #999; margin-bottom: 12px">暂无过程记录</div>
      <div v-for="log in logs" :key="log.id" class="timeline-entry">
        <span class="timeline-content">
          {{ formatDate(log.createdAt) }} — {{ log.content }}
        </span>
        <n-button text type="primary" @click="startEdit(log)">编辑</n-button>
        <n-popconfirm @positive-click="deleteProgress(log)">
          <template #trigger>
            <n-button text type="error">删除</n-button>
          </template>
          确定删除该过程记录吗？
        </n-popconfirm>
      </div>
    </template>

    <n-space style="margin-top: 12px" align="center">
      <n-input
        v-model:value="progressInput"
        placeholder="追加过程"
        style="flex: 1"
        @keyup.enter="saveProgress"
      />
      <n-button @click="saveProgress">
        {{ editingLogId != null ? "保存修改" : "追加" }}
      </n-button>
      <n-button v-if="editingLogId != null" @click="clearEditMode">取消</n-button>
    </n-space>

    <template #footer>
      <n-space justify="end">
        <n-button type="primary" :loading="saving" @click="save">保存</n-button>
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
