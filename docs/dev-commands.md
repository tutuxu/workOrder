# 开发者常用命令

项目根目录下执行（除非注明需进入 `src-tauri`）。

---

## 环境与依赖

```bash
# 安装前端依赖（首次克隆或 package.json 变更后）
npm install
```

**环境要求：** Node.js 18+、Rust（rustup）、Windows 需 Visual Studio Build Tools（C++ 桌面开发）。

---

## 日常开发

```bash
# 启动 Tauri 开发模式（热重载，debug 时自动更新 src/bindings.ts）
npm run tauri dev

# 仅启动前端 Vite（浏览器调试，不启动 Rust 后端）
npm run dev

# 预览生产构建（正式环境本地预览）
npm run build && npm run preview
```

### 默认端口

| 环境 | 命令 | 默认端口 | 说明 |
|------|------|----------|------|
| 测试（Tauri dev） | `npm run tauri dev` | **1420** | Tauri 窗口加载此端口，由 `dev:tauri` 启动 |
| 测试（浏览器） | `npm run dev` | **5173** | 独立前端开发，避免占用 1420 |
| 正式（预览） | `npm run preview` | **6842** | 预览 `dist/` 构建产物 |

端口可在 `.env.tauri`、`.env.development`、`.env.production` 中通过 `VITE_DEV_PORT` / `VITE_PREVIEW_PORT` 覆盖；修改测试端口后须同步更新 `src-tauri/tauri.conf.json` 中的 `devUrl`。

开发数据文件：`data/workorder.db`（项目根目录）。

---

## 类型绑定（tauri-specta）

修改 Rust Command 或 Model 后，须重新生成 `src/bindings.ts`：

```bash
# 推荐：npm 脚本
npm run bindings

# 等价：直接调用 Rust 导出工具
cargo run --manifest-path src-tauri/Cargo.toml --bin export_bindings
```

其他方式：

```bash
# debug 模式启动时自动导出（无需单独执行 bindings）
npm run tauri dev

# 通过单元测试触发导出（部分 Windows 环境可能因 WebView 依赖失败）
cd src-tauri && cargo test export_bindings
```

> `src/bindings.ts` 为生成文件，**勿手动编辑**；生成后请一并提交 git。

---

## 前端检查与构建

```bash
# TypeScript 类型检查
npx vue-tsc --noEmit

# 生产构建（类型检查 + Vite 打包）
npm run build

# 预览构建产物
npm run preview
```

---

## Rust 后端

```bash
# 进入后端目录
cd src-tauri

# 运行全部单元测试
cargo test

# 生成并打开 Rust API 文档（cargo doc）
cargo doc --no-deps --open

# 仅编译检查
cargo build
```

---

## 打包分发

```bash
# Windows 打包（便携版 + 安装包）
npm run package:win

# 或双击项目根目录
打包.bat
```

产物目录：`release/portable/`、`release/installer/`。

---

## 命令速查

| 目的 | 命令 |
|------|------|
| 开发运行 | `npm run tauri dev` |
| 重新生成 TS 绑定 | `npm run bindings` |
| 前端类型检查 | `npx vue-tsc --noEmit` |
| Rust 测试 | `cd src-tauri && cargo test` |
| Rust API 文档 | `cd src-tauri && cargo doc --no-deps --open` |
| Windows 打包 | `npm run package:win` |

---

## 相关文档

- [后端架构](backend.md)
- [Tauri Command API](api/commands.md)
- [README 开发者章节](../README.md#从源码打包开发者)
