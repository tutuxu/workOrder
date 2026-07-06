<script setup lang="ts">
import { ref } from "vue";
import WorkOrderList from "./views/WorkOrderList.vue";
import WorkOrderDetail from "./views/WorkOrderDetail.vue";
import type { WorkOrder } from "./types";

const detailVisible = ref(false);
const selectedOrder = ref<WorkOrder | null>(null);
const listKey = ref(0);

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
</script>

<template>
  <n-config-provider>
    <n-message-provider>
      <WorkOrderList :key="listKey" @open-detail="openDetail" />
      <WorkOrderDetail
        v-if="detailVisible"
        :work-order="selectedOrder"
        @saved="onSaved"
        @closed="onClosed"
      />
    </n-message-provider>
  </n-config-provider>
</template>
