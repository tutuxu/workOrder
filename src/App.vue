<script setup lang="ts">
import { ref } from "vue";
import WorkOrderList from "./views/WorkOrderList.vue";
import WorkOrderDetail from "./views/WorkOrderDetail.vue";
import Settings from "./views/Settings.vue";
import type { WorkOrder } from "./types";

const detailVisible = ref(false);
const selectedOrder = ref<WorkOrder | null>(null);
const listKey = ref(0);
const settingsVisible = ref(false);

function openDetail(order: WorkOrder | null) {
  selectedOrder.value = order;
  detailVisible.value = true;
}

function onSaved() {
  listKey.value += 1;
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
}
</script>

<template>
  <n-config-provider>
    <n-dialog-provider>
      <n-message-provider>
        <WorkOrderList :key="listKey" @open-detail="openDetail" @open-settings="openSettings" />
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
