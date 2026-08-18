/** 将后端错误转成用户可读文案。 */
export function formatServiceError(error: unknown): string {
  const text = String(error);
  if (text.includes("Work order is in recycle bin")) {
    return "该事项在回收站，无法修改";
  }
  return text;
}
