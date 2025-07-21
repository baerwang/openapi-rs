#[cfg(test)]
mod tests {
    use crate::model::parse::OpenAPI;
    use crate::validator::{body, query};
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_enum_validation_query_parameter() {
        let yaml_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /test:
    get:
      parameters:
        - name: status
          in: query
          required: true
          schema:
            type: string
            enum: ["active", "inactive", "pending"]
        - name: priority
          in: query
          required: false
          enum: [1, 2, 3]
components: {}
"#;

        let open_api: OpenAPI = serde_yaml::from_str(yaml_content).unwrap();

        let mut valid_query = HashMap::new();
        valid_query.insert("status".to_string(), "active".to_string());
        valid_query.insert("priority".to_string(), "2".to_string());

        let result = query("/test", valid_query, &open_api);
        if let Err(ref e) = result {
            println!("Error message: {}", e);
        }
        assert!(result.is_ok(), "Valid enum values should pass validation");

        let mut invalid_query = HashMap::new();
        invalid_query.insert("status".to_string(), "unknown".to_string());

        let result = query("/test", invalid_query, &open_api);
        assert!(result.is_err(), "Invalid enum values should be rejected");

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("not in allowed enum values"),
            "Error message should contain enum validation hint"
        );
        assert!(
            error_msg.contains("active"),
            "Error message should show allowed enum values"
        );
    }

    #[test]
    fn test_enum_validation_with_different_types() {
        let yaml_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /test:
    get:
      parameters:
        - name: active
          in: query
          schema:
            type: boolean
            enum: [true, false]
        - name: count
          in: query
          schema:
            type: integer
            enum: [1, 5, 10]
components: {}
"#;

        let open_api: OpenAPI = serde_yaml::from_str(yaml_content).unwrap();

        let mut query_params = HashMap::new();
        query_params.insert("active".to_string(), "true".to_string());

        let result = query("/test", query_params, &open_api);
        assert!(
            result.is_ok(),
            "Valid boolean enum values should pass validation"
        );

        let mut invalid_query = HashMap::new();
        invalid_query.insert("active".to_string(), "maybe".to_string());

        let result = query("/test", invalid_query, &open_api);
        assert!(
            result.is_err(),
            "Invalid boolean enum values should be rejected"
        );
    }

    #[test]
    fn test_enum_validation_in_properties() {
        let yaml_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /test:
    post:
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/TestRequest'
components:
  schemas:
    TestRequest:
      type: object
      properties:
        status:
          type: string
          enum: ["draft", "published", "archived"]
        priority:
          type: integer
          enum: [1, 2, 3, 4, 5]
      required:
        - status
"#;

        let open_api: OpenAPI = serde_yaml::from_str(yaml_content).unwrap();

        let valid_body = json!({
            "status": "published",
            "priority": 3
        });

        let result = body("/test", valid_body, &open_api);
        assert!(
            result.is_ok(),
            "Valid request body enum values should pass validation"
        );

        let invalid_body = json!({
            "status": "invalid_status",
            "priority": 3
        });

        let result = body("/test", invalid_body, &open_api);
        assert!(
            result.is_err(),
            "Invalid request body enum values should be rejected"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("not in allowed enum values"),
            "Error message should contain enum validation hint"
        );
    }
}
