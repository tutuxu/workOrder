/** 毛玻璃风格预设色（rgba），用于状态列表行背景。 */
export const GLASS_COLOR_PRESETS: { label: string; value: string }[] = [
  { label: "雾灰", value: "rgba(203, 213, 225, 0.55)" },
  { label: "天蓝", value: "rgba(147, 197, 253, 0.50)" },
  { label: "薄荷", value: "rgba(134, 239, 172, 0.45)" },
  { label: "琥珀", value: "rgba(252, 211, 77, 0.45)" },
  { label: "玫瑰", value: "rgba(251, 182, 206, 0.45)" },
  { label: "薰衣草", value: "rgba(196, 181, 253, 0.45)" },
  { label: "青绿", value: "rgba(94, 234, 212, 0.40)" },
  { label: "珍珠", value: "rgba(255, 255, 255, 0.65)" },
];

export const DEFAULT_STATUS_COLOR = GLASS_COLOR_PRESETS[0].value;

/** 将颜色选择器 / 配置中的各种格式统一为 CSS 颜色字符串。 */
export function normalizeStatusColor(value: unknown): string {
  if (value == null) return DEFAULT_STATUS_COLOR;
  if (typeof value === "string") {
    const trimmed = value.trim();
    return trimmed || DEFAULT_STATUS_COLOR;
  }
  if (Array.isArray(value)) {
    const [r, g, b, a = 1] = value.map(Number);
    if ([r, g, b].some((n) => Number.isNaN(n))) return DEFAULT_STATUS_COLOR;
    const alpha = Number.isNaN(Number(a)) ? 1 : Number(a);
    return `rgba(${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)}, ${alpha})`;
  }
  return DEFAULT_STATUS_COLOR;
}

export function statusColorFromConfig(
  config: { statuses: { id: string; color?: unknown }[] } | null,
  statusId: string,
): string {
  if (!config) return DEFAULT_STATUS_COLOR;
  const found = config.statuses.find((s) => s.id === statusId);
  return normalizeStatusColor(found?.color);
}

/** 将 rgba / hex 转为可用于 CSS 边框的实色（取 alpha 混合后的近似值）。 */
export function glassBorderColor(background: string): string {
  const rgba = parseRgba(background);
  if (rgba) {
    const [r, g, b, a] = rgba;
    const bg = [245, 245, 245];
    const blend = (c: number, i: number) =>
      Math.round(c * a + bg[i] * (1 - a));
    return `rgb(${blend(r, 0)}, ${blend(g, 1)}, ${blend(b, 2)})`;
  }
  return background;
}

function parseRgba(value: string): [number, number, number, number] | null {
  const match = value
    .trim()
    .match(/^rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)(?:\s*,\s*([\d.]+))?\s*\)$/i);
  if (!match) return null;
  return [
    Number(match[1]),
    Number(match[2]),
    Number(match[3]),
    match[4] !== undefined ? Number(match[4]) : 1,
  ];
}

export function rowStyleForStatus(
  color: string,
  overdue: boolean,
): Record<string, string> {
  if (overdue) {
    return {
      backgroundColor: "#fff1f0",
      borderColor: "#ffccc7",
    };
  }
  return {
    backgroundColor: color,
    borderColor: glassBorderColor(color),
    backdropFilter: "blur(8px)",
    WebkitBackdropFilter: "blur(8px)",
  };
}

/** 处置过程等场景下的状态标签样式（保持 tag 形态，按状态色着色）。 */
export function tagStyleForStatus(color: string): Record<string, string> {
  return {
    backgroundColor: color,
    color: "#333639",
    backdropFilter: "blur(6px)",
    WebkitBackdropFilter: "blur(6px)",
  };
}

export function tagColorFromConfig(
  config: { tags: { id: string; color?: unknown }[] } | null,
  tagId: string,
): string {
  if (!config) return DEFAULT_STATUS_COLOR;
  const found = config.tags.find((t) => t.id === tagId);
  return normalizeStatusColor(found?.color);
}

export function tagStyleForTag(color: string): Record<string, string> {
  return tagStyleForStatus(color);
}
