import type { ProgressLogSummary } from "../bindings";

export type CardProgressView = {
  items: ProgressLogSummary[];
  truncated: boolean;
};

export const CARD_PROGRESS_MAX = 6;

export function sortCardProgress(
  summaries: ProgressLogSummary[],
  orderedStatusIds: string[],
  maxVisible: number = CARD_PROGRESS_MAX,
): CardProgressView {
  if (!summaries.length) return { items: [], truncated: false };

  const statusIndex = new Map(orderedStatusIds.map((id, i) => [id, i]));
  const unknownRank = orderedStatusIds.length;

  const sorted = [...summaries].sort((a, b) => {
    const ai = statusIndex.get(a.status) ?? unknownRank;
    const bi = statusIndex.get(b.status) ?? unknownRank;
    if (ai !== bi) return ai - bi;
    if (ai === unknownRank && a.status !== b.status) {
      return a.status < b.status ? -1 : 1;
    }
    if (a.createdAt < b.createdAt) return 1;
    if (a.createdAt > b.createdAt) return -1;
    return 0;
  });

  return {
    items: sorted.slice(0, maxVisible),
    truncated: sorted.length > maxVisible,
  };
}
