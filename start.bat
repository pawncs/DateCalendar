@echo off
chcp 65001 >nul
echo ========================================
echo   DateCalendar 启动脚本
echo   同时启动桌面应用 + 浏览器开发服务器
echo ========================================
echo.

cd /d "%~dp0datecalendar"

echo [1/3] 安装依赖（如需要）...
if not exist "node_modules" (
    echo   正在安装 npm 依赖...
    call npm install
) else (
    echo   node_modules 已存在，跳过
)

echo.
echo [2/3] 启动 Tauri 桌面应用（含 HTTP API :9876）...
echo   首次启动需要编译 Rust，请耐心等待...
start "DateCalendar-Tauri" cmd /c "npx tauri dev"
echo   Tauri 正在后台启动...

echo.
echo [3/3] 启动浏览器开发服务器（:5173）...
echo   等待 Tauri HTTP API 就绪（最多 30 秒）...
timeout /t 3 /nobreak >nul

:check_api
curl.exe -s http://localhost:9876/api/health >nul 2>&1
if %errorlevel% equ 0 (
    echo   HTTP API 已就绪！
    start http://localhost:5173
    npx vite --open
) else (
    timeout /t 2 /nobreak >nul
    goto check_api
)

pause
