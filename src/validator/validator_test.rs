/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#[cfg(test)]
mod tests {
    use crate::model::parse::{Format, OpenAPI};
    use crate::request;
    use crate::validator::validate_field_format;
    use axum::body::Bytes;
    use serde_json::Value;

    fn make_request_body_with_value(value: &str) -> request::axum::RequestData {
        request::axum::RequestData {
            path: "/example".to_string(),
            inner: axum::http::Request::builder()
                .method("POST")
                .uri("/example")
                .body(axum::body::Body::from(format!("{}", value)))
                .unwrap(),
            body: Some(Bytes::from(format!("{}", value))),
        }
    }

    #[test]
    fn test_uuid_path_validation() {
        let content = r#"
openapi: 3.1.0
info:
  title: Example API
  description: API definitions for example
  version: '0.0.1'
  x-file-identifier: example

components:
  schemas:
    ExampleResponse:
      properties:
        uuid:
          type: string
          description: The UUID for this example.
          format: uuid
          example: 00000000-0000-0000-0000-000000000000

security: [ ]

paths:
  /example/{uuid}:
    get:
      parameters:
        - name: uuid
          description: The UUID for this example.
          in: path
          schema:
            type: string
            format: uuid
            example: 00000000-0000-0000-0000-000000000000
      responses:
        '200':
          description: Get a Example response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ExampleResponse'
"#;

        let openapi: OpenAPI = OpenAPI::yaml(content).expect("Failed to parse OpenAPI content");

        fn make_request(uri: &str) -> request::axum::RequestData {
            request::axum::RequestData {
                path: "/example/{uuid}".to_string(),
                inner: axum::http::Request::builder()
                    .method("GET")
                    .uri(uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
                body: None,
            }
        }

        struct Tests {
            value: &'static str,
            assert: bool,
        }

        let tests: Vec<Tests> = vec![
            Tests {
                value: "/example/00000000-0000-0000-0000-000000000000",
                assert: true,
            },
            Tests {
                value: "/example/00000000",
                assert: false,
            },
        ];

        for test in tests {
            assert_eq!(
                openapi.validator(make_request(test.value)).is_ok(),
                test.assert
            );
        }
    }

    #[test]
    fn test_uuid_query_validation() {
        let content = r#"
openapi: 3.1.0
info:
  title: Example API
  description: API definitions for example
  version: '0.0.1'
  x-file-identifier: example

components:
  schemas:
    ExampleResponse:
      properties:
        uuid:
          type: string
          description: The UUID for this example.
          format: uuid
          example: 00000000-0000-0000-0000-000000000000

security: [ ]

paths:
  /example:
    get:
      summary: Get a example
      description: Get a example
      operationId: get-a-example
      parameters:
        - name: uuid
          description: UUID of the example
          in: query
          schema:
            type: string
            format: uuid
            example: "00000000-0000-0000-0000-000000000000"
      responses:
        '200':
          description: Get a Example response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ExampleResponse'
"#;

        let openapi: OpenAPI = OpenAPI::yaml(content).expect("Failed to parse OpenAPI content");

        fn make_request(uuid: &str) -> request::axum::RequestData {
            request::axum::RequestData {
                path: "/example".to_string(),
                inner: axum::http::Request::builder()
                    .method("GET")
                    .uri(format!("/example?uuid={}", uuid))
                    .body(axum::body::Body::empty())
                    .unwrap(),
                body: None,
            }
        }

        struct Tests {
            value: &'static str,
            assert: bool,
        }

        let tests: Vec<Tests> = vec![
            Tests {
                value: "00000000-0000-0000-0000-000000000000",
                assert: true,
            },
            Tests {
                value: "00000000-0000-0000-0000-xxxx",
                assert: false,
            },
        ];

        for test in tests {
            assert_eq!(
                openapi.validator(make_request(test.value)).is_ok(),
                test.assert
            );
        }
    }

    #[test]
    fn test_uuid_body_validation() {
        let content = r#"
openapi: 3.1.0
info:
  title: Example API
  description: API definitions for example
  version: '0.0.1'
  x-file-identifier: example

components:
  schemas:
    ExampleRequest:
      type: object
      properties:
        uuid:
          type: string
          description: The UUID for this example.
          format: uuid
          example: 00000000-0000-0000-0000-000000000000
      required:
        - uuid
    ExampleResponse:
      properties:
        uuid:
          type: string
          description: The UUID for this example.
          format: uuid
          example: 00000000-0000-0000-0000-000000000000

security: [ ]

paths:
  /example:
    post:
      requestBody:
        content:
          application/json:
            schema:
                $ref: '#/components/schemas/ExampleRequest'
      responses:
        '200':
          description: Post a Example response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ExampleResponse'
"#;

        let openapi: OpenAPI = OpenAPI::yaml(content).expect("Failed to parse OpenAPI content");

        struct Tests {
            value: &'static str,
            assert: bool,
        }

        let tests: Vec<Tests> = vec![
            Tests {
                value: r#"{"uuid":"00000000-0000-0000-0000-000000000000"}"#,
                assert: true,
            },
            Tests {
                value: r#"{"uuid":"00000000-0000-0000-0000-xxxx"}"#,
                assert: false,
            },
        ];

        for test in tests {
            assert_eq!(
                openapi
                    .validator(make_request_body_with_value(test.value))
                    .is_ok(),
                test.assert
            );
        }
    }

    #[test]
    fn test_query_required_validation() {
        let content = r#"
openapi: 3.1.0
info:
  title: Example API
  description: API definitions for example
  version: '0.0.1'
  x-file-identifier: example

components:
  schemas:
    ExampleResponse:
      properties:
        uuid:
          type: string
          description: The UUID for this example.
          format: uuid
          example: 00000000-0000-0000-0000-000000000000

security: [ ]

paths:
  /example:
    get:
      summary: Get a example
      description: Get a example
      operationId: get-a-example
      parameters:
        - name: uuid
          description: UUID of the example
          in: query
          required: true
          schema:
            type: string
            format: uuid
            example: "00000000-0000-0000-0000-000000000000"
        - name: name
          description:  Name of the example
          in: query
          required: true
          schema:
            type: string
            example: "example"
        - name: age
          description: Age of the example
          in: query
          required: false
          schema:
            type: integer
            example: 1
      responses:
        '200':
          description: Get a Example response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ExampleResponse'
"#;

        let openapi: OpenAPI = OpenAPI::yaml(content).expect("Failed to parse OpenAPI content");

        fn make_request(uri: &str) -> request::axum::RequestData {
            request::axum::RequestData {
                path: "/example".to_string(),
                inner: axum::http::Request::builder()
                    .method("GET")
                    .uri(uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
                body: None,
            }
        }

        struct Tests {
            value: &'static str,
            assert: bool,
        }

        let tests: Vec<Tests> = vec![
            Tests {
                value: "/example?uuid=00000000-0000-0000-0000-000000000000&name=example",
                assert: true,
            },
            Tests {
                value: "/example?uuid=00000000-0000-0000-0000-000000000000&name=example&age=1",
                assert: true,
            },
            Tests {
                value: "/example?uuid=00000000-0000-0000-0000-000000000000&age=1",
                assert: false,
            },
            Tests {
                value: "/example?uuid=00000000-0000-0000-0000-000000000000",
                assert: false,
            },
        ];

        for test in tests {
            assert_eq!(
                openapi.validator(make_request(test.value)).is_ok(),
                test.assert
            );
        }
    }

    #[test]
    fn format_types_validation() {
        fn t(v: &str, format: Format) -> bool {
            validate_field_format("", &Value::from(v), Some(&format)).is_ok()
        }

        struct Tests {
            f: Format,
            value: &'static str,
            assert: bool,
        }

        let tests: Vec<Tests> = vec![
            Tests {
                f: Format::Date,
                value: "2025-01-32",
                assert: false,
            },
            Tests {
                f: Format::Email,
                value: "e@example .com",
                assert: false,
            },
            Tests {
                f: Format::Time,
                value: "00:00:61",
                assert: false,
            },
            Tests {
                f: Format::DateTime,
                value: "2025-01-01T00:61:00Z",
                assert: false,
            },
            Tests {
                f: Format::UUID,
                value: "00000000-0000-0000-0000-xxxx",
                assert: false,
            },
            Tests {
                f: Format::IPV4,
                value: "127.0.0.x",
                assert: false,
            },
            Tests {
                f: Format::IPV6,
                value: "example",
                assert: false,
            },
            Tests {
                f: Format::Email,
                value: "a@example.com",
                assert: true,
            },
            Tests {
                f: Format::UUID,
                value: "00000000-0000-0000-0000-000000000000",
                assert: true,
            },
            Tests {
                f: Format::Time,
                value: "00:00:00",
                assert: true,
            },
            Tests {
                f: Format::Date,
                value: "2025-01-01",
                assert: true,
            },
            Tests {
                f: Format::DateTime,
                value: "2025-01-01T00:00:00Z",
                assert: true,
            },
            Tests {
                f: Format::IPV4,
                value: "127.0.0.1",
                assert: true,
            },
            Tests {
                f: Format::IPV6,
                value: "::",
                assert: true,
            },
        ];

        for test in tests {
            assert_eq!(t(test.value, test.f), test.assert);
        }
    }

    #[test]
    fn test_query_value_limit_validation() {
        let content = r#"
openapi: 3.1.0
info:
  title: Example API
  description: API definitions for example
  version: '0.0.1'
  x-file-identifier: example

components:
  schemas:
    ExampleRequest:
      type: object
      properties:
        name:
          type: string
          description: The Name for this example.
          example: example
          minLength: 1
          maxLength: 7
        age:
          type: integer
          description: The age for this example.
          example: 1
          minimum: 1
          maximum: 10
      required:
        - name
        - age
    ExampleResponse:
      properties:
        name:
          type: string
          description: The Name for this example.
          example: example

security: [ ]

paths:
  /example:
    post:
      requestBody:
        content:
          application/json:
            schema:
                $ref: '#/components/schemas/ExampleRequest'
      responses:
        '200':
          description: Post a Example response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ExampleResponse'
"#;

        let openapi: OpenAPI = OpenAPI::yaml(content).expect("Failed to parse OpenAPI content");

        struct Tests {
            value: &'static str,
            assert: bool,
        }

        let tests: Vec<Tests> = vec![
            Tests {
                value: r#"{"name":"example","age":1}"#,
                assert: true,
            },
            Tests {
                value: r#"{"name":"example","age":100}"#,
                assert: false,
            },
            Tests {
                value: r#"{"name":"example-100","age":1}"#,
                assert: false,
            },
        ];

        for test in tests {
            assert_eq!(
                openapi
                    .validator(make_request_body_with_value(test.value))
                    .is_ok(),
                test.assert
            );
        }
    }

    #[test]
    fn test_body_array_validation() {
        let content = r#"
openapi: 3.1.0
info:
  title: Example API
  description: API definitions for example
  version: '0.0.1'
  x-file-identifier: example

components:
  schemas:
    ExampleRequest:
      type: array
      minItems: 1
      maxItems: 2
      items:
        properties:
          name:
            type: string
            description: The Name for this example.
            example: example
            minLength: 1
            maxLength: 7
          age:
            type: integer
            description: The age for this example.
            example: 1
            minimum: 1
            maximum: 10
        required:
          - name
          - age
    ExampleResponse:
      properties:
        name:
          type: string
          description: The Name for this example.
          example: example

security: [ ]

paths:
  /example:
    post:
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ExampleRequest'
      responses:
        '200':
          description: Post a Example response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ExampleResponse'
"#;

        let openapi: OpenAPI = OpenAPI::yaml(content).expect("Failed to parse OpenAPI content");

        struct Tests {
            value: &'static str,
            assert: bool,
        }

        let tests: Vec<Tests> = vec![
            Tests {
                value: r#"[{"name":"example","age":1}]"#,
                assert: true,
            },
            Tests {
                value: r#"[{"name":"example","age":100}]"#,
                assert: false,
            },
            Tests {
                value: r#"[{"name":"example-100","age":1}]"#,
                assert: false,
            },
            Tests {
                value: r#"[]"#,
                assert: false,
            },
            Tests {
                value: r#"[{"name":"example-100","age":1},{"name":"example-101","age":2},{"name":"example-102","age":2}]"#,
                assert: false,
            },
            Tests {
                value: r#"{"name":"example","age":1}"#,
                assert: false,
            },
        ];

        for test in tests {
            assert_eq!(
                openapi
                    .validator(make_request_body_with_value(test.value))
                    .is_ok(),
                test.assert
            );
        }
    }

    #[test]
    fn test_body_enum_validation() {
        let content = r#"
openapi: 3.1.0
info:
  title: Example API
  description: API definitions for example
  version: '0.0.1'
  x-file-identifier: example

components:
  schemas:
    ExampleRequest:
      type: object
      properties:
        name:
          type: string
          description: The Name for this example.
          example: example
          enum:
            - example
            - example-100
            - example-101
        priority:
          type: integer
          description: Priority level
          enum:
            - 1
            - 2
            - 3
            - 10
        status:
          type: string
          description: Status of the example
          enum:
            - active
            - inactive
            - pending
        enabled:
          type: boolean
          description: Whether the example is enabled
          enum:
            - true
            - false
        category:
          type: number
          description: Category identifier
          enum:
            - 1.0
            - 2.5
            - 3.14
            - 10.0
        mixed_type:
          description: Mixed type enum (string and number)
          enum:
            - "text"
            - 42
            - "another_text"
            - 99.99
      required:
        - name
        - priority
    ExampleResponse:
      properties:
        name:
          type: string
          description: The Name for this example.
          example: example

security: [ ]

paths:
  /example:
    post:
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ExampleRequest'
      responses:
        '200':
          description: Post a Example response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ExampleResponse'
"#;

        let openapi: OpenAPI = OpenAPI::yaml(content).expect("Failed to parse OpenAPI content");

        struct Tests {
            value: &'static str,
            assert: bool,
            description: &'static str,
        }

        let tests: Vec<Tests> = vec![
            Tests {
                value: r#"{"name":"example","priority":1}"#,
                assert: true,
                description: "Valid string and integer enum",
            },
            Tests {
                value: r#"{"name":"example-100","priority":2}"#,
                assert: true,
                description: "Valid string enum variant",
            },
            Tests {
                value: r#"{"name":"example-101","priority":3}"#,
                assert: true,
                description: "Another valid string enum variant",
            },
            Tests {
                value: r#"{"name":"example","priority":10,"status":"active","enabled":true}"#,
                assert: true,
                description: "Valid with optional boolean and string enums",
            },
            Tests {
                value: r#"{"name":"example","priority":1,"category":2.5}"#,
                assert: true,
                description: "Valid with optional number enum",
            },
            Tests {
                value: r#"{"name":"example","priority":1,"mixed_type":"text"}"#,
                assert: true,
                description: "Valid with mixed type enum (string)",
            },
            Tests {
                value: r#"{"name":"example","priority":1,"mixed_type":42}"#,
                assert: true,
                description: "Valid with mixed type enum (number)",
            },
            Tests {
                value: r#"{"name":"example-103","priority":1}"#,
                assert: false,
                description: "Invalid string enum value",
            },
            Tests {
                value: r#"{"name":"example","priority":5}"#,
                assert: false,
                description: "Invalid integer enum value",
            },
            Tests {
                value: r#"{"name":"example","priority":1,"status":"running"}"#,
                assert: false,
                description: "Invalid status enum value",
            },
            Tests {
                value: r#"{"name":"example","priority":1,"enabled":"yes"}"#,
                assert: false,
                description: "Invalid boolean enum value (string instead of boolean)",
            },
            Tests {
                value: r#"{"name":"example","priority":1,"category":5.5}"#,
                assert: false,
                description: "Invalid number enum value",
            },
            Tests {
                value: r#"{"name":"example","priority":1,"mixed_type":"invalid"}"#,
                assert: false,
                description: "Invalid mixed type enum value",
            },
            Tests {
                value: r#"{"name":"example","priority":1,"mixed_type":100}"#,
                assert: false,
                description: "Invalid mixed type enum number value",
            },
            Tests {
                value: r#"[{"name":"example"}]"#,
                assert: false,
                description: "Invalid JSON structure (array instead of object)",
            },
            Tests {
                value: r#"{"name":"example"}"#,
                assert: false,
                description: "Missing required priority field",
            },
            Tests {
                value: r#"{"priority":1}"#,
                assert: false,
                description: "Missing required name field",
            },
        ];

        for test in tests {
            let result = openapi.validator(make_request_body_with_value(test.value));
            assert_eq!(
                result.is_ok(),
                test.assert,
                "Test failed: {} - Expected: {}, Got: {:?}",
                test.description,
                test.assert,
                result
            );
        }
    }

    #[test]
    fn test_pattern_validation() {
        let content = r#"
openapi: 3.1.0
info:
  title: Pattern Validation Test API
  description: API for testing pattern validation
  version: '1.0.0'

components:
  schemas:
    UserRequest:
      type: object
      properties:
        email:
          type: string
          pattern: '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
          description: User email address
        phone:
          type: string
          pattern: '^\+?1?[-.\s]?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}$'
          description: User phone number
        username:
          type: string
          pattern: '^[a-zA-Z0-9_]{3,20}$'
          description: Username with alphanumeric and underscore only
      required:
        - email
        - username

paths:
  /users:
    post:
      parameters:
        - name: userId
          in: query
          required: true
          schema:
            type: string
            pattern: '^[0-9]+$'
            description: Numeric user ID
        - name: token
          in: query
          required: false
          schema:
            type: string
            pattern: '^[A-Za-z0-9]{32}$'
            description: 32-character alphanumeric token
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/UserRequest'
      responses:
        '201':
          description: User created successfully
"#;

        let openapi: OpenAPI = OpenAPI::yaml(content).expect("Failed to parse OpenAPI YAML");

        fn make_request_with_query_and_body(query: &str, body: &str) -> request::axum::RequestData {
            request::axum::RequestData {
                path: "/users".to_string(),
                inner: axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/users?{}", query))
                    .body(axum::body::Body::from(body.to_string()))
                    .unwrap(),
                body: Some(Bytes::from(body.to_string())),
            }
        }

        struct Tests {
            query: &'static str,
            body: &'static str,
            assert: bool,
            description: &'static str,
        }

        let tests: Vec<Tests> = vec![
            Tests {
                query: "userId=12345&token=abc123DEF456ghi789JKL012mno345PQ",
                body: r#"{"email":"test@example.com","username":"valid_user123","phone":"(555) 123-4567"}"#,
                assert: true,
                description: "All valid patterns",
            },
            Tests {
                query: "userId=999",
                body: r#"{"email":"user@domain.org","username":"testuser"}"#,
                assert: true,
                description: "Required fields only with valid patterns",
            },
            Tests {
                query: "userId=abc123",
                body: r#"{"email":"test@example.com","username":"validuser"}"#,
                assert: false,
                description: "Invalid userId pattern (contains letters)",
            },
            Tests {
                query: "userId=123&token=short",
                body: r#"{"email":"test@example.com","username":"validuser"}"#,
                assert: false,
                description: "Invalid token pattern (too short)",
            },
            Tests {
                query: "userId=123",
                body: r#"{"email":"invalid-email","username":"validuser"}"#,
                assert: false,
                description: "Invalid email pattern",
            },
            Tests {
                query: "userId=123",
                body: r#"{"email":"test@example.com","username":"in valid"}"#,
                assert: false,
                description: "Invalid username pattern (contains space)",
            },
            Tests {
                query: "userId=123",
                body: r#"{"email":"test@example.com","username":"ab"}"#,
                assert: false,
                description: "Invalid username pattern (too short)",
            },
            Tests {
                query: "userId=123",
                body: r#"{"email":"test@example.com","username":"validuser","phone":"invalid-phone"}"#,
                assert: false,
                description: "Invalid phone pattern",
            },
        ];

        for test in tests {
            let result = openapi.validator(make_request_with_query_and_body(test.query, test.body));
            assert_eq!(
                result.is_ok(),
                test.assert,
                "Test failed: {} - Expected: {}, Got: {:?}",
                test.description,
                test.assert,
                result
            );
        }
    }
}
