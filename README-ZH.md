# OpenAPI-RS

[English](README.md) | [中文](README-ZH.md)

---

## 中文

一个功能强大的 Rust OpenAPI 3.1 库，提供 OpenAPI 规范的解析、验证和请求处理功能。

### 🚀 特性

- **OpenAPI 3.1 支持**: 完整支持 OpenAPI 3.1 规范
- **YAML 解析**: 支持从 YAML 格式解析 OpenAPI 文档
- **请求验证**: 全面的 HTTP 请求验证，包括：
    - 路径参数验证
    - 查询参数验证
    - 请求体验证
- **类型安全**: 强类型支持，包括联合类型和复合类型
- **格式验证**: 支持多种数据格式验证（Email、UUID、日期时间等）
- **Axum 集成**: 提供 Axum 框架的集成支持
- **详细错误信息**: 提供清晰的验证错误消息

### 📦 安装

将以下内容添加到你的 `Cargo.toml` 文件中：

```toml
[dependencies]
openapi-rs = { git = "https://github.com/baerwang/openapi-rs" }
axum = "0.7"
```

### 🔧 使用方法

```rust
use openapi_rs::model::parse::OpenAPI;
use openapi_rs::request::axum::RequestData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从 YAML 文件解析 OpenAPI 规范
    // 你可以使用项目中的示例文件：examples/api.yaml
    let content = std::fs::read_to_string("examples/api.yaml")?;
    let openapi = OpenAPI::yaml(&content)?;

    // 创建请求数据进行验证
    let request_data = RequestData {
        path: "/users".to_string(),
        inner: axum::http::Request::builder()
            .method("GET")
            .uri("/users?page=1&limit=10")
            .body(axum::body::Body::empty())
            .unwrap(),
        body: None,
    };

    // 根据 OpenAPI 规范验证请求
    openapi.validator(request_data)?;

    // 对于带请求体的 POST 请求
    let body_data = r#"{"name": "John Doe", "email": "john.doe@example.com", "age": 30}"#;
    let post_request = RequestData {
        path: "/users".to_string(),
        inner: axum::http::Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(body_data))
            .unwrap(),
        body: Some(axum::body::Bytes::from(body_data)),
    };

    openapi.validator(post_request)?;

    Ok(())
}
```

**示例 OpenAPI 规范文件 (`examples/api.yaml`)：**

这个库包含一个完整的示例 OpenAPI 规范文件，展示了用户管理 API 的定义，包括：

- 📝 **用户 CRUD 操作**：创建、读取、更新、删除用户
- 🔍 **查询参数验证**：分页、搜索等参数
- 📋 **请求体验证**：JSON 格式的用户数据
- 🏷️ **数据类型验证**：字符串、数字、布尔值、数组等
- 📧 **格式验证**：Email、UUID、日期时间等

### 🎯 支持的验证类型

#### 数据类型

- **字符串**: 支持长度限制、格式验证
- **数字**: 支持最小值、最大值验证
- **整数**: 支持范围验证
- **布尔值**: 类型验证
- **数组**: 支持项目数量限制
- **对象**: 支持嵌套属性验证
- **联合类型**: 支持多类型验证

#### 格式验证

- Email (`email`)
- UUID (`uuid`)
- 日期 (`date`)
- 时间 (`time`)
- 日期时间 (`date-time`)
- IPv4 地址 (`ipv4`)
- IPv6 地址 (`ipv6`)
- Base64 编码 (`base64`)
- 二进制数据 (`binary`)

#### 验证约束

- 字符串长度 (`minLength`, `maxLength`)
- 数值范围 (`minimum`, `maximum`)
- 数组项目数 (`minItems`, `maxItems`)
- 必填字段 (`required`)
- 枚举值 (`enum`)
- 正则表达式 (`pattern`)

### 📁 项目结构

```
src/
├── lib.rs              # 库入口
├── model/              # 数据模型
│   ├── mod.rs
│   └── parse.rs        # OpenAPI 解析模型
├── request/            # 请求处理
│   ├── mod.rs
│   └── axum.rs         # Axum 框架集成
└── validator/          # 验证器
    ├── mod.rs          # 核心验证逻辑
    └── validator_test.rs
```

### 🧪 测试

运行测试：

```bash
cargo test
```

### 📋 开发路线图

- [x] **解析器**: OpenAPI 3.1 规范解析
- [ ] **验证器**: 完整的请求验证功能
- [ ] **更多框架集成**: 支持 Warp、Actix-web 等框架
- [ ] **性能优化**: 提升大型 API 规范的处理性能

### 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启 Pull Request

### 📄 许可证

本项目采用 Apache License 2.0 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。
