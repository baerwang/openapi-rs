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
    use openapi_rs::model::prase::{Format, In, Method, OpenAPI, Type};
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
        assert!(openapi.servers.is_empty());
        assert!(openapi.security.is_empty());
        assert!(openapi.tags.is_empty());
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
            .get(&Method::Get)
            .ok_or("Missing GET method for /example/{uuid}")?;

        // Validate GET parameters
        let parameter = get_value
            .parameters
            .as_ref()
            .and_then(|params| params.first())
            .ok_or("Missing parameter")?;
        assert_eq!(parameter.name, "uuid");
        assert_eq!(
            parameter.description.as_deref(),
            Some("The UUID for this example.")
        );
        assert_eq!(parameter._in, In::Path);
        assert_eq!(parameter.schema._type, Type::String);
        assert_eq!(parameter.schema.format, Format::UUID);
        assert_eq!(
            parameter.schema.example.clone().unwrap(),
            "00000000-0000-0000-0000-000000000000"
        );
        assert!(parameter.schema.examples.is_none());

        // Validate responses
        let resp = get_value
            .responses
            .as_ref()
            .and_then(|r| r.get("200"))
            .ok_or("Missing 200 response")?;
        assert_eq!(resp.description.as_deref(), Some("Get a Example response"));

        let resp_content = resp
            .content
            .get("application/json")
            .ok_or("Missing content type: application/json")?;
        let schema = &resp_content.schema;
        assert!(schema._ref.is_some());
        assert_eq!(
            schema._ref.as_ref().unwrap(),
            "#/components/schemas/ExampleResponse"
        );
        assert_eq!(schema._type, Type::Undefined);
        assert_eq!(schema.format, Format::Undefined);
        assert!(schema.example.is_none());

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
        assert_eq!(openapi.info.description, "API definitions for example");
        assert_eq!(openapi.info.version, "0.0.1");
        assert!(openapi.security.is_empty());

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
        assert_eq!(result._type, Type::String);
        assert_eq!(result.minimum, 0);
        assert_eq!(result.maximum, 0);
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
  version: '0.0.1'

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
                  minimum: 0
                  maximum: 1
              required:
                - uuid
paths:
    "#;

        let openapi: OpenAPI = OpenAPI::yaml(content)?;

        // Validate general properties
        assert_eq!(openapi.openapi, "3.1.0");
        assert_eq!(openapi.info.title, "Example API");
        assert_eq!(openapi.info.description, "API definitions for example");
        assert_eq!(openapi.info.version, "0.0.1");
        assert!(openapi.security.is_empty());

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
        assert_eq!(first._type, Type::Object);
        assert!(first.description.is_none());

        // Validate "result" object properties
        let result = first
            .properties
            .get("result")
            .ok_or("Missing result property")?;
        assert_eq!(result._type, Type::Object);
        assert_eq!(result.description.as_deref(), Some("example."));
        assert!(!result.required.is_empty());

        // Validate "uuid" property in result
        let uuid = result
            .properties
            .as_ref()
            .ok_or("Missing properties in result")?
            .get("uuid")
            .ok_or("Missing uuid")?;
        assert_eq!(uuid._type, Type::String);
        assert_eq!(
            uuid.description.as_deref(),
            Some("The UUID for this example.")
        );
        assert_eq!(uuid.format, Format::UUID);
        assert_eq!(
            uuid.example.clone().unwrap(),
            "00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(uuid.minimum, 0);
        assert_eq!(uuid.maximum, 0);

        // Validate "count" property in result
        let count = result
            .properties
            .as_ref()
            .ok_or("Missing properties in result")?
            .get("count")
            .ok_or("Missing count")?;
        assert_eq!(count._type, Type::Integer);
        assert_eq!(count.description.as_deref(), Some("example count."));
        assert_eq!(count.format, Format::Undefined);
        assert_eq!(count.example.clone().unwrap(), 1);
        assert_eq!(count.minimum, 0);
        assert_eq!(count.maximum, 1);

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
        assert_eq!(openapi.info.description, "API definitions for example");
        assert_eq!(openapi.info.version, "0.0.1");
        assert!(openapi.security.is_empty());

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
        assert_eq!(one_of._type, Type::Object);
        assert!(one_of.description.is_none());

        // Validate "result" object properties

        let result = one_of
            .properties
            .get("result")
            .ok_or("Missing result property")?;
        assert_eq!(result._type, Type::Object);
        assert_eq!(result.description.as_deref(), Some("example."));
        assert!(result.required.is_empty());

        // Validate "uuid" property in result
        let uuid = result
            .properties
            .as_ref()
            .ok_or("Missing properties in result")?
            .get("uuid")
            .ok_or("Missing uuid")?;
        assert_eq!(uuid._type, Type::String);
        assert_eq!(
            uuid.description.as_deref(),
            Some("The UUID for this example.")
        );
        assert_eq!(uuid.format, Format::UUID);
        assert_eq!(
            uuid.example.clone().unwrap(),
            "00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(uuid.minimum, 0);
        assert_eq!(uuid.maximum, 0);

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
        assert_eq!(openapi.info.description, "API definitions for example");
        assert_eq!(openapi.info.version, "0.0.1");
        assert!(openapi.security.is_empty());
        assert!(openapi.components.is_none());

        // Validate paths
        let example_path = openapi
            .paths
            .get("/example")
            .ok_or("Missing path: /example")?;

        let get_value = example_path
            .get(&Method::Get)
            .ok_or("Missing GET method for /example")?;

        // Validate responses
        let resp = get_value
            .responses
            .as_ref()
            .and_then(|r| r.get("200"))
            .ok_or("Missing 200 response")?;
        assert_eq!(resp.description.as_deref(), Some("OK"));

        let resp_content = resp
            .content
            .get("application/json")
            .ok_or("Missing content type: application/json")?;

        let schema = &resp_content.schema;
        assert!(schema._ref.is_none());
        assert!(schema.all_of.is_none());
        assert!(schema.one_of.is_some());

        // Validate 'oneOf' directly in the loop with better error reporting
        let one_of = schema.one_of.as_ref().ok_or("Missing oneOf")?;
        assert_eq!(one_of.len(), 3);
        for one_of_item in one_of {
            assert_eq!(one_of_item._type, Type::Undefined);
            assert!(one_of_item.description.is_none());
            assert!(one_of_item.properties.is_empty());

            match &one_of_item._ref {
                Some(r#ref) if !r#ref.is_empty() => (),
                Some(_) => return Err("Reference is empty".into()),
                None => return Err("Missing reference".into()),
            }
        }

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
        assert_eq!(openapi.info.description, "API definitions for example");
        assert_eq!(openapi.info.version, "0.0.1");
        assert!(openapi.security.is_empty());

        // Validate components and schemas
        let components = openapi.components.as_ref().unwrap();
        let example_response = components.schemas.get("ExampleResponse").unwrap();
        let properties = example_response.properties.as_ref().unwrap();

        // Validate uuid property
        let uuid = properties.get("uuid").unwrap();
        assert_eq!(uuid._type, Type::String);
        assert_eq!(
            uuid.description.as_deref(),
            Some("The UUID for this example.")
        );
        assert_eq!(uuid.format, Format::UUID);
        assert_eq!(
            uuid.example.clone().unwrap(),
            "00000000-0000-0000-0000-000000000000"
        );

        // Validate multi_uuid property
        let multi_uuid = properties.get("multi_uuid").unwrap();
        assert_eq!(multi_uuid._type, Type::Array);
        assert_eq!(
            multi_uuid.description.as_deref(),
            Some("The Multi UUID for this example.")
        );
        assert_eq!(multi_uuid.format, Format::UUID);

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
}
