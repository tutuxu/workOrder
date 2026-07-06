@echo off
setlocal

set JAVA_HOME=C:\Program Files\jdk-17.0.1
set PATH=%JAVA_HOME%\bin;D:\apache-maven-3.6.3\bin;%PATH%

cd /d "%~dp0.."

echo Building workOrder...
call mvn -q clean package -DskipTests -Pproduction
if errorlevel 1 exit /b 1

set JAR=target\workorder-1.0.0-SNAPSHOT.jar
if not exist "%JAR%" (
    echo JAR not found: %JAR%
    exit /b 1
)

echo Packaging with jpackage...
jpackage ^
  --input target ^
  --name workOrder ^
  --main-jar workorder-1.0.0-SNAPSHOT.jar ^
  --main-class com.workorder.launcher.DesktopLauncher ^
  --type app-image ^
  --dest target/dist ^
  --java-options "-Dfile.encoding=UTF-8"

echo Done. Run target\dist\workOrder\workOrder.exe
