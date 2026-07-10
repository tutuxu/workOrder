<script setup lang="ts">
import { useTagConfig } from "../composables/useTagConfig";
import { tagStyleForTag } from "../utils/statusColors";

const selectedTags = defineModel<string[]>("value", { default: () => [] });

const { tagOptions, tagColor } = useTagConfig();

function isSelected(tagId: string): boolean {
  return selectedTags.value.includes(tagId);
}

function toggleTag(tagId: string): void {
  if (isSelected(tagId)) {
    selectedTags.value = selectedTags.value.filter((id) => id !== tagId);
  } else {
    selectedTags.value = [...selectedTags.value, tagId];
  }
}

function tagStyle(tagId: string): Record<string, string> {
  const color = tagColor(tagId);
  if (isSelected(tagId)) {
    return {
      ...tagStyleForTag(color),
      cursor: "pointer",
      boxShadow: `0 0 0 2px ${color}66`,
    };
  }
  return {
    backgroundColor: "rgba(255, 255, 255, 0.6)",
    color: "#333639",
    border: `2px solid ${color}`,
    cursor: "pointer",
  };
}
</script>

<template>
  <n-space v-if="tagOptions.length" size="small" class="tag-picker">
    <n-tag
      v-for="opt in tagOptions"
      :key="opt.value"
      size="small"
      :bordered="false"
      :style="tagStyle(opt.value)"
      class="tag-picker-item"
      @click="toggleTag(opt.value)"
    >
      {{ opt.label }}
    </n-tag>
  </n-space>
  <span v-else class="tag-picker-empty">暂无标签，请先在设置中配置</span>
</template>

<style scoped>
.tag-picker {
  flex-wrap: wrap;
}

.tag-picker-item {
  transition: box-shadow 0.15s, background-color 0.15s;
  user-select: none;
}

.tag-picker-empty {
  color: #999;
  font-size: 13px;
}
</style>
