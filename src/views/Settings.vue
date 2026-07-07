<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useDialog, useMessage } from "naive-ui";
import * as settingsApi from "../api/settings";

const emit = defineEmits<{
  closed: [];
}>();

const message = useMessage();
const dialog = useDialog();
const show = ref(true);
const loading = ref(false);
const applying = ref(false);

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

function close() {
  emit("closed");
}
</script>

<template>
  <n-modal
    v-model:show="show"
    preset="card"
    title="设置"
    style="width: 560px"
    @after-leave="close"
  >
    <n-spin :show="loading">
      <n-form label-placement="top">
        <n-form-item label="数据存储位置">
          <n-input :value="pendingDataDir" readonly placeholder="未设置" />
        </n-form-item>
        <n-alert v-if="envOverride" type="warning" style="margin-bottom: 12px">
          当前由环境变量 WORKORDER_DATA_DIR 指定，无法在应用内修改。
        </n-alert>
        <n-text depth="3" style="display: block; margin-bottom: 12px">
          更改后将复制数据到新位置，并需要重启应用。原数据会保留。
        </n-text>
        <n-text v-if="settingsPath" depth="3" style="display: block; font-size: 12px">
          配置文件：{{ settingsPath }}
        </n-text>
      </n-form>
    </n-spin>

    <template #footer>
      <n-space justify="end">
        <n-button @click="show = false">取消</n-button>
        <n-button :disabled="!canEdit" @click="browse">浏览...</n-button>
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
