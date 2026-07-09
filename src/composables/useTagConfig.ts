import { computed, ref } from "vue";
import { commands } from "../bindings";
import type { TagConfig } from "../bindings";
import { sortedTags, tagLabelFromConfig } from "../types";
import { tagColorFromConfig } from "../utils/statusColors";

const config = ref<TagConfig | null>(null);
let loadPromise: Promise<TagConfig> | null = null;

export function useTagConfig() {
  const tagOptions = computed(() => {
    if (!config.value) return [];
    return sortedTags(config.value).map((t) => ({
      value: t.id,
      label: t.label,
    }));
  });

  async function load(force = false): Promise<TagConfig> {
    if (!force && config.value) return config.value;
    if (!force && loadPromise) return loadPromise;
    loadPromise = commands.getTagConfig().then((loaded) => {
      config.value = loaded;
      return loaded;
    });
    return loadPromise;
  }

  async function save(next: TagConfig): Promise<void> {
    await commands.saveTagConfig(next);
    config.value = next;
  }

  function tagLabel(tagId: string): string {
    return tagLabelFromConfig(config.value, tagId);
  }

  function tagColor(tagId: string): string {
    return tagColorFromConfig(config.value, tagId);
  }

  return {
    config,
    tagOptions,
    load,
    save,
    tagLabel,
    tagColor,
  };
}
