# DateCalendar workbuddy Skill

本 Skill 允许 workbuddy（或任何 AI 助手）通过 CLI 或 HTTP API 管理 DateCalendar 中的任务和日程。

## 安装

1. 确保 DateCalendar 已安装并运行
2. 确保 `datecalendar-cli` 在 PATH 中（或知道其路径）
3. 将本 Skill 目录添加到 workbuddy 的 Skill 搜索路径

## 使用

workbuddy 会自动读取 `skill.md`，无需手动操作。

示例对话：
- 用户：「帮我把明天下午 3 点的会议加到 DateCalendar」
- workbuddy：（读取 Skill → 调用 CLI → 返回确认）

## 开发

- `skill.md`：Skill 主文件，描述所有可用的操作
- `examples/`：各种场景的完整示例

## 测试

```powershell
# 确保 DateCalendar 正在运行
# 测试 CLI
datecalendar-cli task list

# 测试 HTTP API
Invoke-RestMethod -Uri "http://127.0.0.1:9876/api/tasks" -Method Get
```
