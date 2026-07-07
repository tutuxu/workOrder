import dayjs from "dayjs";
import utc from "dayjs/plugin/utc";

dayjs.extend(utc);

/** 后端以 UTC 存储的无时区时间字符串，按本地时区展示。 */
export function formatServerDateTime(value?: string | null): string {
  if (!value) return "";
  return dayjs.utc(value).local().format("YYYY-MM-DD HH:mm");
}

/** 用户输入的本地时间（如计划完成时间），直接按本地展示。 */
export function formatLocalDateTime(value?: string | null): string {
  if (!value) return "";
  return dayjs(value).format("YYYY-MM-DD HH:mm");
}
