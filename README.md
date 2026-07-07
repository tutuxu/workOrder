# workOrder

个人工作事项追踪桌面应用。基于 Tauri 2 + Vue 3 + Rust，数据保存在本地 SQLite。

## 获取可运行版本（最终用户）

若你**只需要运行程序、不需要源码**，请克隆发布分支：

```bash
git clone -b workOrder-release <仓库地址>
```

克隆后目录结构如下，可直接使用：

```
├── README.md
├── portable/          # 便携版，双击 workOrder.exe 运行
└── installer/         # 安装包（setup.exe / msi）
```

也可在 GitHub 上将分支切换为 `workOrder-release` 后下载 ZIP。

---

## 一键运行（最终用户）

若你拿到的是已打包的 **`portable`** 文件夹，无需安装 Node.js 或 Rust：

1. 解压整个 `portable` 文件夹到任意位置（路径尽量不含特殊字符）
2. 双击 **`workOrder.exe`**，或双击 **`启动 workOrder.bat`**
3. 首次运行会在同目录下自动创建 `data/workorder.db` 保存数据

### 目录说明

```
portable/
├── workOrder.exe          # 主程序，双击运行
├── 启动 workOrder.bat     # 启动脚本（效果同上）
└── data/                  # 数据目录（首次运行后生成 workorder.db）
```

### 系统要求

- Windows 10 或更高版本（需 WebView2 运行时，Win10+ 通常已预装）
- 若无法启动，可改用安装包（见下文）

### 安装包方式（可选）

若提供的是 `installer` 目录：

| 文件 | 说明 |
|------|------|
| `workOrder_1.0.0_x64-setup.exe` | NSIS 安装程序，推荐；会自动处理 WebView2 |
| `workOrder_1.0.0_x64_en-US.msi` | MSI 安装包 |

安装后从开始菜单或桌面快捷方式启动。数据默认保存在安装目录旁的 `data/` 下。

---

## 从源码打包（开发者）

### 环境要求

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/)（`rustup` 安装后重启终端）
- Windows：[Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)（勾选「使用 C++ 的桌面开发」）

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

开发时数据文件位于项目根目录 `data/workorder.db`。

修改 Rust Command 或 Model 后，运行 `npm run bindings` 重新生成 `src/bindings.ts`（debug 模式启动时也会自动导出）。更多命令见 [开发者常用命令](docs/dev-commands.md)。

### 打包为可分发版本

任选其一：

```bash
# 方式一：npm 脚本
npm run package:win

# 方式二：双击项目根目录的 打包.bat
```

打包完成后，产物位于 `release/`：

```
release/
├── portable/              # 便携版，可直接 zip 分发给他人
│   ├── workOrder.exe
│   ├── 启动 workOrder.bat
│   └── data/
└── installer/             # 安装包
    ├── workOrder_1.0.0_x64-setup.exe
    └── workOrder_1.0.0_x64_en-US.msi
```

将 **`release/portable`** 文件夹压缩为 zip 即可分享给他人一键运行。

### 更新 `workOrder-release` 发布分支

`workOrder-release` 分支**仅包含可运行产物**（无源码），供他人直接克隆使用。本地打包完成后，可手动更新该分支：

```bash
npm run package:win
git checkout workOrder-release
# 用 release/portable 与 release/installer 覆盖分支根目录对应文件夹
git add portable installer README.md
git commit -m "发布 workOrder x.x.x"
git checkout main
```

### 数据目录

| 场景 | 数据路径 |
|------|----------|
| 便携版运行 | `portable/data/workorder.db` |
| 开发模式 | 项目根 `data/workorder.db` |
| 自定义路径 | 设置环境变量 `WORKORDER_DATA_DIR` |

---

## 更多文档

- [开发者常用命令](docs/dev-commands.md)
- [后端架构](docs/backend.md)
- [Tauri Command API](docs/api/commands.md)
- [需求文档](plan/需求文档.md)
- [技术选型](plan/技术选型.md)
- [实现计划](plan/实现计划.md)
