# 测试流程：API 文档

> 对应设计文档：[D-20 API 文档](../design/D-20-api-docs.md)

## 前置条件

- DateCalendar Tauri 应用已启动（HTTP API 在 `localhost:9876` 运行）
- 已集成 `utoipa` 和 `utoipa-swagger-ui`
- 浏览器可用

---

## 白盒测试（Rust 后端）

```bash
cd datecalendar/src-tauri
cargo test --lib api::openapi::tests -- --nocapture
```

### 覆盖用例

| # | 用例 | 验证点 |
|---|------|--------|
| 1 | `test_openapi_spec_valid` | 生成的 OpenAPI 规范是合法的 JSON |
| 2 | `test_openapi_paths_complete` | 所有路由都在规范中 |
| 3 | `test_openapi_schemas_complete` | 所有 DTO 都有 schema 定义 |
| 4 | `test_swagger_ui_serves` | `/docs` 路径返回 HTML |
| 5 | `test_openapi_json_serves` | `/api-docs/openapi.json` 返回 JSON |
| 6 | `test_security_scheme_defined` | Bearer Auth 在规范中定义 |

---

## 手动黑盒测试

### TC-01: 访问 Swagger UI

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 浏览器打开 `http://127.0.0.1:9876/docs` | 显示 Swagger UI 页面 |
| 2 | 页面加载完成 | 左侧列出所有 API 端点 |
| 3 | 点击「Authorize」按钮 | 弹出输入 Bearer token 的对话框 |

### TC-02: 在 Swagger UI 中测试 API

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 在 Swagger UI 中展开 `GET /api/tasks` | 显示该端点的描述 |
| 2 | 点击「Try it out」 | 进入测试模式 |
| 3 | 点击「Execute」 | 发送请求，显示响应 |
| 4 | 检查响应码 | 应为 200 或 401（如果未认证） |
| 5 | 先点击「Authorize」输入 token，再执行 | 应为 200 |

### TC-03: 获取 OpenAPI 规范

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | `curl http://127.0.0.1:9876/api-docs/openapi.json` | 返回 JSON |
| 2 | 检查 JSON 的 `openapi` 字段 | 值为 `"3.0.3"`（或类似版本） |
| 3 | 检查 `paths` 字段 | 包含所有 `/api/` 路由 |
| 4 | 检查 `components.schemas` | 包含 `TaskDto`、`ScheduleDto` 等 |

### TC-04: OpenAPI 规范格式验证

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 下载规范：`curl http://127.0.0.1:9876/api-docs/openapi.json -o openapi.json` | 保存成功 |
| 2 | 使用在线验证工具（如 https://editor.swagger.io/） | 规范合法，无错误 |
| 3 | 使用 `swagger-cli` 验证：`npx @apidevtools/swagger-cli validate openapi.json` | 输出 `openapi.json is valid` |

---

## 工具集成测试

### TC-05: Postman 导入规范

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 打开 Postman | 应用启动 |
| 2 | Import → Link: `http://127.0.0.1:9876/api-docs/openapi.json` | 成功导入 |
| 3 | 查看导入的集合 | 包含所有端点 |
| 4 | 在 Postman 中执行 `GET /api/tasks` | 返回响应 |

### TC-06: VSCode REST Client 插件

创建 `test.http` 文件：

```http
# 在 VSCode 中使用 REST Client 插件

@baseUrl = http://127.0.0.1:9876
@token = <your_token>

### 获取所有任务
GET {{baseUrl}}/api/tasks
Authorization: Bearer {{token}}

### 创建任务
POST {{baseUrl}}/api/tasks
Authorization: Bearer {{token}}
Content-Type: application/json

{
  "title": "REST Client 测试",
  "priority": 1
}
```

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 在 VSCode 中打开 `test.http` | 文件显示 |
| 2 | 点击「Send Request」（`GET /api/tasks`） | 返回响应 |
| 3 | 点击「Send Request」（`POST /api/tasks`） | 创建任务成功 |

---

## workbuddy 集成验证

### TC-07: workbuddy 读取 OpenAPI 规范

模拟 workbuddy 读取规范并生成 API 调用：

```bash
# 1. workbuddy 读取规范
spec=$(curl -s http://127.0.0.1:9876/api-docs/openapi.json)

# 2. 解析规范，获取创建任务的端点信息
# （实际中 workbuddy 会用 JSON 解析器）
echo "$spec" | jq '.paths["/api/tasks"].post'

# 3. 根据规范构造请求
# （workbuddy 自动生成正确的请求体和头）
```

### TC-08: 根据规范验证请求格式

```bash
# 使用 OpenAPI 规范验证请求格式
# （可以用 `openapi-validator` 等工具）

# 示例：验证创建任务的请求体
request_body='{"title":"验证测试"}'

# 假设有验证工具
# openapi-validator validate openapi.json --request POST /api/tasks --body "$request_body"
```

---

## 文档完整性检查

### TC-09: 所有端点都有文档

对比路由注册代码（`api/task_routes.rs`、`api/schedule_routes.rs`）和 OpenAPI 规范中的 `paths`：

| 路由文件中的路径 | 是否在规范中 |
|----------------|------------|
| `/api/tasks` (GET) | ✅ / ❌ |
| `/api/tasks` (POST) | ✅ / ❌ |
| `/api/tasks/{id}` (GET) | ✅ / ❌ |
| ... | ... |

### TC-10: 所有 DTO 都有 Schema

对比 Rust 结构体定义和 OpenAPI 规范中的 `components/schemas`：

| Rust 结构体 | 是否在规范中 |
|-------------|------------|
| `TaskDto` | ✅ / ❌ |
| `NewTaskDto` | ✅ / ❌ |
| `ScheduleDto` | ✅ / ❌ |
| `NewScheduleDto` | ✅ / ❌ |

---

## 技术验证

```bash
# 编译（包含 utoipa 依赖）
cd datecalendar/src-tauri
cargo check

# 启动 Tauri 应用（另一个终端）

# 测试 1：访问 Swagger UI
start http://127.0.0.1:9876/docs

# 测试 2：获取 OpenAPI 规范
curl http://127.0.0.1:9876/api-docs/openapi.json | jq .info

# 测试 3：验证规范格式
curl http://127.0.0.1:9876/api-docs/openapi.json -o openapi.json
npx @apidevtools/swagger-cli validate openapi.json

# 测试 4：在 Swagger UI 中测试 API
#   （手动在浏览器中操作）
#   1. 打开 http://127.0.0.1:9876/docs
#   2. 点击「Authorize」输入 token
#   3. 执行 GET /api/tasks
#   4. 执行 POST /api/tasks
```
