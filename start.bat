@echo off
chcp 65001 >nul
echo ========================================
echo   DateCalendar 启动脚本
echo   启动 Tauri 桌面应用 + 打开浏览器
echo   日志输出到: datecalendar\startup.log
echo ========================================
echo.

cd /d "%~dp0datecalendar"

:: === 清空旧日志，重新记录 ===
echo. > startup.log
echo [%date% %time%] DateCalendar 启动日志 >> startup.log
echo. >> startup.log

echo [1/2] 安装依赖（如需要）...
if not exist "node_modules" (
    echo   正在安装 npm 依赖...
    call npm install >> startup.log 2>&1
) else (
    echo   node_modules 已存在，跳过
)

echo.
echo [2/2] 启动 Tauri 开发模式（桌面应用 + 浏览器）...
echo   首次启动需要编译 Rust，请耐心等待...
echo   所有输出将记录到 startup.log
echo.
echo   Tauri 启动后：
echo     - 桌面窗口会自动打开
echo     - 悬浮窗 会出现在屏幕右侧
echo     - 等待 HTTP API :9876 就绪后，浏览器会自动打开
echo.

start "DateCalendar-Tauri" cmd /c "npx tauri dev >> startup.log 2>&1"

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
echo     悬浮窗  ：Ctrl+Shift+D 切换显隐
echo     浏览器  ：http://localhost:5173
echo     HTTP API：http://localhost:9876
echo     日志文件：datecalendar\startup.log
echo   ========================================
pause
