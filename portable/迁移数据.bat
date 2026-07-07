@echo off
chcp 65001 >nul
cd /d "%~dp0"
echo workOrder v1.1 数据迁移工具
echo.
if not exist "data\workorder.db" (
  echo 未找到 data\workorder.db，请确认 data 目录位置。
  pause
  exit /b 1
)
if not exist "workOrder-migrate.exe" (
  echo 缺少 workOrder-migrate.exe，请使用完整 portable 包。
  pause
  exit /b 1
)
workOrder-migrate.exe --data-dir "%~dp0data"
echo.
pause
