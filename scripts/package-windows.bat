@echo off
setlocal EnableExtensions

cd /d "%~dp0.."

set "LOCAL_RELEASE=%CD%\release"

echo Building workOrder with Tauri...
call npm run tauri build
if errorlevel 1 (
    echo Tauri build failed.
    exit /b 1
)

if exist "%LOCAL_RELEASE%" rmdir /s /q "%LOCAL_RELEASE%"
mkdir "%LOCAL_RELEASE%" >nul 2>&1

if exist "src-tauri\target\release\bundle\nsis" (
    xcopy /E /I /Y "src-tauri\target\release\bundle\nsis\*" "%LOCAL_RELEASE%\" >nul
) else if exist "src-tauri\target\release\bundle\msi" (
    xcopy /E /I /Y "src-tauri\target\release\bundle\msi\*" "%LOCAL_RELEASE%\" >nul
)

if exist "src-tauri\target\release\workorder.exe" (
    copy /Y "src-tauri\target\release\workorder.exe" "%LOCAL_RELEASE%\" >nul
)

echo.
echo Done.
echo Exe: src-tauri\target\release\workorder.exe
echo Release: %LOCAL_RELEASE%
echo Data: %CD%\data
