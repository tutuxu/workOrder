<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useDialog, useMessage } from "naive-ui";
import * as settingsApi from "../api/settings";
import StatusConfigPanel from "../components/StatusConfigPanel.vue";
const emit = defineEmits<{
  closed: [];
}>();

const message = useMessage();
const dialog = useDialog();
const show = ref(true);
const loading = ref(false);
const applying = ref(false);
const exporting = ref(false);
const restoring = ref(false);

const currentDataDir = ref("");
const pendingDataDir = ref("");
const envOverride = ref(false);
const settingsPath = ref("");

const hasChange = computed(
  () =>
    pendingDataDir.value.trim() !== "" &&
    pendingDataDir.value !== currentDataDir.value,
);
const canEdit = computed(() => !envOverride.value);

onMounted(async () => {
  loading.value = true;
  try {
    const info = await settingsApi.getSettings();
    currentDataDir.value = info.dataDir;
    pendingDataDir.value = info.dataDir;
    envOverride.value = info.envOverride;
    settingsPath.value = info.settingsPath;
  } catch (error) {
    message.error(`加载设置失败：${error}`);
  } finally {
    loading.value = false;
  }
});

async function browse() {
  try {
    const picked = await settingsApi.pickDataDir();
    if (picked) {
      pendingDataDir.value = picked;
    }
  } catch (error) {
    message.error(`选择目录失败：${error}`);
  }
}

async function applyAndRestart() {
  if (!hasChange.value || !canEdit.value) return;
  applying.value = true;
  try {
    const result = await settingsApi.changeDataDir(pendingDataDir.value);
    if (result.restartRequired) {
      dialog.warning({
        title: "迁移完成",
        content: "数据已复制到新位置。需要重启应用后生效，是否立即重启？",
        positiveText: "立即重启",
        negativeText: "稍后",
        onPositiveClick: () => {
          void settingsApi.restartApp();
        },
      });
    }
  } catch (error) {
    message.error(`迁移失败：${error}`);
  } finally {
    applying.value = false;
  }
}

async function backup() {
  exporting.value = true;
  try {
    const savePath = await settingsApi.pickBackupSavePath();
    if (!savePath) return;
    const result = await settingsApi.exportBackup(savePath);
    message.success(`备份已保存：${result.filePath}`);
  } catch (error) {
    message.error(`备份失败：${error}`);
  } finally {
    exporting.value = false;
  }
}

function restore() {
  dialog.error({
    title: "确认恢复",
    content:
      "将用备份完全替换当前所有数据（工单、进度记录、附件），此操作不可撤销。建议先备份当前数据。是否继续？",
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
    const zipPath = await settingsApi.pickBackupFile();
    if (!zipPath) return;
    const result = await settingsApi.importBackup(zipPath);
    if (result.restartRequired) {
      dialog.warning({
        title: "恢复准备完成",
        content: "备份已校验通过。需要重启应用后完成恢复，是否立即重启？",
        positiveText: "立即重启",
        negativeText: "稍后",
        onPositiveClick: () => {
          void settingsApi.restartApp();
        },
      });
    }
  } catch (error) {
    message.error(`恢复失败：${error}`);
  } finally {
    restoring.value = false;
  }
}

function close() {
  emit("closed");
}
</script>

<template>
  <n-modal
    v-model:show="show"
    preset="card"
    title="设置"
    class="settings-modal"
    style="width: 720px"
    content-scrollable
    @after-leave="close"
  >
    <n-spin :show="loading">
      <n-form label-placement="top">
        <n-form-item label="数据存储位置">
          <n-input-group>
            <n-input :value="pendingDataDir" readonly placeholder="未设置" />
            <n-button :disabled="!canEdit" @click="browse">选择目录</n-button>
          </n-input-group>
        </n-form-item>
        <n-alert v-if="envOverride" type="warning" style="margin-bottom: 12px">
          当前由环境变量 WORKORDER_DATA_DIR 指定，无法在应用内修改。
        </n-alert>
        <n-text depth="3" style="display: block; margin-bottom: 12px">
          更改后将复制数据到新位置，并需要重启应用。原数据会保留。
        </n-text>
        <n-text v-if="settingsPath" depth="3" style="display: block; font-size: 12px; margin-bottom: 16px">
          配置文件：{{ settingsPath }}
        </n-text>

        <n-divider />

        <n-form-item label="代办状态">
          <StatusConfigPanel />
        </n-form-item>

        <n-divider />

        <n-form-item label="数据备份">
          <n-space vertical>
            <n-text depth="3" style="font-size: 13px">
              备份包含全部工单、进度记录和附件。恢复将替换当前所有数据，完成后需重启应用。
            </n-text>
            <n-space>
              <n-button :loading="exporting" @click="backup">备份...</n-button>
              <n-button type="error" :loading="restoring" @click="restore">
                恢复...
              </n-button>
            </n-space>
          </n-space>
        </n-form-item>
      </n-form>
    </n-spin>

    <template #footer>
      <n-space justify="end">
        <n-button @click="show = false">取消</n-button>
        <n-button
          type="primary"
          :disabled="!canEdit || !hasChange"
          :loading="applying"
          @click="applyAndRestart"
        >
          应用并重启
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>
