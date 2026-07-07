<script setup lang="ts">
import { ref } from "vue";
import { dateZhCN, zhCN } from "naive-ui";
import { isTauri } from "./tauri";
import WorkOrderList from "./views/WorkOrderList.vue";
import WorkOrderDetail from "./views/WorkOrderDetail.vue";
import Settings from "./views/Settings.vue";
import type { WorkOrder } from "./types";

const detailVisible = ref(false);
const selectedOrder = ref<WorkOrder | null>(null);
const listRef = ref<InstanceType<typeof WorkOrderList> | null>(null);
const settingsVisible = ref(false);

function openDetail(order: WorkOrder | null) {
  selectedOrder.value = order;
  detailVisible.value = true;
}

async function onSaved() {
  await listRef.value?.reload();
}

function onClosed() {
  detailVisible.value = false;
  selectedOrder.value = null;
}

function openSettings() {
  settingsVisible.value = true;
}

function onSettingsClosed() {
  settingsVisible.value = false;
  void listRef.value?.reload();
}
</script>

<template>
  <n-config-provider :locale="zhCN" :date-locale="dateZhCN">
    <n-dialog-provider>
      <n-message-provider>
        <n-alert
          v-if="!isTauri()"
          type="warning"
          title="请在 Tauri 桌面窗口中运行"
          style="margin: 16px"
        >
          当前页面在普通浏览器中打开，无法调用 Rust 后端。请关闭此标签页，在项目根目录执行
          <code>npm run tauri dev</code>，并使用弹出的 workOrder 桌面窗口（测试环境端口
          1420，勿与正式预览端口 6842 混用）。
        </n-alert>
        <WorkOrderList
          v-else
          ref="listRef"
          @open-detail="openDetail"
          @open-settings="openSettings"
        />
        <WorkOrderDetail
          v-if="detailVisible"
          :work-order="selectedOrder"
          @saved="onSaved"
          @closed="onClosed"
        />
        <Settings v-if="settingsVisible" @closed="onSettingsClosed" />
      </n-message-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>
