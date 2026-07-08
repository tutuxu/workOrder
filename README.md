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
| `workOrder_1.1.2_x64-setup.exe` | NSIS 安装程序，推荐；会自动处理 WebView2 |
| `workOrder_1.1.2_x64_en-US.msi` | MSI 安装包 |

安装后从开始菜单或桌面快捷方式启动。数据默认保存在安装目录旁的 `data/` 下。

---

## 版本升级指南（最终用户）

当前版本**不会自动更新**，需要手动下载新版本并覆盖安装。从 **v1.0 升级到 1.1** 的详细说明见 **[docs/upgrade/v1.0-to-1.1.md](docs/upgrade/v1.0-to-1.1.md)**。

升级后首次启动时，程序会自动迁移数据库并生成 `status_config.json`，**无需手动改数据库**。

### 从 v1.0 快速升级（摘要）

1. （推荐）在 v1.0 **设置 → 备份** 导出 ZIP，或复制 `data/` 文件夹
2. **完全退出** workOrder
3. 用 v1.1 安装包覆盖安装，或仅用新 `workOrder.exe` 替换旧 exe（**保留 `data/`**）
4. 启动 v1.1 — 自动完成迁移

可选：便携包中双击 **`迁移数据.bat`**，在不打开主界面的情况下先迁移（与启动时逻辑相同）。

### 升级前

1. **完全退出** workOrder（确认任务管理器中无 `workOrder.exe` 进程）
2. **备份数据**（推荐）：复制 `workorder.db` 到安全位置

数据文件常见位置：

| 使用方式 | 默认路径 |
|----------|----------|
| 便携版 | `portable/data/workorder.db` |
| 安装版 | `{安装目录}/data/workorder.db` |
| 应用内设置过数据目录 | 设置页显示的路径 |
| 环境变量 | `WORKORDER_DATA_DIR` 指向的目录 |

### 安装版升级（setup.exe / msi）

适用于当初通过 `installer` 目录中的安装程序安装的用户。

1. 从 `workOrder-release` 分支或 Release 页面下载新版 `installer/`
2. 运行新版 **`workOrder_x.x.x_x64-setup.exe`**（推荐）或 MSI
3. 安装到**原安装路径**（安装程序会识别旧版并覆盖升级）
4. 从开始菜单或桌面快捷方式启动

升级后以下内容会保留：

| 内容 | 是否保留 |
|------|----------|
| `data/workorder.db`（代办与处置过程） | 保留 |
| `data/status_config.json`（v1.1 新增，首次启动自动生成） | 保留或新建 |
| `data/attachments/`（图片附件） | 保留 |
| `settings.json`（若存在，记录自定义数据目录） | 保留 |
| 程序 exe | 替换为新版本 |

### 便携版升级

适用于解压 `portable` 文件夹直接运行的用户。

1. 退出程序
2. 用新版 **`workOrder.exe`** 覆盖旧文件
3. **保留**同目录下的 `data/` 文件夹和 `settings.json`（如有）
4. 双击 `workOrder.exe` 或 `启动 workOrder.bat` 启动

```
portable/
├── workOrder.exe          ← 仅替换此文件
├── workOrder-migrate.exe  ← v1.1 新增（可选）
├── 启动 workOrder.bat
├── 迁移数据.bat           ← v1.1 新增（可选）
├── settings.json          ← 保留（若存在）
└── data/                  ← 保留
    ├── workorder.db
    ├── attachments/       ← 若有图片附件
    └── status_config.json ← v1.1 首次启动后生成
```

### 从安装版切换到便携版（可选）

1. 在旧版中打开「设置」，确认当前数据目录
2. 将 `data/` 文件夹（含 `workorder.db`）复制到新解压的 `portable/data/`
3. 若安装目录旁有 `settings.json`，一并复制到 `portable/` 目录
4. 用便携版启动并核对数据是否完整

### 升级后验证

- 打开若干旧代办，确认列表与详情正常
- 检查「处置过程」是否完整（标题、状态、展开后的详细内容）
- 若异常，关闭程序后用备份的 `workorder.db` 还原

### 常见问题

**Q：升级会丢失数据吗？**  
A：正常覆盖安装不会。只要 `data/workorder.db` 未被删除，数据会保留；首次启动会自动执行数据库迁移。

**Q：需要卸载旧版再装新版吗？**  
A：不需要。直接运行新版安装程序覆盖即可。

**Q：可以回退到旧版本吗？**  
A：可以。保留升级前的 `workorder.db` 备份；若新版有问题，装回旧版 exe 后通常仍可打开数据库。

**Q：安装到 Program Files 后设置页改不了数据目录？**  
A：该目录可能无写入权限。升级前请在旧版「设置」中确认实际数据路径，或改用便携版并将 `data/` 放在有写权限的位置。

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
    ├── workOrder_1.1.2_x64-setup.exe
    └── workOrder_1.1.2_x64_en-US.msi
```

将 **`release/portable`** 文件夹压缩为 zip 即可分享给他人一键运行。

### 更新 `workOrder-release` 发布分支

`workOrder-release` 分支**仅包含可运行产物**（无源码），供他人直接克隆使用。本地打包完成后，可手动更新该分支：

> 中文文件名乱码预防与修复见 **[docs/release-playbook.md](docs/release-playbook.md)**。

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
| 安装版运行 | `{安装目录}/data/workorder.db` |
| 应用内设置 | 设置页修改后写入 exe 旁 `settings.json` |
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
