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
    use openapi_rs::model::parse::{Format, In, OpenAPI, Type, TypeOrUnion};
    use serde_yaml::Value;
    use serde_yaml::Value::Sequence;
    use std::env;

    #[test]
    fn parse_example() -> Result<(), Box<dyn std::error::Error>> {
        let content =
            std::fs::read_to_string(env::current_dir()?.join("tests/example/example.yaml"))?;

        let openapi: OpenAPI = OpenAPI::yaml(&content)?;

        // Validate general OpenAPI properties
        assert_eq!(openapi.openapi, "3.1.0");
        assert_eq!(openapi.info.title, "Example API");
        assert!(openapi.components.is_some());

        let components = openapi.components.as_ref().unwrap();

        // Validate schemas' presence of "oneOf" and "allOf"
        let schemas_check = [("ExampleRequest", false), ("ExampleResponse", false)];
        for (schema_name, expected_one_of) in schemas_check.iter() {
            let schema = components
                .schemas
                .get(*schema_name)
                .ok_or(format!("Missing schema: {}", *schema_name))?;
            assert_eq!(schema.one_of.is_some(), *expected_one_of);
        }

        let schemas_check_all_of = [("ExampleRequest", false), ("ExampleResponse", true)];
        for (schema_name, expected_all_of) in schemas_check_all_of.iter() {
            let schema = components
                .schemas
                .get(*schema_name)
                .ok_or(format!("Missing schema: {}", *schema_name))?;
            assert_eq!(schema.all_of.is_some(), *expected_all_of);
        }

        // Validate paths
        let example_path = openapi
            .paths
            .get("/example/{uuid}")
            .ok_or("Missing path: /example/{uuid}")?;
        let get_value = example_path
            .operations
            .get("get")
            .ok_or("Missing GET method for /example/{uuid}")?;

        // Validate GET parameters
        let parameter = get_value
            .parameters
            .as_ref()
            .and_then(|params| params.first())
            .ok_or("Missing parameter")?;

        // Since Parameter is now a struct, access fields directly
        assert_eq!(parameter.name.as_deref(), Some("uuid"));
        assert_eq!(
            parameter.description.as_deref(),
            Some("The UUID for this example.")
        );
        assert_eq!(parameter.r#in.as_ref(), Some(&In::Path));
        if let Some(schema) = &parameter.schema {
            assert_eq!(schema.r#type, Some(TypeOrUnion::Single(Type::String)));
            assert_eq!(schema.format, Some(Format::UUID));
            assert_eq!(
                schema.example.clone().unwrap(),
                "00000000-0000-0000-0000-000000000000"
            );
            assert!(schema.examples.is_none());
        }

        Ok(())
    }

    #[test]
    fn parse_components_base() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.1.0
info:
  title: Example API
  description: API definitions for example
  version: '0.0.1'

components:
  schemas:
    ExampleRequest:
      title: example request
      description: example description
      type: object
      properties:
        result:
          type: string
          description: example
          example: example
      required:
        - result
paths:
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        // Validate general properties
        assert_eq!(openapi.openapi, "3.1.0");
        assert_eq!(openapi.info.title, "Example API");
        assert_eq!(
            openapi.info.description.as_deref(),
            Some("API definitions for example")
        );
        assert_eq!(openapi.info.version, "0.0.1");

        // Validate components and schemas
        let components = openapi.components.as_ref().unwrap();
        let example_request = components.schemas.get("ExampleRequest").unwrap();

        assert!(example_request.one_of.is_none());
        assert!(example_request.all_of.is_none());

        // Validate "ExampleRequest" properties
        assert_eq!(example_request.title.as_ref().unwrap(), "example request");
        assert_eq!(
            example_request.description.as_ref().unwrap(),
            "example description"
        );
        assert!(!example_request.required.is_empty());

        // Validate "result" property in ExampleRequest
        let result = example_request
            .properties
            .as_ref()
            .unwrap()
            .get("result")
            .unwrap();
        assert_eq!(result.r#type, Some(TypeOrUnion::Single(Type::String)));
        assert_eq!(result.minimum, None);
        assert_eq!(result.maximum, None);
        assert_eq!(result.example.clone().unwrap(), "example");

        Ok(())
    }

    #[test]
    fn parse_components_all_of() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.1.0
info:
  title: Example API
  description: API definitions for example
  version: "0.0.1"

components:
  schemas:
    ExampleResponse:
      allOf:
        - type: object
          properties:
            result:
              type: object
              description: example.
              properties:
                uuid:
                  type: string
                  description: The UUID for this example.
                  format: uuid
                  example: 00000000-0000-0000-0000-000000000000
                count:
                  type: integer
                  description: example count.
                  example: 1
                  maximum: 1
              required:
                - uuid
paths:
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        // Validate general properties
        assert_eq!(openapi.openapi, "3.1.0");
        assert_eq!(openapi.info.title, "Example API");
        assert_eq!(
            openapi.info.description.as_deref(),
            Some("API definitions for example")
        );
        assert_eq!(openapi.info.version, "0.0.1");

        // Validate components and schemas
        let components = openapi.components.as_ref().ok_or("Missing components")?;
        let example_response = components
            .schemas
            .get("ExampleResponse")
            .ok_or("Missing ExampleResponse schema")?;

        // Assert "allOf" exists and validate it
        let all_of = example_response
            .all_of
            .as_ref()
            .ok_or("Missing allOf in ExampleResponse")?;
        let first = &all_of[0];

        // Validate "allOf" object
        assert_eq!(first.r#type, Some(TypeOrUnion::Single(Type::Object)));
        assert!(first.description.is_none());

        // Validate "result" object properties
        let result = first
            .properties
            .get("result")
            .ok_or("Missing result property")?;
        assert_eq!(result.r#type, Some(TypeOrUnion::Single(Type::Object)));
        assert_eq!(result.description.as_deref(), Some("example."));
        assert!(!result.required.is_empty());

        // Validate "uuid" property in result
        let uuid = result
            .properties
            .as_ref()
            .ok_or("Missing properties in result")?
            .get("uuid")
            .ok_or("Missing uuid")?;
        assert_eq!(uuid.r#type, Some(TypeOrUnion::Single(Type::String)));
        assert_eq!(
            uuid.description.as_deref(),
            Some("The UUID for this example.")
        );
        assert_eq!(uuid.format, Some(Format::UUID));
        assert_eq!(
            uuid.example.clone().unwrap(),
            "00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(uuid.minimum, None);
        assert_eq!(uuid.maximum, None);

        // Validate "count" property in result
        let count = result
            .properties
            .as_ref()
            .ok_or("Missing properties in result")?
            .get("count")
            .ok_or("Missing count")?;
        assert_eq!(count.r#type, Some(TypeOrUnion::Single(Type::Integer)));
        assert_eq!(count.description.as_deref(), Some("example count."));
        assert_eq!(count.format, None);
        assert_eq!(count.example.clone().unwrap(), 1);
        assert_eq!(count.minimum, None);
        assert_eq!(count.maximum, Some(1.0));

        Ok(())
    }

    #[test]
    fn parse_components_one_of() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.1.0
info:
  title: Example API
  description: API definitions for example
  version: '0.0.1'

components:
  schemas:
    ExampleResponse:
      oneOf:
        - type: object
          properties:
            result:
              type: object
              description: example.
              properties:
                uuid:
                  type: string
                  description: The UUID for this example.
                  format: uuid
                  example: 00000000-0000-0000-0000-000000000000
paths:
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        // Validate general properties
        assert_eq!(openapi.openapi, "3.1.0");
        assert_eq!(openapi.info.title, "Example API");
        assert_eq!(
            openapi.info.description.as_deref(),
            Some("API definitions for example")
        );
        assert_eq!(openapi.info.version, "0.0.1");

        // Validate components and schemas
        let components = openapi.components.as_ref().ok_or("Missing components")?;
        let example_response = components
            .schemas
            .get("ExampleResponse")
            .ok_or("Missing ExampleResponse schema")?;

        assert!(example_response.one_of.is_some());

        // Validate "oneOf"
        let one_of = &example_response.one_of.as_ref().unwrap()[0];

        // Validate "oneOf" object
        assert_eq!(one_of.r#type, Some(TypeOrUnion::Single(Type::Object)));
        assert!(one_of.description.is_none());

        // Validate "result" object properties

        let result = one_of
            .properties
            .get("result")
            .ok_or("Missing result property")?;
        assert_eq!(result.r#type, Some(TypeOrUnion::Single(Type::Object)));
        assert_eq!(result.description.as_deref(), Some("example."));
        assert!(result.required.is_empty());

        // Validate "uuid" property in result
        let uuid = result
            .properties
            .as_ref()
            .ok_or("Missing properties in result")?
            .get("uuid")
            .ok_or("Missing uuid")?;
        assert_eq!(uuid.r#type, Some(TypeOrUnion::Single(Type::String)));
        assert_eq!(
            uuid.description.as_deref(),
            Some("The UUID for this example.")
        );
        assert_eq!(uuid.format, Some(Format::UUID));
        assert_eq!(
            uuid.example.clone().unwrap(),
            "00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(uuid.minimum, None);
        assert_eq!(uuid.maximum, None);

        Ok(())
    }

    #[test]
    fn parse_path_response_one_of() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.1.0
info:
  title: Example API
  description: API definitions for example
  version: '0.0.1'

paths:

  /example:
    get:
      responses:
        200:
          description: OK
          content:
            application/json:
              schema:
                oneOf:
                  - $ref: '#/components/schemas/Cat'
                  - $ref: '#/components/schemas/Dog'
                  - $ref: '#/components/schemas/Hamster'
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        // Validate general properties
        assert_eq!(openapi.openapi, "3.1.0");
        assert_eq!(openapi.info.title, "Example API");
        assert_eq!(
            openapi.info.description.as_deref(),
            Some("API definitions for example")
        );
        assert_eq!(openapi.info.version, "0.0.1");
        assert!(openapi.components.is_none());

        // Validate paths
        let example_path = openapi
            .paths
            .get("/example")
            .ok_or("Missing path: /example")?;

        let _ = example_path
            .operations
            .get("get")
            .ok_or("Missing GET method for /example")?;

        Ok(())
    }

    #[test]
    fn parse_field_of_example() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.1.0
info:
  title: Example API
  description: API definitions for example
  version: '0.0.1'

components:
  schemas:
    ExampleResponse:
      type: object
      properties:
        uuid:
          type: string
          description: The UUID for this example.
          format: uuid
          example: 00000000-0000-0000-0000-000000000000
        multi_uuid:
          type: array
          description: The Multi UUID for this example.
          format: uuid
          example:
            - 00000000-0000-0000-0000-000000000000
            - 00000000-0000-0000-0000-000000000001
            - 00000000-0000-0000-0000-000000000002
paths:
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        // Validate general properties
        assert_eq!(openapi.openapi, "3.1.0");
        assert_eq!(openapi.info.title, "Example API");
        assert_eq!(
            openapi.info.description.as_deref(),
            Some("API definitions for example")
        );
        assert_eq!(openapi.info.version, "0.0.1");

        // Validate components and schemas
        let components = openapi.components.as_ref().unwrap();
        let example_response = components.schemas.get("ExampleResponse").unwrap();
        let properties = example_response.properties.as_ref().unwrap();

        // Validate uuid property
        let uuid = properties.get("uuid").unwrap();
        assert_eq!(uuid.r#type, Some(TypeOrUnion::Single(Type::String)));
        assert_eq!(
            uuid.description.as_deref(),
            Some("The UUID for this example.")
        );
        assert_eq!(uuid.format, Some(Format::UUID));
        assert_eq!(
            uuid.example.clone().unwrap(),
            "00000000-0000-0000-0000-000000000000"
        );

        // Validate multi_uuid property
        let multi_uuid = properties.get("multi_uuid").unwrap();
        assert_eq!(multi_uuid.r#type, Some(TypeOrUnion::Single(Type::Array)));
        assert_eq!(
            multi_uuid.description.as_deref(),
            Some("The Multi UUID for this example.")
        );
        assert_eq!(multi_uuid.format, Some(Format::UUID));

        // Check example value
        assert_eq!(
            multi_uuid.example,
            Some(Sequence(vec![
                Value::String("00000000-0000-0000-0000-000000000000".to_owned()),
                Value::String("00000000-0000-0000-0000-000000000001".to_owned()),
                Value::String("00000000-0000-0000-0000-000000000002".to_owned()),
            ]))
        );

        Ok(())
    }

    // ==================== OpenAPI 3.1 Tests ====================

    #[test]
    fn parse_openapi_31_with_webhooks() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.1.0
info:
  title: Webhook API
  description: API with webhooks
  version: '1.0.0'
webhooks:
  newEvent:
    post:
      summary: New webhook event
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                id:
                  type: string
      responses:
        '200':
          description: Success
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        assert_eq!(openapi.openapi, "3.1.0");
        assert_eq!(openapi.info.title, "Webhook API");
        assert!(openapi.webhooks.is_some());

        let webhooks = openapi.webhooks.as_ref().unwrap();
        assert!(webhooks.contains_key("newEvent"));

        Ok(())
    }

    #[test]
    fn parse_openapi_31_with_json_schema_dialect() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.1.0
jsonSchemaDialect: https://spec.openapis.org/oas/3.1/dialect/base
info:
  title: Schema Dialect API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        assert_eq!(openapi.openapi, "3.1.0");
        assert_eq!(
            openapi.json_schema_dialect.as_ref().unwrap(),
            "https://spec.openapis.org/oas/3.1/dialect/base"
        );

        Ok(())
    }

    #[test]
    fn version_detection_31() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.1.0
info:
  title: Test API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        assert!(openapi.is_31());
        assert!(!openapi.is_32());

        Ok(())
    }

    // ==================== OpenAPI 3.2 Tests ====================

    #[test]
    fn parse_openapi_32_with_self_field() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.2.0
$self: https://api.example.com
info:
  title: Self Reference API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        assert_eq!(openapi.openapi, "3.2.0");
        assert_eq!(
            openapi.self_ref.as_ref().unwrap(),
            "https://api.example.com"
        );

        Ok(())
    }

    #[test]
    fn parse_openapi_32_with_info_summary() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.2.0
info:
  title: Summary API
  summary: A short summary of the API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        assert_eq!(openapi.openapi, "3.2.0");
        assert_eq!(
            openapi.info.summary.as_ref().unwrap(),
            "A short summary of the API"
        );

        Ok(())
    }

    #[test]
    fn parse_openapi_32_with_query_method() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.2.0
info:
  title: Query Method API
  version: '1.0.0'
paths:
  /users:
    query:
      summary: Query users
      description: Execute a query on users
      operationId: queryUsers
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                filters:
                  type: array
                  items:
                    type: string
      responses:
        '200':
          description: Query results
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        assert_eq!(openapi.openapi, "3.2.0");

        let users_path = openapi.paths.get("/users").unwrap();
        assert!(users_path.query.is_some());

        let query_op = users_path.query.as_ref().unwrap();
        assert_eq!(query_op.summary.as_ref().unwrap(), "Query users");
        assert_eq!(query_op.operation_id.as_ref().unwrap(), "queryUsers");

        Ok(())
    }

    #[test]
    fn parse_openapi_32_with_querystring_parameter() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.2.0
info:
  title: QueryString Parameter API
  version: '1.0.0'
paths:
  /search:
    get:
      summary: Search with querystring
      parameters:
        - name: filters
          in: querystring
          description: Query string filters
          required: true
          content:
            application/json:
              schema:
                type: object
                properties:
                  name:
                    type: string
                  status:
                    type: string
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        assert_eq!(openapi.openapi, "3.2.0");

        let search_path = openapi.paths.get("/search").unwrap();
        let get_op = search_path.operations.get("get").unwrap();

        let params = get_op.parameters.as_ref().unwrap();
        let filter_param = &params[0];

        assert_eq!(filter_param.name.as_ref().unwrap(), "filters");
        assert_eq!(filter_param.r#in.as_ref().unwrap(), &In::QueryString);

        Ok(())
    }

    #[test]
    fn version_detection_32() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.2.0
info:
  title: Test API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        assert!(!openapi.is_31());
        assert!(openapi.is_32());

        Ok(())
    }

    // ==================== Backward Compatibility Tests ====================

    #[test]
    fn backward_compat_31_works_with_32_fields() -> Result<(), Box<dyn std::error::Error>> {
        // 3.1 spec should parse fine even when 3.2 fields are present (as optional)
        let content = r#"
openapi: 3.1.0
info:
  title: Test API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        // 3.1 fields work
        assert!(openapi.is_31());

        // 3.2 fields are None
        assert!(openapi.self_ref.is_none());
        assert!(openapi.info.summary.is_none());

        Ok(())
    }

    #[test]
    fn backward_compat_32_parses_31_spec() -> Result<(), Box<dyn std::error::Error>> {
        // 3.2 support should be able to parse 3.1 specs
        let content = r#"
openapi: 3.1.0
info:
  title: Test API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        assert_eq!(openapi.openapi, "3.1.0");
        assert_eq!(openapi.info.title, "Test API");

        Ok(())
    }

    #[test]
    fn parse_all_3_1_and_3_2_fields_together() -> Result<(), Box<dyn std::error::Error>> {
        // Full spec with all 3.1 and 3.2 fields
        let content = r#"
openapi: 3.2.0
$self: https://api.example.com/v2
jsonSchemaDialect: https://spec.openapis.org/oas/3.2/dialect/2025-09-17
info:
  title: Complete API
  summary: A complete API with all fields
  description: API description
  version: '2.0.0'
webhooks:
  newEvent:
    post:
      summary: New event
      responses:
        '200':
          description: OK
paths:
  /users:
    get:
      summary: List users
      responses:
        '200':
          description: OK
    query:
      summary: Query users
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        // Version detection
        assert!(openapi.is_32());

        // 3.1 fields
        assert!(openapi.json_schema_dialect.is_some());
        assert!(openapi.webhooks.is_some());

        // 3.2 fields
        assert!(openapi.self_ref.is_some());
        assert!(openapi.info.summary.is_some());

        // Both HTTP methods
        let users_path = openapi.paths.get("/users").unwrap();
        assert!(users_path.operations.get("get").is_some());
        assert!(users_path.query.is_some());

        Ok(())
    }

    // ==================== Complex Webhooks Tests ====================

    #[test]
    fn webhooks_with_multiple_operations() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.1.0
info:
  title: Multi-Webhook API
  version: '1.0.0'
webhooks:
  userCreated:
    post:
      summary: User created webhook
      operationId: userCreated
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                id:
                  type: string
                email:
                  type: string
                  format: email
      responses:
        '200':
          description: Webhook processed
        '400':
          description: Invalid payload
  userUpdated:
    put:
      summary: User updated webhook
      operationId: userUpdated
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                id:
                  type: string
                changes:
                  type: array
                  items:
                    type: string
      responses:
        '200':
          description: Update processed
  userDeleted:
    delete:
      summary: User deleted webhook
      parameters:
        - name: userId
          in: query
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Delete processed
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        assert!(openapi.webhooks.is_some());
        let webhooks = openapi.webhooks.as_ref().unwrap();

        // Verify all three webhooks exist
        assert_eq!(webhooks.len(), 3);
        assert!(webhooks.contains_key("userCreated"));
        assert!(webhooks.contains_key("userUpdated"));
        assert!(webhooks.contains_key("userDeleted"));

        // Verify userCreated webhook structure
        let user_created = webhooks.get("userCreated").unwrap();
        let post_op = user_created.operations.get("post").unwrap();
        assert_eq!(post_op.summary.as_ref().unwrap(), "User created webhook");
        assert_eq!(post_op.operation_id.as_ref().unwrap(), "userCreated");
        assert!(post_op.request.is_some());

        // Verify userUpdated webhook structure
        let user_updated = webhooks.get("userUpdated").unwrap();
        let put_op = user_updated.operations.get("put").unwrap();
        assert_eq!(put_op.summary.as_ref().unwrap(), "User updated webhook");

        // Verify userDeleted webhook structure
        let user_deleted = webhooks.get("userDeleted").unwrap();
        let delete_op = user_deleted.operations.get("delete").unwrap();
        assert_eq!(delete_op.summary.as_ref().unwrap(), "User deleted webhook");

        Ok(())
    }

    #[test]
    fn webhooks_with_parameters_and_security() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.1.0
info:
  title: Secure Webhook API
  version: '1.0.0'
webhooks:
  paymentEvent:
    post:
      summary: Payment webhook with authentication
      parameters:
        - name: X-Webhook-Signature
          in: header
          required: true
          description: HMAC signature for verification
          schema:
            type: string
        - name: event_id
          in: query
          required: true
          schema:
            type: string
            format: uuid
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                payment_id:
                  type: string
                amount:
                  type: number
                  format: float
                currency:
                  type: string
              required:
                - payment_id
                - amount
      responses:
        '200':
          description: Event processed successfully
        '401':
          description: Invalid signature
        '400':
          description: Invalid event data
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        let webhooks = openapi.webhooks.as_ref().unwrap();
        let payment_webhook = webhooks.get("paymentEvent").unwrap();
        let post_op = payment_webhook.operations.get("post").unwrap();

        // Verify parameters
        let params = post_op.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 2);

        let sig_param = &params[0];
        assert_eq!(sig_param.name.as_ref().unwrap(), "X-Webhook-Signature");
        assert_eq!(sig_param.r#in.as_ref().unwrap(), &In::Header);
        assert!(sig_param.required);

        let event_param = &params[1];
        assert_eq!(event_param.name.as_ref().unwrap(), "event_id");
        assert_eq!(event_param.r#in.as_ref().unwrap(), &In::Query);
        assert!(event_param.required);

        Ok(())
    }

    // ==================== JSON Schema Dialect Tests ====================

    #[test]
    fn json_schema_dialect_variations() -> Result<(), Box<dyn std::error::Error>> {
        // Test different JSON Schema Dialect values
        let test_cases = vec![
            (
                "https://spec.openapis.org/oas/3.1/dialect/base",
                "3.1.0",
                "OAS 3.1 base dialect",
            ),
            (
                "https://json-schema.org/draft/2020-12/schema",
                "3.1.0",
                "JSON Schema 2020-12",
            ),
            (
                "https://json-schema.org/draft/2019-09/schema",
                "3.1.0",
                "JSON Schema 2019-09",
            ),
            (
                "https://spec.openapis.org/oas/3.2/dialect/2025-09-17",
                "3.2.0",
                "OAS 3.2 dialect",
            ),
            (
                "http://custom-schema.example.com/dialect/v1",
                "3.1.0",
                "Custom dialect",
            ),
        ];

        for (dialect, version, desc) in test_cases {
            let content = format!(
                r#"
openapi: {}
jsonSchemaDialect: {}
info:
  title: Test API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#,
                version, dialect
            );

            let openapi: OpenAPI = OpenAPI::yaml(&content)?;
            assert_eq!(
                openapi.json_schema_dialect.as_ref().unwrap(),
                dialect,
                "{}: dialect mismatch",
                desc
            );
        }

        Ok(())
    }

    #[test]
    fn json_schema_dialect_optional_field() -> Result<(), Box<dyn std::error::Error>> {
        // Verify jsonSchemaDialect is truly optional
        let content = r#"
openapi: 3.1.0
info:
  title: Test API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;
        assert!(openapi.json_schema_dialect.is_none());

        Ok(())
    }

    // ==================== $self Field Tests ====================

    #[test]
    fn self_field_variations() -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = vec![
            ("https://api.example.com", "Absolute HTTPS URL"),
            ("http://api.example.com", "Absolute HTTP URL"),
            ("https://api.example.com/v1", "URL with path"),
            ("https://api.example.com:8080", "URL with port"),
            ("https://api.example.com/v2/api", "URL with nested path"),
            ("/v1/api", "Relative path"),
        ];

        for (self_value, desc) in test_cases {
            let content = format!(
                r#"
openapi: 3.2.0
$self: {}
info:
  title: Test API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#,
                self_value
            );

            let openapi: OpenAPI = OpenAPI::yaml(&content)?;
            assert_eq!(
                openapi.self_ref.as_ref().unwrap(),
                self_value,
                "{}: $self mismatch",
                desc
            );
        }

        Ok(())
    }

    #[test]
    fn self_field_optional() -> Result<(), Box<dyn std::error::Error>> {
        // Verify $self is optional in 3.2
        let content = r#"
openapi: 3.2.0
info:
  title: Test API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;
        assert!(openapi.self_ref.is_none());

        Ok(())
    }

    // ==================== info.summary Tests ====================

    #[test]
    fn info_summary_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = vec![
            ("A", "Single character"),
            ("API", "Single word"),
            ("A short API", "Multiple words"),
            (
                "A comprehensive REST API for managing users, products, and orders",
                "Long summary",
            ),
            ("API-with-special-chars!@#$%", "Special characters"),
            ("   API with spaces   ", "Leading/trailing spaces"),
        ];

        for (summary, desc) in test_cases {
            let escaped_summary = summary.replace('\\', "\\\\").replace('"', "\\\"");
            let content = format!(
                r#"
openapi: 3.2.0
info:
  title: Test API
  summary: "{}"
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#,
                escaped_summary
            );

            let openapi: OpenAPI = OpenAPI::yaml(&content)?;
            assert_eq!(
                openapi.info.summary.as_ref().unwrap(),
                summary,
                "{}: summary mismatch",
                desc
            );
        }

        Ok(())
    }

    #[test]
    fn info_summary_optional_in_32() -> Result<(), Box<dyn std::error::Error>> {
        // Verify summary is optional in 3.2
        let content = r#"
openapi: 3.2.0
info:
  title: Test API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;
        assert!(openapi.info.summary.is_none());

        Ok(())
    }

    // ==================== QUERY Method Tests ====================

    #[test]
    fn query_method_with_full_operation_details() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.2.0
info:
  title: QUERY Method API
  version: '1.0.0'
paths:
  /users:
    query:
      summary: Execute complex user queries
      description: |
        Allows execution of complex queries on the user collection.
        Supports filtering, sorting, and pagination.
      operationId: queryUsers
      tags:
        - users
        - queries
      parameters:
        - name: includeDeleted
          in: query
          description: Include deleted users in results
          required: false
          schema:
            type: boolean
            default: false
      requestBody:
        required: true
        description: Query DSL object
        content:
          application/json:
            schema:
              type: object
              properties:
                filter:
                  type: object
                  properties:
                    status:
                      type: string
                      enum: [active, inactive, pending]
                    role:
                      type: string
                sort:
                  type: array
                  items:
                    type: object
                    properties:
                      field:
                        type: string
                      order:
                        type: string
                        enum: [asc, desc]
                pagination:
                  type: object
                  properties:
                    offset:
                      type: integer
                      minimum: 0
                      default: 0
                    limit:
                      type: integer
                      minimum: 1
                      maximum: 100
                      default: 20
              required:
                - filter
      responses:
        '200':
          description: Query results
          content:
            application/json:
              schema:
                type: object
                properties:
                  data:
                    type: array
                    items:
                      $ref: '#/components/schemas/User'
                  total:
                    type: integer
                  offset:
                    type: integer
                  limit:
                    type: integer
        '400':
          description: Invalid query
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
        '401':
          description: Unauthorized
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
        name:
          type: string
    Error:
      type: object
      properties:
        code:
          type: string
        message:
          type: string
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        let users_path = openapi.paths.get("/users").unwrap();
        assert!(users_path.query.is_some());

        let query_op = users_path.query.as_ref().unwrap();

        // Verify all operation fields
        assert_eq!(
            query_op.summary.as_ref().unwrap(),
            "Execute complex user queries"
        );
        assert!(query_op.description.is_some());
        assert_eq!(query_op.operation_id.as_ref().unwrap(), "queryUsers");

        // Verify parameters
        let params = query_op.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name.as_ref().unwrap(), "includeDeleted");
        assert!(!params[0].required);

        // Verify request body exists
        assert!(query_op.request.is_some());

        // Verify responses through the PathBase structure
        assert!(query_op.request.is_some());

        Ok(())
    }

    #[test]
    fn query_method_alongside_traditional_methods() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.2.0
info:
  title: Full CRUD API
  version: '1.0.0'
paths:
  /users:
    get:
      summary: List users
      operationId: listUsers
      parameters:
        - name: limit
          in: query
          schema:
            type: integer
      responses:
        '200':
          description: OK
    post:
      summary: Create user
      operationId: createUser
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
      responses:
        '201':
          description: Created
    query:
      summary: Query users
      operationId: queryUsers
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
      responses:
        '200':
          description: Query results
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;
        let users_path = openapi.paths.get("/users").unwrap();

        // All three methods should coexist
        assert!(users_path.operations.get("get").is_some());
        assert!(users_path.operations.get("post").is_some());
        assert!(users_path.query.is_some());

        // Verify each has correct operationId
        assert_eq!(
            users_path
                .operations
                .get("get")
                .unwrap()
                .operation_id
                .as_ref()
                .unwrap(),
            "listUsers"
        );
        assert_eq!(
            users_path
                .operations
                .get("post")
                .unwrap()
                .operation_id
                .as_ref()
                .unwrap(),
            "createUser"
        );
        assert_eq!(
            users_path
                .query
                .as_ref()
                .unwrap()
                .operation_id
                .as_ref()
                .unwrap(),
            "queryUsers"
        );

        Ok(())
    }

    // ==================== QueryString Parameter Tests ====================

    #[test]
    fn querystring_parameter_with_complex_schema() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
openapi: 3.2.0
info:
  title: Complex QueryString API
  version: '1.0.0'
paths:
  /search:
    get:
      summary: Advanced search
      parameters:
        - name: filters
          in: querystring
          description: Complex filter object as query string
          required: true
          content:
            application/json:
              schema:
                type: object
                properties:
                  name:
                    type: string
                    description: Filter by name
                    minLength: 1
                    maxLength: 100
                  email:
                    type: string
                    format: email
                    description: Filter by email
                  age_min:
                    type: integer
                    minimum: 0
                    maximum: 150
                  age_max:
                    type: integer
                    minimum: 0
                    maximum: 150
                  status:
                    type: string
                    enum: [active, inactive, pending, suspended]
                  tags:
                    type: array
                    items:
                      type: string
                  created_after:
                    type: string
                    format: date-time
                  coordinates:
                    type: object
                    properties:
                      lat:
                        type: number
                        format: float
                        minimum: -90
                        maximum: 90
                      lng:
                        type: number
                        format: float
                        minimum: -180
                        maximum: 180
              required:
                - name
        - name: sort
          in: querystring
          description: Sort criteria
          required: false
          content:
            application/json:
              schema:
                type: array
                items:
                  type: object
                  properties:
                    field:
                      type: string
                    direction:
                      type: string
                      enum: [asc, desc]
        - name: pagination
          in: querystring
          description: Pagination settings
          required: false
          content:
            application/json:
              schema:
                type: object
                properties:
                  offset:
                    type: integer
                    minimum: 0
                    default: 0
                  limit:
                    type: integer
                    minimum: 1
                    maximum: 100
                    default: 20
      responses:
        '200':
          description: Search results
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;
        let search_path = openapi.paths.get("/search").unwrap();
        let get_op = search_path.operations.get("get").unwrap();

        let params = get_op.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 3);

        // Verify first querystring parameter (filters)
        let filters_param = &params[0];
        assert_eq!(filters_param.name.as_ref().unwrap(), "filters");
        assert_eq!(filters_param.r#in.as_ref().unwrap(), &In::QueryString);
        assert!(filters_param.required);

        // Verify second querystring parameter (sort)
        let sort_param = &params[1];
        assert_eq!(sort_param.name.as_ref().unwrap(), "sort");
        assert_eq!(sort_param.r#in.as_ref().unwrap(), &In::QueryString);
        assert!(!sort_param.required);

        // Verify third querystring parameter (pagination)
        let pagination_param = &params[2];
        assert_eq!(pagination_param.name.as_ref().unwrap(), "pagination");
        assert_eq!(pagination_param.r#in.as_ref().unwrap(), &In::QueryString);
        assert!(!pagination_param.required);

        Ok(())
    }

    #[test]
    fn querystring_vs_regular_query_parameters() -> Result<(), Box<dyn std::error::Error>> {
        // Test that both query and querystring can coexist
        let content = r#"
openapi: 3.2.0
info:
  title: Mixed Parameter Types API
  version: '1.0.0'
paths:
  /search:
    get:
      summary: Search with mixed parameter types
      parameters:
        - name: format
          in: query
          description: Response format
          schema:
            type: string
            enum: [json, xml, csv]
            default: json
        - name: filters
          in: querystring
          description: Complex filter object
          required: false
          content:
            application/json:
              schema:
                type: object
                properties:
                  category:
                    type: string
                  price_range:
                    type: object
                    properties:
                      min:
                        type: number
                      max:
                        type: number
        - name: debug
          in: query
          description: Enable debug mode
          schema:
            type: boolean
            default: false
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;
        let search_path = openapi.paths.get("/search").unwrap();
        let get_op = search_path.operations.get("get").unwrap();

        let params = get_op.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 3);

        // First is regular query
        assert_eq!(params[0].name.as_ref().unwrap(), "format");
        assert_eq!(params[0].r#in.as_ref().unwrap(), &In::Query);

        // Second is querystring
        assert_eq!(params[1].name.as_ref().unwrap(), "filters");
        assert_eq!(params[1].r#in.as_ref().unwrap(), &In::QueryString);

        // Third is regular query
        assert_eq!(params[2].name.as_ref().unwrap(), "debug");
        assert_eq!(params[2].r#in.as_ref().unwrap(), &In::Query);

        Ok(())
    }

    // ==================== Version Detection ====================

    #[test]
    fn version_detection_various_versions() -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = vec![
            ("3.1.0", true, false),
            ("3.1.1", true, false),
            ("3.1.2", true, false),
            ("3.2.0", false, true),
            ("3.2.1", false, true),
            ("3.0.0", false, false),
            ("3.0.1", false, false),
        ];

        for (version_str, expected_is_31, expected_is_32) in test_cases {
            let content = format!(
                r#"
openapi: {}
info:
  title: Test API
  version: '1.0.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    "#,
                version_str
            );

            let openapi: OpenAPI = OpenAPI::yaml(&content)?;
            assert_eq!(
                openapi.is_31(),
                expected_is_31,
                "is_31() failed for {}",
                version_str
            );
            assert_eq!(
                openapi.is_32(),
                expected_is_32,
                "is_32() failed for {}",
                version_str
            );
        }

        Ok(())
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn serialization_of_new_fields() -> Result<(), Box<dyn std::error::Error>> {
        use serde_yaml::Value;

        // Test that 3.2 fields serialize correctly
        let content = r#"
openapi: 3.2.0
$self: https://api.example.com
jsonSchemaDialect: https://spec.openapis.org/oas/3.2/dialect/base
info:
  title: Serialization Test
  summary: Test serialization
  version: '1.0.0'
webhooks:
  testWebhook:
    post:
      summary: Test
      responses:
        '200':
          description: OK
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
    query:
      summary: Query test
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        // Serialize back to YAML
        let serialized = serde_yaml::to_value(&openapi)?;
        let serialized_obj = serialized.as_mapping().unwrap();

        // Verify $self field
        assert!(serialized_obj.contains_key(&Value::String("$self".to_string())));
        assert_eq!(
            serialized_obj
                .get(&Value::String("$self".to_string()))
                .unwrap(),
            &Value::String("https://api.example.com".to_string())
        );

        // Verify jsonSchemaDialect
        assert!(serialized_obj.contains_key(&Value::String("jsonSchemaDialect".to_string())));

        // Verify webhooks
        assert!(serialized_obj.contains_key(&Value::String("webhooks".to_string())));

        // Verify info.summary
        let info = serialized_obj
            .get(&Value::String("info".to_string()))
            .unwrap();
        let info_obj = info.as_mapping().unwrap();
        assert!(info_obj.contains_key(&Value::String("summary".to_string())));

        // Verify query method in path item
        let paths = serialized_obj
            .get(&Value::String("paths".to_string()))
            .unwrap();
        let paths_obj = paths.as_mapping().unwrap();
        let test_path = paths_obj.get(&Value::String("/test".to_string())).unwrap();
        let test_obj = test_path.as_mapping().unwrap();
        assert!(test_obj.contains_key(&Value::String("query".to_string())));

        Ok(())
    }

    // ==================== Integration Tests ====================

    #[test]
    fn complex_real_world_api_spec() -> Result<(), Box<dyn std::error::Error>> {
        // A comprehensive real-world-like API spec
        let content = r#"
openapi: 3.2.0
$self: https://api.example.com/v2
jsonSchemaDialect: https://spec.openapis.org/oas/3.2/dialect/base
info:
  title: E-Commerce API
  summary: Complete e-commerce management API
  description: |
    A comprehensive API for managing products, orders, customers, and inventory.
    Supports traditional REST operations and advanced QUERY method for complex searches.
  version: '2.1.0'
  contact:
    name: API Support
    email: support@example.com
servers:
  - url: https://api.example.com/v2
    description: Production server
  - url: https://staging-api.example.com/v2
    description: Staging server
webhooks:
  orderCreated:
    post:
      summary: New order created
      description: Fired when a new order is created
      operationId: orderCreated
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/Order'
      responses:
        '200':
          description: Webhook received
  orderShipped:
    post:
      summary: Order shipped
      description: Fired when an order is shipped
      operationId: orderShipped
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/Order'
      responses:
        '200':
          description: Webhook received
  inventoryLow:
    post:
      summary: Low inventory alert
      description: Fired when product inventory falls below threshold
      operationId: inventoryLow
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                product_id:
                  type: string
                current_stock:
                  type: integer
                threshold:
                  type: integer
      responses:
        '200':
          description: Webhook received
paths:
  /products:
    get:
      summary: List products
      operationId: listProducts
      tags:
        - products
      parameters:
        - name: category
          in: query
          schema:
            type: string
        - name: limit
          in: query
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 20
      responses:
        '200':
          description: List of products
          content:
            application/json:
              schema:
                type: object
                properties:
                  products:
                    type: array
                    items:
                      $ref: '#/components/schemas/Product'
                  total:
                    type: integer
    post:
      summary: Create product
      operationId: createProduct
      tags:
        - products
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ProductCreate'
      responses:
        '201':
          description: Product created
    query:
      summary: Complex product search
      description: Execute complex queries with filtering, sorting, and faceting
      operationId: queryProducts
      tags:
        - products
      parameters:
        - name: include
          in: query
          description: Include related resources
          schema:
            type: array
            items:
              type: string
              enum: [variants, reviews, inventory]
          style: form
          explode: false
      requestBody:
        required: true
        description: Query DSL for complex searches
        content:
          application/json:
            schema:
              type: object
              properties:
                filter:
                  type: object
                  properties:
                    price_range:
                      type: object
                      properties:
                        min:
                          type: number
                        max:
                          type: number
                    categories:
                      type: array
                      items:
                        type: string
                    in_stock:
                      type: boolean
                sort:
                  type: array
                  items:
                    type: object
                    properties:
                      field:
                        type: string
                      direction:
                        type: string
                        enum: [asc, desc]
                facet:
                  type: array
                  items:
                    type: string
                    enum: [category, brand, price_range]
                pagination:
                  type: object
                  properties:
                    offset:
                      type: integer
                    limit:
                      type: integer
      responses:
        '200':
          description: Query results
          content:
            application/json:
              schema:
                type: object
                properties:
                  results:
                    type: array
                    items:
                      $ref: '#/components/schemas/Product'
                  facets:
                    type: object
                  total:
                    type: integer
  /products/{id}:
    get:
      summary: Get product by ID
      operationId: getProduct
      tags:
        - products
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Product details
        '404':
          description: Product not found
    put:
      summary: Update product
      operationId: updateProduct
      tags:
        - products
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ProductUpdate'
      responses:
        '200':
          description: Product updated
        '404':
          description: Product not found
  /orders:
    get:
      summary: List orders
      operationId: listOrders
      tags:
        - orders
      parameters:
        - name: customer_id
          in: query
          schema:
            type: string
            format: uuid
        - name: status
          in: query
          schema:
            type: string
            enum: [pending, processing, shipped, delivered, cancelled]
        - name: from_date
          in: query
          schema:
            type: string
            format: date-time
        - name: to_date
          in: query
          schema:
            type: string
            format: date-time
      responses:
        '200':
          description: Orders list
    query:
      summary: Complex order queries
      description: Query orders with complex criteria
      operationId: queryOrders
      tags:
        - orders
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                filter:
                  type: object
                  properties:
                    customer_email:
                      type: string
                      format: email
                    total_min:
                      type: number
                    total_max:
                      type: number
                    items_count:
                      type: object
                      properties:
                        min:
                          type: integer
                        max:
                          type: integer
                    status_history:
                      type: array
                      items:
                        type: string
      responses:
        '200':
          description: Query results
components:
  schemas:
    Product:
      type: object
      properties:
        id:
          type: string
          format: uuid
        name:
          type: string
        description:
          type: string
        price:
          type: number
          format: float
        category:
          type: string
        stock:
          type: integer
        created_at:
          type: string
          format: date-time
        updated_at:
          type: string
          format: date-time
      required:
        - id
        - name
        - price
    ProductCreate:
      type: object
      properties:
        name:
          type: string
          minLength: 1
          maxLength: 200
        description:
          type: string
        price:
          type: number
          format: float
          minimum: 0
        category:
          type: string
        stock:
          type: integer
          minimum: 0
          default: 0
      required:
        - name
        - price
    ProductUpdate:
      type: object
      properties:
        name:
          type: string
        description:
          type: string
        price:
          type: number
          format: float
        category:
          type: string
        stock:
          type: integer
    Order:
      type: object
      properties:
        id:
          type: string
          format: uuid
        customer_id:
          type: string
          format: uuid
        items:
          type: array
          items:
            type: object
            properties:
              product_id:
                type: string
              quantity:
                type: integer
              price:
                type: number
        total:
          type: number
          format: float
        status:
          type: string
          enum: [pending, processing, shipped, delivered, cancelled]
        created_at:
          type: string
          format: date-time
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        // Version detection
        assert!(openapi.is_32());

        // 3.1 fields
        assert_eq!(
            openapi.json_schema_dialect.as_ref().unwrap(),
            "https://spec.openapis.org/oas/3.2/dialect/base"
        );
        assert!(openapi.webhooks.is_some());

        // 3.2 fields
        assert_eq!(
            openapi.self_ref.as_ref().unwrap(),
            "https://api.example.com/v2"
        );
        assert_eq!(
            openapi.info.summary.as_ref().unwrap(),
            "Complete e-commerce management API"
        );

        // Webhooks verification
        let webhooks = openapi.webhooks.as_ref().unwrap();
        assert_eq!(webhooks.len(), 3);
        assert!(webhooks.contains_key("orderCreated"));
        assert!(webhooks.contains_key("orderShipped"));
        assert!(webhooks.contains_key("inventoryLow"));

        // Paths verification
        let products_path = openapi.paths.get("/products").unwrap();
        assert!(products_path.operations.get("get").is_some());
        assert!(products_path.operations.get("post").is_some());
        assert!(products_path.query.is_some());

        // QUERY method verification
        let products_query = products_path.query.as_ref().unwrap();
        assert_eq!(
            products_query.operation_id.as_ref().unwrap(),
            "queryProducts"
        );
        assert!(products_query.request.is_some());

        let orders_path = openapi.paths.get("/orders").unwrap();
        assert!(orders_path.operations.get("get").is_some());
        assert!(orders_path.query.is_some());

        // Components verification
        let components = openapi.components.as_ref().unwrap();
        assert!(components.schemas.contains_key("Product"));
        assert!(components.schemas.contains_key("ProductCreate"));
        assert!(components.schemas.contains_key("ProductUpdate"));
        assert!(components.schemas.contains_key("Order"));

        Ok(())
    }

    // ==================== Validation Tests for New Features ====================

    #[test]
    fn validate_query_method_recognized() {
        use openapi_rs::model::parse::OpenAPI;
        use openapi_rs::validator::method;

        let content = r#"
openapi: 3.2.0
info:
  title: Test API
  version: '1.0.0'
paths:
  /test:
    query:
      summary: Query operation
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content).unwrap();
        assert!(method("/test", "query", &openapi).is_ok());
        assert!(method("/test", "QUERY", &openapi).is_ok());
    }

    #[test]
    fn validate_querystring_parameter_must_be_json() -> Result<(), Box<dyn std::error::Error>> {
        use openapi_rs::model::parse::OpenAPI;
        use openapi_rs::validator::query;
        use std::collections::HashMap;

        let content = r#"
openapi: 3.2.0
info:
  title: Test API
  version: '1.0.0'
paths:
  /search:
    get:
      parameters:
        - name: filter
          in: querystring
          content:
            application/json:
              schema:
                type: object
      responses:
        '200':
          description: OK
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        // Valid JSON should pass
        let mut query_params = HashMap::new();
        query_params.insert("filter".to_string(), r#"{"status":"active"}"#.to_string());
        assert!(query("/search", &query_params, &openapi).is_ok());

        // Invalid JSON should fail
        query_params.insert("filter".to_string(), "invalid-json".to_string());
        assert!(query("/search", &query_params, &openapi).is_err());

        Ok(())
    }
}
