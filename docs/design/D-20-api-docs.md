# D-20: API 文档

## 1. 必要性 (Why)

### 问题

当前 HTTP API 已完成路由实现（D-11），但缺少：
- 结构化的 API 文档，workbuddy 开发者不知道如何调用
- 自动生成的 OpenAPI/Swagger 文档，无法用标准工具测试
- 请求/响应示例，手动测试时需要猜格式

### 场景

- workbuddy 开发者需要查看完整的 API 规范来集成
- 用户想用 Postman/Insomnia 手动测试 API
- 前端开发者需要了解 API 契约（请求格式、响应格式、错误码）
- 未来如果需要支持其他客户端（手机 App、Web 前端），需要标准 API 文档

### 设计原则

- **自动生成**：从代码注释/属性自动生成，不手写文档
- **OpenAPI 3.0 标准**：业界标准，工具链丰富
- **可交互**：生成 Swagger UI，可直接在浏览器中测试 API
- **与代码同步**：文档随代码更新，不会过时

---

## 2. 实现方案 (How)

### 2.1 技术选型

| 技术 | 用途 | 理由 |
|------|------|------|
| `utoipa` | OpenAPI 规范生成 | 与 Actix-web 集成好，宏驱动 |
| `utoipa-swagger-ui` | Swagger UI 服务 | 提供可交互的文档页面 |
| `utoipa-rapidoc` | RapiDoc UI（可选） | 另一种文档 UI |

**选择 `utoipa` 的原因**：
- 纯 Rust，无 Node.js 依赖
- 宏驱动，编译时检查
- 与 Actix-web 有官方集成示例

### 2.2 实现步骤

**Step 1：添加依赖**

```toml
# datecalendar/src-tauri/Cargo.toml
[dependencies]
utoipa = { version = "5", features = ["actix_extras"] }
utoipa-swagger-ui = { version = "5", features = ["actix-web"] }
```

**Step 2：用宏标注 API 路由**

```rust
// api/task_routes.rs

use utoipa::ToSchema;
use utoipa_actix_web::path;

/// 获取所有任务
#[utoipa::path(
    get,
    path = "/api/tasks",
    responses(
        (status = 200, description = "成功获取任务列表", body = Vec<TaskDto>),
        (status = 401, description = "未认证")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_all_tasks(svc: web::Data<TaskService>) -> HttpResponse { ... }

/// 创建任务
#[utoipa::path(
    post,
    path = "/api/tasks",
    request_body = NewTaskDto,
    responses(
        (status = 201, description = "任务创建成功", body = TaskDto),
        (status = 400, description = "请求参数错误"),
        (status = 401, description = "未认证")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn create_task(svc: web::Data<TaskService>, input: web::Json<NewTaskDto>) -> HttpResponse { ... }
```

**Step 3：定义 OpenAPI 规范**

```rust
// api/openapi.rs

use utoipa::{OpenApi, security::{Bearer, SecurityScheme}};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "DateCalendar API",
        version = "1.0.0",
        description = "DateCalendar 任务与日程管理 API",
        contact(
            name = "DateCalendar",
            url = "https://github.com/yourusername/DateCalendar"
        )
    ),
    paths(
        task_routes::get_all_tasks,
        task_routes::create_task,
        task_routes::get_task,
        task_routes::update_task,
        task_routes::delete_task,
        // ... 其他路由
    ),
    components(
        schemas(TaskDto, NewTaskDto, ScheduleDto, NewScheduleDto)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tags(
        (name = "tasks", description = "任务管理"),
        (name = "schedules", description = "日程管理"),
        (name = "auth", description = "认证")
    )
)]
pub struct ApiDoc;

// Bearer 认证方案
impl SecurityScheme for BearerAuth {
    fn scheme() -> &'static str {
        "bearer"
    }
}
```

**Step 4：挂载 Swagger UI**

```rust
// api/server.rs

use utoipa_swagger_ui::SwaggerUi;

pub fn start_api_server(pool: Pool<SqliteConnectionManager>) {
    std::thread::spawn(move || {
        let task_service = web::Data::new(TaskService::new(pool.clone()));
        let schedule_service = web::Data::new(ScheduleService::new(pool.clone()));
        
        // 生成 OpenAPI 规范
        let openapi = ApiDoc::openapi();

        let server = actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .wrap(Cors::default().allow_any_origin())
                .app_data(task_service.clone())
                .app_data(schedule_service.clone())
                // Swagger UI 挂载到 /docs
                .service(
                    SwaggerUi::new("/docs/{_:.*}")
                        .url("/api-docs/openapi.json", openapi.clone())
                )
                // 健康检查
                .route("/api/health", web::get().to(health))
                // 任务路由
                .configure(task_routes::configure)
                // 日程路由
                .configure(schedule_routes::configure)
        })
        .bind("127.0.0.1:9876")
        .expect("HTTP API 绑定失败")
        .run();

        actix_web::rt::System::new().block_on(server).unwrap();
    });
}
```

### 2.3 文档访问方式

启动 Tauri 应用后，API 文档可通过以下 URL 访问：

| URL | 内容 |
|-----|------|
| `http://127.0.0.1:9876/docs` | Swagger UI 交互式文档 |
| `http://127.0.0.1:9876/api-docs/openapi.json` | OpenAPI 规范（JSON） |
| `http://127.0.0.1:9876/api-docs/openapi.yaml` | OpenAPI 规范（YAML） |

### 2.4 OpenAPI 规范示例

生成的 `openapi.json` 片段：

```json
{
  "openapi": "3.0.3",
  "info": {
    "title": "DateCalendar API",
    "version": "1.0.0",
    "description": "DateCalendar 任务与日程管理 API"
  },
  "security": [
    { "bearer_auth": [] }
  ],
  "paths": {
    "/api/tasks": {
      "get": {
        "summary": "获取所有任务",
        "responses": {
          "200": {
            "description": "成功获取任务列表",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": { "$ref": "#/components/schemas/TaskDto" }
                }
              }
            }
          }
        }
      },
      "post": {
        "summary": "创建任务",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": { "$ref": "#/components/schemas/NewTaskDto" }
            }
          }
        },
        "responses": { ... }
      }
    }
  },
  "components": {
    "securitySchemes": {
      "bearer_auth": {
        "type": "http",
        "scheme": "bearer"
      }
    },
    "schemas": { ... }
  }
}
```

### 2.5 workbuddy Skill 如何使用 API 文档

workbuddy Skill 可以：

1. **读取 OpenAPI 规范**：`GET http://127.0.0.1:9876/api-docs/openapi.json`
2. **自动生成调用代码**：基于 OpenAPI 规范，workbuddy 可以自动生成正确的 API 调用
3. **验证请求格式**：确保发送的 JSON 符合规范

---

## 3. 验证标准 (Verify)

### 功能验证

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| 访问 `/docs` | 显示 Swagger UI 页面 | 浏览器打开 |
| 访问 `/api-docs/openapi.json` | 返回合法 JSON | `curl ... \| jq .` |
| Swagger UI 中点击「Authorize」 | 可输入 Bearer token | 手动测试 |
| 在 Swagger UI 中执行 `GET /api/tasks` | 返回任务列表（或 401） | 手动测试 |
| OpenAPI 规范包含全部路由 | 所有 `/api/` 路径都在规范中 | 对比路由注册代码 |
| Schema 定义完整 | TaskDto、ScheduleDto 等都有定义 | 检查 `openapi.json` |

### workbuddy 集成验证

| 场景 | 预期 |
|------|------|
| workbuddy 读取 openapi.json | 成功解析，获取所有端点信息 |
| workbuddy 根据规范生成请求 | 请求格式正确，能被 API 接受 |
| workbuddy 在 Swagger UI 中测试 | 所有端点都可从 Swagger UI 成功调用 |

### 技术验证

```bash
# 编译（包含 utoipa 依赖）
cd datecalendar/src-tauri && cargo check

# 启动 Tauri 应用
# 在另一个终端：
# 1. 获取 OpenAPI 规范
curl http://127.0.0.1:9876/api-docs/openapi.json | jq .info.title
# 应输出: "DateCalendar API"

# 2. 打开 Swagger UI
start http://127.0.0.1:9876/docs

# 3. 在 Swagger UI 中测试 API
#    - 点击「Authorize」输入 token
#    - 执行 GET /api/tasks
#    - 执行 POST /api/tasks
```
