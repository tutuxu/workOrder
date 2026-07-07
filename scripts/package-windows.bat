@echo off
setlocal EnableExtensions

cd /d "%~dp0.."

set "LOCAL_RELEASE=%CD%\release"
set "PORTABLE_DIR=%LOCAL_RELEASE%\portable"
set "CARGO_BIN=%USERPROFILE%\.cargo\bin"

if exist "%CARGO_BIN%\cargo.exe" (
    set "PATH=%CARGO_BIN%;%PATH%"
)

echo ========================================
echo   workOrder Windows Package
echo ========================================
echo.

echo [1/3] Building frontend and Tauri app...
call npm run tauri build
if errorlevel 1 (
    echo.
    echo Build failed. Ensure Node.js, Rust, and VS Build Tools are installed.
    exit /b 1
)

echo Building migration tool...
pushd src-tauri
call cargo build --release --bin workorder-migrate
if errorlevel 1 (
    popd
    echo Migration tool build failed.
    exit /b 1
)
popd

echo.
echo [2/3] Creating portable one-click package...
if exist "%LOCAL_RELEASE%" rmdir /s /q "%LOCAL_RELEASE%"
mkdir "%LOCAL_RELEASE%" >nul 2>&1
mkdir "%PORTABLE_DIR%" >nul 2>&1
mkdir "%PORTABLE_DIR%\data" >nul 2>&1

if not exist "src-tauri\target\release\workorder.exe" (
    echo Missing workorder.exe after build.
    exit /b 1
)

copy /Y "src-tauri\target\release\workorder.exe" "%PORTABLE_DIR%\workOrder.exe" >nul

if exist "src-tauri\target\release\workorder-migrate.exe" (
    copy /Y "src-tauri\target\release\workorder-migrate.exe" "%PORTABLE_DIR%\workOrder-migrate.exe" >nul
)

(
    echo @echo off
    echo cd /d "%%~dp0"
    echo start "" "workOrder.exe"
) > "%PORTABLE_DIR%\启动 workOrder.bat"

(
    echo @echo off
    echo chcp 65001 ^>nul
    echo cd /d "%%~dp0"
    echo echo workOrder v1.0 数据迁移工具
    echo echo.
    echo if not exist "data\workorder.db" ^(
    echo   echo 未找到 data\workorder.db，请确认 data 目录位置。
    echo   pause
    echo   exit /b 1
    echo ^)
    echo if not exist "workOrder-migrate.exe" ^(
    echo   echo 缺少 workOrder-migrate.exe，请使用完整 portable 包。
    echo   pause
    echo   exit /b 1
    echo ^)
    echo workOrder-migrate.exe --data-dir "%%~dp0data"
    echo echo.
    echo pause
) > "%PORTABLE_DIR%\迁移数据.bat"

echo.
echo [3/3] Copying installers...
if exist "src-tauri\target\release\bundle\nsis" (
    xcopy /E /I /Y "src-tauri\target\release\bundle\nsis\*" "%LOCAL_RELEASE%\installer\" >nul
)
if exist "src-tauri\target\release\bundle\msi" (
    xcopy /E /I /Y "src-tauri\target\release\bundle\msi\*" "%LOCAL_RELEASE%\installer\" >nul
)

echo.
echo ========================================
echo   Package complete
echo ========================================
echo.
echo Portable (double-click to run):
echo   %PORTABLE_DIR%\workOrder.exe
echo   %PORTABLE_DIR%\启动 workOrder.bat
echo.
echo Data directory (created on first run):
echo   %PORTABLE_DIR%\data\
echo.
if exist "%LOCAL_RELEASE%\installer" (
    echo Installer:
    dir /b "%LOCAL_RELEASE%\installer"
    echo.
)
echo You can zip the "portable" folder for distribution.
