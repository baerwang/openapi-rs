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
    use crate::model::parse::OpenAPI;
    use crate::request;
    use axum::body::Bytes;

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
          description: Get a Example response
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
}
