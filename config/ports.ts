/** 测试环境（npm run tauri dev）默认端口 */
export const TAURI_DEV_PORT = 1420;

/** 浏览器独立开发（npm run dev）默认端口，避免与 Tauri dev 冲突 */
export const WEB_DEV_PORT = 5173;

/** 正式环境预览（npm run preview）默认端口；避开常见默认端口（3000/4173/5173/8080 等） */
export const PREVIEW_PORT = 6842;

export function tauriDevUrl(port = TAURI_DEV_PORT): string {
  return `http://localhost:${port}`;
}
