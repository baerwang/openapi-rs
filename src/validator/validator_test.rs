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

        assert!(
            openapi
                .validator(make_request(
                    "/example/00000000-0000-0000-0000-000000000000"
                ))
                .is_ok(),
            "Valid UUID should pass validation"
        );

        assert!(
            openapi
                .validator(make_request("/example/00000000"))
                .is_err(),
            "Invalid UUID should fail validation"
        );
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

        assert!(
            openapi
                .validator(make_request("00000000-0000-0000-0000-000000000000"))
                .is_ok(),
            "Valid body should pass validation"
        );

        assert!(
            !openapi
                .validator(make_request("00000000-0000-0000-0000-xxxx"))
                .is_ok(),
            "Valid body should pass validation"
        );
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

        fn make_request(value: &str) -> request::axum::RequestData {
            request::axum::RequestData {
                path: "/example".to_string(),
                inner: axum::http::Request::builder()
                    .method("POST")
                    .uri("/example")
                    .body(axum::body::Body::from(format!(
                        "{{\"uuid\":\"{}\"}}",
                        value
                    )))
                    .unwrap(),
                body: Some(Bytes::from(format!("{{\"uuid\":\"{}\"}}", value))),
            }
        }

        assert!(
            openapi
                .validator(make_request("00000000-0000-0000-0000-000000000000"))
                .is_ok(),
            "Valid body should pass validation"
        );

        assert!(
            !openapi
                .validator(make_request("00000000-0000-0000-0000-xxxx"))
                .is_ok(),
            "Valid body should pass validation"
        );
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
    fn format_types() {
        fn t(v: &str, format: Format) -> bool {
            validate_field_format("", &Value::from(v), Some(format)).is_ok()
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
}
