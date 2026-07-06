# workOrder

个人工作事项追踪桌面应用，数据保存在本地，无需联网。

## 快速开始（推荐：便携版）

1. 进入 **`portable`** 文件夹
2. 双击 **`workOrder.exe`**（或 **`启动 workOrder.bat`**）
3. 首次运行会在同目录下 `data/` 中自动创建 `workorder.db` 保存你的数据

```
portable/
├── workOrder.exe          # 主程序
├── 启动 workOrder.bat     # 启动脚本
└── data/                  # 数据目录（首次运行后生成 workorder.db）
```

可将整个 `portable` 文件夹复制到 U 盘或其他电脑，解压即用。

## 安装包方式（可选）

进入 **`installer`** 文件夹，任选其一安装：

| 文件 | 说明 |
|------|------|
| `workOrder_1.0.0_x64-setup.exe` | 推荐；会自动处理 WebView2 依赖 |
| `workOrder_1.0.0_x64_en-US.msi` | MSI 安装包 |

安装后从开始菜单或桌面快捷方式启动。

## 系统要求

- Windows 10 或更高版本
- 需要 WebView2 运行时（Win10+ 通常已预装；便携版无法启动时请改用安装包）

## 数据说明

- 便携版数据路径：`portable/data/workorder.db`
- 备份时复制整个 `data` 文件夹即可
- 自定义数据目录：设置环境变量 `WORKORDER_DATA_DIR` 指向目标文件夹

## 版本

当前版本：**1.0.0**
