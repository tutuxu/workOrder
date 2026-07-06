@echo off
setlocal EnableExtensions

set "JAVA_HOME=C:\Program Files\jdk-17.0.1"
set "PATH=%JAVA_HOME%\bin;D:\apache-maven-3.6.3\bin;%PATH%"

cd /d "%~dp0.."

set "PROJECT_DIR=%CD%"
set "JAR=%PROJECT_DIR%\target\workorder-1.0.0-SNAPSHOT.jar"
set "INPUT=C:\wo\jpkg-input"
set "DIST=C:\wo\release"
set "TEMP_DIR=C:\wo\jpkg-temp"
set "LOCAL_RELEASE=%PROJECT_DIR%\release"

if exist "%INPUT%" rmdir /s /q "%INPUT%"
if exist "%DIST%" rmdir /s /q "%DIST%"
if exist "%TEMP_DIR%" rmdir /s /q "%TEMP_DIR%"
if exist "%LOCAL_RELEASE%" rmdir /s /q "%LOCAL_RELEASE%"

echo Building workOrder...
call mvn -q package -DskipTests -Pproduction
if errorlevel 1 (
    echo Maven build failed.
    exit /b 1
)

if not exist "%JAR%" (
    echo JAR not found: %JAR%
    exit /b 1
)

echo Preparing jpackage input (app JAR + JavaFX natives)...
mkdir "%INPUT%"
call mvn -q dependency:copy-dependencies -DoutputDirectory="%INPUT%" -DincludeScope=runtime
if errorlevel 1 (
    echo Failed to copy runtime dependencies.
    exit /b 1
)
copy /y "%JAR%" "%INPUT%\" >nul
if errorlevel 1 (
    echo Failed to copy JAR into jpackage input directory.
    exit /b 1
)

mkdir "%TEMP_DIR%" >nul 2>&1

echo Packaging with jpackage...
jpackage ^
  --input "%INPUT%" ^
  --name workOrder ^
  --main-jar workorder-1.0.0-SNAPSHOT.jar ^
  --main-class com.workorder.launcher.DesktopLauncher ^
  --type app-image ^
  --dest "%DIST%" ^
  --temp "%TEMP_DIR%" ^
  --java-options "-Dfile.encoding=UTF-8" ^
  --java-options "-Dworkorder.data.dir=./data" ^
  --java-options "-Dserver.port=8081"

if errorlevel 1 (
    echo jpackage failed.
    exit /b 1
)

if not exist "%DIST%\workOrder\workOrder.exe" (
    echo Package output missing: %DIST%\workOrder\workOrder.exe
    exit /b 1
)

echo Copying release to project folder...
xcopy /E /I /Y "%DIST%\workOrder" "%LOCAL_RELEASE%\workOrder" >nul

echo.
echo Done.
echo Run: %LOCAL_RELEASE%\workOrder\workOrder.exe
echo Or:  %DIST%\workOrder\workOrder.exe
echo Data: %PROJECT_DIR%\data
echo URL:  http://localhost:8081
