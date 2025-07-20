# OpenAPI-RS

[English](README.md) | [中文](README-ZH.md)

---

A powerful Rust library for OpenAPI 3.1 specification parsing, validation, and request handling.

### 🚀 Features

- **OpenAPI 3.1 Support**: Full compatibility with OpenAPI 3.1 specification
- **YAML Parsing**: Support for parsing OpenAPI documents from both YAML formats
- **Request Validation**: Comprehensive HTTP request validation including:
    - Path parameter validation
    - Query parameter validation
    - Request body validation
- **Type Safety**: Strong typing support with union types and composite types
- **Format Validation**: Support for various data format validations (Email, UUID, DateTime, etc.)
- **Axum Integration**: Built-in integration support for the Axum framework
- **Detailed Error Messages**: Clear and informative validation error messages

### 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
openapi-rs = { git = "https://github.com/baerwang/openapi-rs" }
axum = "0.7"
```

### 🔧 Usage

```rust
use openapi_rs::model::parse::OpenAPI;
use openapi_rs::request::axum::RequestData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse OpenAPI specification from YAML file
    // You can use the example file included in the project: examples/api.yaml
    let content = std::fs::read_to_string("examples/api.yaml")?;
    let openapi = OpenAPI::yaml(&content)?;

    // Create request data for validation
    let request_data = RequestData {
        path: "/users".to_string(),
        inner: axum::http::Request::builder()
            .method("GET")
            .uri("/users?page=1&limit=10")
            .body(axum::body::Body::empty())
            .unwrap(),
        body: None,
    };

    // Validate the request against OpenAPI specification
    openapi.validator(request_data)?;

    // For POST requests with body
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

**Example OpenAPI Specification File (`examples/api.yaml`):**

This library includes a complete example OpenAPI specification file that demonstrates a User Management API definition,
featuring:

- 📝 **User CRUD Operations**: Create, Read, Update, Delete users
- 🔍 **Query Parameter Validation**: Pagination, search parameters
- 📋 **Request Body Validation**: JSON formatted user data
- 🏷️ **Data Type Validation**: Strings, numbers, booleans, arrays
- 📧 **Format Validation**: Email, UUID, date-time formats

### 🎯 Supported Validation Types

#### Data Types

- **String**: Length constraints and format validation
- **Number**: Minimum and maximum value validation
- **Integer**: Range validation
- **Boolean**: Type validation
- **Array**: Item count constraints
- **Object**: Nested property validation
- **Union Types**: Multi-type validation

#### Format Validation

- Email (`email`)
- UUID (`uuid`)
- Date (`date`)
- Time (`time`)
- Date-Time (`date-time`)
- IPv4 Address (`ipv4`)
- IPv6 Address (`ipv6`)
- Base64 Encoding (`base64`)
- Binary Data (`binary`)

#### Validation Constraints

- String length (`minLength`, `maxLength`)
- Numeric ranges (`minimum`, `maximum`)
- Array item count (`minItems`, `maxItems`)
- Required fields (`required`)
- Enum values (`enum`)
- Pattern matching (`pattern`)

### 📁 Project Structure

```
src/
├── lib.rs              # Library entry point
├── model/              # Data models
│   ├── mod.rs
│   └── parse.rs        # OpenAPI parsing models
├── request/            # Request handling
│   ├── mod.rs
│   └── axum.rs         # Axum framework integration
└── validator/          # Validators
    ├── mod.rs          # Core validation logic
    └── validator_test.rs
```

### 🧪 Testing

Run tests:

```bash
cargo test
```

### 📋 Roadmap

- [x] **Parser**: OpenAPI 3.1 specification parsing
- [ ] **Validator**: Complete request validation functionality
- [ ] **More Framework Integration**: Support for Warp, Actix-web, and other frameworks
- [ ] **Performance Optimization**: Improve handling of large API specifications

### 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.