//! OpenAPI 文档生成
//! 
//! 提供手动编写的 OpenAPI 3.0 规范 JSON

/// 获取 OpenAPI 规范 JSON 字符串
/// 
/// 这是一个简化的 OpenAPI 规范，包含基本的 API 信息
/// 未来可以使用 utoipa 宏自动生成
pub fn get_openapi_json() -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "openapi": "3.0.3",
        "info": {
            "title": "DateCalendar API",
            "version": "1.0.0",
            "description": "DateCalendar 任务与日程管理 API"
        },
        "paths": {
            "/api/health": {
                "get": {
                    "summary": "健康检查",
                    "responses": {
                        "200": {
                            "description": "成功",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "status": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
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
                                        "items": { "$ref": "#/components/schemas/Task" }
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
                                "schema": { "$ref": "#/components/schemas/NewTask" }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "任务创建成功",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/Task" }
                                }
                            }
                        }
                    }
                }
            },
            "/api/tasks/{id}": {
                "get": {
                    "summary": "获取单个任务",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "成功获取任务",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/Task" }
                                }
                            }
                        }
                    }
                },
                "put": {
                    "summary": "更新任务",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        }
                    ],
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/UpdateTask" }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "任务更新成功",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/Task" }
                                }
                            }
                        }
                    }
                },
                "delete": {
                    "summary": "删除任务",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "任务删除成功"
                        }
                    }
                }
            },
            "/api/schedules": {
                "get": {
                    "summary": "获取所有日程",
                    "responses": {
                        "200": {
                            "description": "成功获取日程列表",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": { "$ref": "#/components/schemas/Schedule" }
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "创建日程",
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/NewSchedule" }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "日程创建成功",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/Schedule" }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "Task": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "title": { "type": "string" },
                        "description": { "type": "string" },
                        "status": { "type": "string" },
                        "priority": { "type": "integer" },
                        "created_at": { "type": "string", "format": "date-time" }
                    }
                },
                "NewTask": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "description": { "type": "string" },
                        "priority": { "type": "integer" }
                    }
                },
                "Schedule": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "task_id": { "type": "string" },
                        "title": { "type": "string" },
                        "start_time": { "type": "string", "format": "date-time" },
                        "end_time": { "type": "string", "format": "date-time" }
                    }
                },
                "NewSchedule": {
                    "type": "object",
                    "properties": {
                        "task_id": { "type": "string" },
                        "title": { "type": "string" },
                        "start_time": { "type": "string", "format": "date-time" },
                        "end_time": { "type": "string", "format": "date-time" }
                    }
                }
            }
        }
    })).unwrap_or_else(|e| {
        eprintln!("Failed to generate OpenAPI spec: {}", e);
        "{}".to_string()
    })
}

