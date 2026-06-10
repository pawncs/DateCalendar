@echo off
chcp 65001 >nul
echo ========================================
echo   DateCalendar 启动脚本
echo   启动 Tauri 桌面应用 + 打开浏览器
echo ========================================
echo.

cd /d "%~dp0datecalendar"

echo [1/2] 安装依赖（如需要）...
if not exist "node_modules" (
    echo   正在安装 npm 依赖...
    call npm install
) else (
    echo   node_modules 已存在，跳过
)

echo.
echo [2/2] 启动 Tauri 开发模式（桌面应用 + 浏览器）...
echo   首次启动需要编译 Rust，请耐心等待...
echo.
echo   Tauri 启动后：
echo     - 桌面窗口会自动打开
echo     - 等待 HTTP API :9876 就绪后，浏览器会自动打开
echo.

start "DateCalendar-Tauri" cmd /c "npx tauri dev"

echo   等待 HTTP API 就绪...
timeout /t 5 /nobreak >nul

:check_api
curl.exe -s http://localhost:9876/api/health >nul 2>&1
if %errorlevel% equ 0 (
    echo   HTTP API 已就绪！打开浏览器...
    start http://localhost:5173
    goto done
)
timeout /t 2 /nobreak >nul
goto check_api

:done
echo.
echo   ========================================
echo   启动完成！
echo     桌面应用：Tauri 窗口
echo     浏览器  ：http://localhost:5173
echo     HTTP API：http://localhost:9876
echo   ========================================
pause
