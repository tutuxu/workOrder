<script setup lang="ts">
import { computed } from "vue";
import { GLASS_COLOR_PRESETS, normalizeStatusColor } from "../utils/statusColors";

const props = defineProps<{
  value: string;
}>();

const emit = defineEmits<{
  "update:value": [value: string];
}>();

const swatches = GLASS_COLOR_PRESETS.map((p) => p.value);

const isPresetSelected = computed(() =>
  swatches.some((c) => c === props.value),
);

function selectPreset(color: string) {
  emit("update:value", color);
}

function onPickerUpdate(value: unknown) {
  if (value == null) return;
  emit("update:value", normalizeStatusColor(value));
}
</script>

<template>
  <div class="status-color-picker">
    <n-text depth="3" style="display: block; font-size: 12px; margin-bottom: 6px">
      预设（毛玻璃）
    </n-text>
    <div class="preset-grid">
      <button
        v-for="preset in GLASS_COLOR_PRESETS"
        :key="preset.value"
        type="button"
        class="preset-swatch"
        :class="{ selected: value === preset.value }"
        :title="preset.label"
        :style="{ backgroundColor: preset.value }"
        @click="selectPreset(preset.value)"
      />
    </div>

    <n-text depth="3" style="display: block; font-size: 12px; margin: 10px 0 6px">
      自定义
    </n-text>
    <n-color-picker
      :value="value"
      :modes="['hex']"
      :swatches="swatches"
      :show-alpha="true"
      size="small"
      @update:value="onPickerUpdate"
    />
    <n-text
      v-if="!isPresetSelected"
      depth="3"
      style="display: block; font-size: 11px; margin-top: 4px"
    >
      当前为自定义颜色
    </n-text>
  </div>
</template>

<style scoped>
.status-color-picker {
  width: 100%;
}

.preset-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.preset-swatch {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.08);
  transition: transform 0.15s, border-color 0.15s;
}

.preset-swatch:hover {
  transform: scale(1.08);
}

.preset-swatch.selected {
  border-color: var(--n-primary-color);
  box-shadow: 0 0 0 1px var(--n-primary-color);
}
</style>
