# 测试流程：API 认证（暂不测试）

> 对应设计文档：[D-19 API 认证（保留入口，暂不实现）](../design/D-19-api-auth.md)
> 
> **当前状态**：API 认证仅保留设计入口，暂不实现。本测试计划仅作为未来实现时的参考。

## 前置条件（未来）

- DateCalendar Tauri 应用已启动（HTTP API 在 `localhost:9876` 运行）
- 数据库中存在 `settings` 表，且包含 `api_token` 记录
- `curl` 或 Postman 可用

---

## 当前测试重点

由于认证暂不实现，当前测试重点是：

1. **API 可访问性**：所有 API 端点无需认证即可访问
2. **CLI 功能**：CLI 直接访问数据库，无需认证
3. **API 文档**：OpenAPI 规范中包含安全方案定义（但未强制执行）

### 当前手动测试

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | `curl http://127.0.0.1:9876/api/tasks` | 返回 200，任务列表 JSON（无需认证） |
| 2 | `curl http://127.0.0.1:9876/api/health` | 返回 200，健康检查通过 |
| 3 | `datecalendar-cli task list` | 返回任务列表（CLI 无需认证） |

---

## 白盒测试（Rust 后端）— 未来实现时

```bash
cd datecalendar/src-tauri
cargo test --lib api::auth::tests -- --nocapture
```

### 覆盖用例（未来）

| # | 用例 | 验证点 |
|---|------|--------|
| 1 | `test_auth_missing_header` | 无 Authorization 头返回 401 |
| 2 | `test_auth_invalid_format` | Authorization 头格式错误返回 401 |
| 3 | `test_auth_wrong_token` | Token 错误返回 401 |
| 4 | `test_auth_valid_token` | 正确 Token 通过认证 |
| 5 | `test_auth_health_public` | `/api/health` 无需认证 |
| 6 | `test_auth_token_endpoint` | `/api/auth/token` 无需认证（需要 setup_secret） |
| 7 | `test_auth_disabled` | `api_auth_enabled=false` 时跳过认证 |
| 8 | `test_setup_secret_only_once` | setup_secret 二次使用失效 |

---

## 手动黑盒测试 — 未来实现时

### TC-01: 无认证访问 API（未来预期）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | `curl http://127.0.0.1:9876/api/tasks` | **未来**：返回 401 Unauthorized；**当前**：返回 200 |

### TC-02: 正确认证访问 API（未来预期）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 获取 token（通过 CLI 或 setup_secret） | 获得 token |
| 2 | `curl -H "Authorization: Bearer <TOKEN>" http://127.0.0.1:9876/api/tasks` | **未来**：返回 200；**当前**：无需 token 即可返回 200 |

### TC-03: 错误 Token（未来预期）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | `curl -H "Authorization: Bearer wrong_token" http://127.0.0.1:9876/api/tasks` | **未来**：返回 401；**当前**：返回 200 |

---

## HTTP 客户端测试（Postman/curl）— 未来实现时

### TC-07: Postman 测试（未来预期）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 在 Postman 中打开 `http://127.0.0.1:9876/api/tasks` | **未来**：返回 401；**当前**：返回 200 |
| 2 | 在 Authorization 选项卡选择 "Bearer Token"，输入 token | 保存 |
| 3 | 再次发送请求 | **未来**：返回 200；**当前**：无需 token 即可返回 200 |

### TC-08: Swagger UI 测试（未来预期）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 打开 `http://127.0.0.1:9876/docs` | Swagger UI 加载 |
| 2 | 点击 "Authorize" 按钮，输入 Bearer token | **未来**：成功；**当前**：无需认证 |
| 3 | 执行 `GET /api/tasks` | 返回 200 |

---

## workbuddy 集成验证 — 未来实现时

### TC-09: workbuddy 自动获取 Token（未来预期）

模拟 workbuddy 的行为：

```bash
# 1. workbuddy 首先检查是否已有 token（从配置文件或环境变量）
token="$DATECALENDAR_TOKEN"

if [ -z "$token" ]; then
  # 2. 如果没有，通过 CLI 获取
  token=$(datecalendar-cli auth token)
fi

# 3. 使用 token 调用 API
curl -H "Authorization: Bearer $token" \
  http://127.0.0.1:9876/api/tasks
```

**当前**：workbuddy 可以直接调用 API，无需 token。

### TC-10: Token 失效处理（未来预期）

```bash
# 模拟 token 失效（手动删除 settings 表中的 api_token）

# workbuddy 收到 401 响应
response=$(curl -s -w "%{http_code}" -H "Authorization: Bearer old_token" \
  http://127.0.0.1:9876/api/tasks)

if [ "$response" = "401" ]; then
  echo "Token 失效，重新获取..."
  new_token=$(datecalendar-cli auth token)
  # 重试请求...
fi
```

**当前**：无需处理 token 失效，因为认证未启用。

---

## 安全验证 — 未来实现时

### TC-11: 局域网访问测试

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 从同一局域网的另一台机器访问 `http://<HOST_IP>:9876/api/tasks` | 连接拒绝（Actix-web 绑定 127.0.0.1） |

### TC-12: Token 泄露影响

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 假设 token 泄露，但从其他机器访问 | 无法访问（仅本机） |
| 2 | 重置 token：`datecalendar-cli auth reset-token` | 旧 token 失效 |

---

## 技术验证 — 未来实现时

```bash
# 编译
cd datecalendar/src-tauri
cargo check

# 启动 Tauri 应用（另一个终端）
# 在 Tauri 应用中：打开 DevTools 控制台，检查是否有错误

# 测试认证流程（未来）
# 1. 无认证
curl -v http://127.0.0.1:9876/api/tasks
# **未来** 应返回 401；**当前** 返回 200

# 2. 获取 token
# （需要先找到 setup_secret，或直接使用 CLI）
token=$(datecalendar-cli auth token)

# 3. 使用 token
curl -H "Authorization: Bearer $token" http://127.0.0.1:9876/api/tasks
# **未来** 应返回 200；**当前** 无需 token 即可返回 200

# 4. 白名单
curl http://127.0.0.1:9876/api/health
# 应返回 200，无需 token
```

---

## 当前状态总结

| 状态 | 说明 |
|------|------|
| ✅ 设计预留 | D-19 设计文档已完整，包含认证方案、Token 管理、中间件实现 |
| ✅ OpenAPI 规范 | D-20 的 OpenAPI 规范中包含 `BearerAuth` 安全方案定义 |
| ❌ 暂不实现 | 认证中间件、Token 生成、Token 验证均暂不实现 |
| ❌ 暂不测试 | 本测试计划中的所有测试用例均暂不执行 |

### 未来实现时机

当出现以下情况时，再实现认证并执行本测试计划：
- 需要局域网访问（手机访问桌面端）
- 有多用户场景
- 有安全审计要求
