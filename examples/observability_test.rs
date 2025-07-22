use anyhow::Result;
use openapi_rs::model::parse::OpenAPI;
use openapi_rs::observability::init_logger;
use openapi_rs::request;

fn main() -> Result<()> {
    init_logger();

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

    println!("✅ Testing successful case");
    match openapi.validator(make_request(
        "/example/00000000-0000-0000-0000-000000000000",
    )) {
        Ok(_) => println!("✓ Validation successful"),
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    println!("❌ Testing failure case");
    match openapi.validator(make_request(
        "/example/00000000-0000-0000-0000-00000000000x",
    )) {
        Ok(_) => println!("✓ Validation successful"),
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    println!("🎉 Testing completed");
    Ok(())
}
