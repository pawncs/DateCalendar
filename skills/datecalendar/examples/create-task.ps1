#!/usr/bin/env pwsh
# 创建一个高优先级任务（PowerShell 版本）

datecalendar-cli task create `
  --title "完成 Q2 报告" `
  --priority 3 `
  --description "需要在 6 月 30 日前完成"
