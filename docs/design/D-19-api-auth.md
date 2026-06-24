# D-19: API 认证（保留入口，暂不实现）

## 1. 必要性 (Why)

### 项目定位

DateCalendar 是个人使用的桌面应用，仅在本人电脑上运行：
- HTTP API 绑定 `127.0.0.1`（仅本机可访问）
- 无局域网/互联网暴露计划
- 无多用户场景

### 保留认证入口的原因

虽然当前无需实现认证，但保留设计入口有以下好处：
1. **未来扩展**：如果后续需要局域网访问（如手机访问桌面端），认证设计已就绪
2. **workbuddy 集成**：workbuddy 可以假设 API 有认证，提前做好 token 管理逻辑
3. **OpenAPI 规范完整**：API 文档中包含安全方案，符合业界标准

### 设计原则（预留）

- **本地 token 足够**：不需要 OAuth、JWT 等复杂方案，静态 Bearer token 即可
- **简单获取**：token 存储在配置文件中，用户可查看/重置
- **可选关闭**：开发模式下可关闭认证（通过环境变量或配置）
- **错误提示友好**：401 时返回清晰的错误信息，包含如何获取 token 的提示

---

## 2. 实现方案 (How) — 预留设计

### 2.1 认证方案选择

| 方案 | 优点 | 缺点 | 适用性 |
|------|------|------|--------|
| HTTP Basic Auth | 简单 | 每次传密码，不够安全 | ❌ 不适合 |
| Bearer Token (静态) | 简单、标准、workbuddy 友好 | token 泄露 = 全权限 | ✅ 适合（预留） |
| API Key (Query 参数) | 简单 | URL 中可见，不安全 | ❌ 不适合 |
| mTLS (客户端证书) | 最安全 | 配置复杂，workbuddy 难以集成 | ❌ 过度设计 |

**预留方案**：Bearer Token（静态），通过 `Authorization: Bearer <token>` 头传递。

### 2.2 Token 生成与存储（预留）

**生成时机**：Tauri 应用首次启动时自动生成，存储在 `settings` 表中。

```sql
-- settings 表已有，复用它
INSERT INTO settings (key, value) VALUES ('api_token', '<uuid>');
```

**Token 格式**：UUID v4，例如 `a1b2c3d4-e5f6-7890-abcd-ef1234567890`

**查看 token**：提供 API 端点（无需认证）和 CLI 命令：

```bash
# HTTP 端点（无需认证，仅用于获取 token）
GET /api/auth/token?secret=<setup_secret>

# CLI 命令（直接读数据库，无需认证）
datecalendar-cli auth token
```

> `setup_secret` 是应用首次启动时生成的一次性密钥，存储在用户目录下，仅本机可读取。

### 2.3 认证中间件实现（预留）

使用 Actix-web 的 `mw` (middleware) 机制：

```rust
// api/auth.rs（预留，暂不实现）

use actix_web::{
    dev::{forward_ready, ServiceRequest, ServiceResponse},
    Error, HttpResponse,
};
use futures::future::{ready, Ready};
use std::future::Future;
use std::pin::Pin;

// 不需要认证的路径白名单
const PUBLIC_PATHS: &[&str] = &[
    "/api/health",
    "/api/auth/token",
];

pub struct AuthMiddleware {
    pub pool: Pool<SqliteConnectionManager>,
}

// ... 实现细节见上面 "实现方案" 章节（预留）...
```

### 2.4 白名单与配置（预留）

**不需要认证的路径**：

| 路径 | 方法 | 说明 |
|------|------|------|
| `/api/health` | GET | 健康检查，监控系统用 |
| `/api/auth/token` | GET | 获取 token（需要 setup_secret） |

**可配置项**（存储在 `settings` 表）：

| Key | 默认值 | 说明 |
|-----|--------|------|
| `api_token` | (自动生成) | Bearer Token |
| `api_auth_enabled` | `true` | 是否启用认证 |
| `api_setup_secret` | (自动生成) | 首次获取 token 用的一次性密钥 |

### 2.5 获取 Token 的流程（预留）

**首次获取 token**（workbuddy 或用户设置时）：

```bash
# 方式一：通过 setup_secret（仅首次）
curl "http://127.0.0.1:9876/api/auth/token?secret=<setup_secret>"
# 返回: {"token": "a1b2c3d4-..."}

# 方式二：通过 CLI（直接读数据库，最可靠）
datecalendar-cli auth token
# 返回: a1b2c3d4-...

# 方式三：直接读配置文件
cat %APPDATA%\DateCalendar\api_token.txt
```

**后续使用 token**：

```bash
# 所有 API 请求都需要带 Authorization 头
curl -H "Authorization: Bearer a1b2c3d4-..." \
  http://127.0.0.1:9876/api/tasks
```

### 2.6 文件结构（预留）

```
datecalendar/src-tauri/src/
├── api/
│   ├── mod.rs           # 预留：注册认证中间件
│   ├── server.rs        # 预留：添加 AuthMiddleware
│   ├── auth.rs         # 预留：认证中间件 + /api/auth/token 端点
│   ├── task_routes.rs  # 不变（中间件会自动保护）
│   └── schedule_routes.rs  # 不变
```

---

## 3. 当前状态 (Status)

### ✅ 已完成的准备工作

1. **OpenAPI 规范中包含安全方案**：`D-20-api-docs.md` 的 OpenAPI 规范中包含 `BearerAuth` 安全方案定义
2. **API 文档标注**：所有需要认证的端点都标注了 `#[openapi(security(...))]`（在 D-20 中实现）
3. **CLI 无认证设计**：CLI 直接访问数据库，无需认证（本机可信）

### ❌ 暂不实现的部分

1. **认证中间件**：不实现 `auth.rs` 和 `AuthMiddleware`
2. **Token 生成**：不在应用启动时自动生成 `api_token`
3. **Token 验证**：所有 API 端点暂不验证 `Authorization` 头
4. **`/api/auth/token` 端点**：不实现 Token 获取端点

### ⏳ 未来实现时机

当出现以下情况时，再实现认证：
- 需要局域网访问（手机访问桌面端）
- 有多用户场景
- 有安全审计要求

---

## 4. 验证标准 (Verify) — 当前无需验证

由于认证暂不实现，以下验证标准仅作为未来实现时的参考：

### 功能验证（未来）

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| 无 Authorization 头访问 `/api/tasks` | 401 + 错误提示 | `curl http://127.0.0.1:9876/api/tasks` |
| 错误 token | 401 + "Invalid API token" | `curl -H "Authorization: Bearer wrong" ...` |
| 正确 token | 200 + 正常返回 | `curl -H "Authorization: Bearer <valid>" ...` |
| `/api/health` 无 token | 200（白名单） | `curl http://127.0.0.1:9876/api/health` |

### workbuddy 集成验证（未来）

| 场景 | 调用方式 | 预期 |
|------|----------|------|
| workbuddy 首次连接 | 读取 `~/.datecalendar/token` 或执行 `datecalendar-cli auth token` | 获取 token |
| workbuddy 调用 API | `fetch(url, {headers: {'Authorization': 'Bearer '+token}})` | 成功 |
| token 失效处理 | API 返回 401 → workbuddy 重新获取 token | 自动恢复 |
