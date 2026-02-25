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
    use crate::model::parse::{
        In, InfoObject, OpenAPI, Parameter, PathBase, PathItem, Schema, Type, TypeOrUnion,
    };
    use crate::validator::{query, validate_pattern};
    use serde_json::Value;
    use std::collections::HashMap;

    const EMAIL_PATTERN: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
    const PHONE_PATTERN: &str = r"^\+?1?[-.\s]?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}$";
    const SSN_PATTERN: &str = r"^\d{3}-\d{2}-\d{4}$";
    const INVALID_REGEX: &str = "[invalid-regex";

    struct TestCase {
        name: &'static str,
        pattern: &'static str,
        valid_values: Vec<&'static str>,
        invalid_values: Vec<&'static str>,
    }

    impl TestCase {
        const fn new(
            name: &'static str,
            pattern: &'static str,
            valid_values: Vec<&'static str>,
            invalid_values: Vec<&'static str>,
        ) -> Self {
            Self {
                name,
                pattern,
                valid_values,
                invalid_values,
            }
        }
    }

    fn create_base_openapi() -> OpenAPI {
        OpenAPI {
            openapi: "3.1.0".to_string(),
            info: InfoObject {
                title: "Test API".to_string(),
                description: None,
                version: "1.0.0".to_string(),
                summary: None,
            },
            servers: vec![],
            paths: HashMap::new(),
            components: None,
            json_schema_dialect: None,
            webhooks: None,
            self_ref: None,
        }
    }

    fn create_parameter_with_pattern(
        name: &str,
        pattern: Option<String>,
        required: bool,
    ) -> Parameter {
        Parameter {
            r#ref: None,
            name: Some(name.to_string()),
            r#in: Some(In::Query),
            required,
            description: None,
            example: None,
            r#type: Some(TypeOrUnion::Single(Type::String)),
            r#enum: None,
            pattern,
            schema: None,
            extra: HashMap::new(),
        }
    }

    fn create_parameter_with_schema_pattern(
        name: &str,
        pattern: Option<String>,
        required: bool,
    ) -> Parameter {
        let schema = Schema {
            r#type: Some(TypeOrUnion::Single(Type::String)),
            format: None,
            title: None,
            description: None,
            r#enum: None,
            pattern,
            properties: None,
            example: None,
            examples: None,
            r#ref: None,
            all_of: None,
            one_of: None,
            items: None,
            required: vec![],
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            minimum: None,
            maximum: None,
        };

        Parameter {
            r#ref: None,
            name: Some(name.to_string()),
            r#in: Some(In::Query),
            required,
            description: None,
            example: None,
            r#type: None,
            r#enum: None,
            pattern: None,
            schema: Some(Box::new(schema)),
            extra: HashMap::new(),
        }
    }

    fn create_openapi_with_parameters(parameters: Vec<Parameter>) -> OpenAPI {
        let mut openapi = create_base_openapi();

        let path_base = PathBase {
            summary: None,
            description: None,
            operation_id: None,
            parameters: Some(parameters),
            request: None,
            servers: vec![],
        };

        let mut operations = HashMap::new();
        operations.insert("get".to_string(), path_base);

        let path_item = PathItem {
            parameters: None,
            operations,
            servers: vec![],
            query: None,
            extra: serde_yaml::Value::Null,
        };

        openapi.paths.insert("/test".to_string(), path_item);
        openapi
    }

    fn test_query_validation(openapi: &OpenAPI, params: &[(&str, &str)], should_succeed: bool) {
        let query_params: HashMap<String, String> = params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let result = query("/test", &query_params, openapi);

        if should_succeed {
            assert!(
                result.is_ok(),
                "Expected validation to succeed but got error: {:?}",
                result.err()
            );
        } else {
            assert!(
                result.is_err(),
                "Expected validation to fail but it succeeded"
            );
            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("does not match the required pattern"),
                "Error message should mention pattern mismatch, got: {}",
                error_msg
            );
        }
    }

    #[test]
    fn test_pattern_validation_comprehensive() {
        let test_cases = [
            TestCase::new(
                "email",
                EMAIL_PATTERN,
                vec![
                    "test@example.com",
                    "user.name+tag@domain.co.uk",
                    "test123@test-domain.org",
                ],
                vec![
                    "invalid-email",
                    "test@",
                    "@domain.com",
                    "test.domain.com",
                    "test@domain",
                ],
            ),
            TestCase::new(
                "phone",
                PHONE_PATTERN,
                vec![
                    "(555) 123-4567",
                    "555-123-4567",
                    "5551234567",
                    "+1 555 123 4567",
                ],
                vec!["invalid-phone", "123", "555-123-456", "(555) 123-45678"],
            ),
            TestCase::new(
                "ssn",
                SSN_PATTERN,
                vec!["123-45-6789", "000-00-0000", "999-99-9999"],
                vec!["123456789", "123-4-5678", "1234-56-789", "123-456-789"],
            ),
        ];

        for test_case in test_cases.iter() {
            let param = create_parameter_with_pattern(
                test_case.name,
                Some(test_case.pattern.to_string()),
                true,
            );
            let openapi = create_openapi_with_parameters(vec![param]);

            for valid_value in &test_case.valid_values {
                test_query_validation(&openapi, &[(test_case.name, valid_value)], true);
            }

            for invalid_value in &test_case.invalid_values {
                test_query_validation(&openapi, &[(test_case.name, invalid_value)], false);
            }
        }
    }

    #[test]
    fn test_schema_pattern_validation() {
        let param =
            create_parameter_with_schema_pattern("ssn", Some(SSN_PATTERN.to_string()), true);
        let openapi = create_openapi_with_parameters(vec![param]);

        test_query_validation(&openapi, &[("ssn", "123-45-6789")], true);

        test_query_validation(&openapi, &[("ssn", "123456789")], false);
    }

    #[test]
    fn test_multiple_patterns_validation() {
        let parameters = vec![
            create_parameter_with_pattern("email", Some(EMAIL_PATTERN.to_string()), true),
            create_parameter_with_pattern("phone", Some(PHONE_PATTERN.to_string()), false),
        ];
        let openapi = create_openapi_with_parameters(parameters);

        test_query_validation(
            &openapi,
            &[("email", "test@example.com"), ("phone", "(555) 123-4567")],
            true,
        );

        test_query_validation(
            &openapi,
            &[("email", "test@example.com"), ("phone", "invalid-phone")],
            false,
        );

        test_query_validation(&openapi, &[("email", "test@example.com")], true);

        test_query_validation(&openapi, &[("email", "invalid-email")], false);
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let param = create_parameter_with_pattern("test", Some(INVALID_REGEX.to_string()), true);
        let openapi = create_openapi_with_parameters(vec![param]);

        let result = query(
            "/test",
            &[("test", "anything")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            &openapi,
        );

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Invalid regex pattern"),
            "Error message should mention invalid regex, got: {}",
            error_msg
        );
    }

    #[test]
    fn test_pattern_with_non_string_values() {
        let test_cases = [
            ("number", Value::Number(123.into())),
            ("boolean", Value::Bool(true)),
            ("null", Value::Null),
            (
                "array",
                Value::Array(vec![Value::String("test".to_string())]),
            ),
            ("object", Value::Object(serde_json::Map::new())),
        ];

        for (name, value) in test_cases.iter() {
            let result = validate_pattern(name, value, Some(&"^\\d+$".to_string()));
            assert!(
                result.is_ok(),
                "Non-string value {} should pass pattern validation",
                name
            );
        }
    }

    #[test]
    fn test_pattern_validation_edge_cases() {
        let edge_cases = [
            (
                "empty pattern should always succeed",
                "anything",
                None,
                true,
            ),
            (
                "empty string with non-empty pattern should fail",
                "",
                Some("^.+$"),
                false,
            ),
            (
                "empty string with empty pattern should succeed",
                "",
                Some("^$"),
                true,
            ),
            (
                "whitespace with whitespace pattern",
                "   ",
                Some("^\\s+$"),
                true,
            ),
            (
                "complex unicode pattern",
                "test@example.rs",
                Some("^[\\p{L}\\p{N}.@]+$"),
                true,
            ),
            (
                "digit pattern with letters should fail",
                "abc123",
                Some("^\\d+$"),
                false,
            ),
            (
                "optional pattern with valid input",
                "test123",
                Some("^[a-z]+\\d*$"),
                true,
            ),
        ];

        for (description, value, pattern, should_succeed) in edge_cases.iter() {
            let pattern_string = pattern.map(|p| p.to_string());
            let result = validate_pattern(
                "test_field",
                &Value::String(value.to_string()),
                pattern_string.as_ref(),
            );

            if *should_succeed {
                assert!(
                    result.is_ok(),
                    "Test '{}' should succeed but failed: {:?}",
                    description,
                    result.err()
                );
            } else {
                assert!(
                    result.is_err(),
                    "Test '{}' should fail but succeeded",
                    description
                );
            }
        }
    }

    #[test]
    fn test_pattern_priority_parameter_vs_schema() {
        let schema = Schema {
            r#type: Some(TypeOrUnion::Single(Type::String)),
            pattern: Some("^schema-pattern$".to_string()),
            format: None,
            title: None,
            description: None,
            r#enum: None,
            properties: None,
            example: None,
            examples: None,
            r#ref: None,
            all_of: None,
            one_of: None,
            items: None,
            required: vec![],
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            minimum: None,
            maximum: None,
        };

        let param = Parameter {
            r#ref: None,
            name: Some("test".to_string()),
            r#in: Some(In::Query),
            required: true,
            description: None,
            example: None,
            r#type: None,
            r#enum: None,
            pattern: Some("^param-pattern$".to_string()),
            schema: Some(Box::new(schema)),
            extra: HashMap::new(),
        };

        let openapi = create_openapi_with_parameters(vec![param]);

        test_query_validation(&openapi, &[("test", "param-pattern")], false); // 只匹配参数 pattern，不匹配 schema pattern
        test_query_validation(&openapi, &[("test", "schema-pattern")], false); // 只匹配 schema pattern，不匹配参数 pattern
    }

    #[test]
    fn test_pattern_performance_with_complex_regex() {
        let complex_pattern = r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$";

        let start = std::time::Instant::now();

        let param = create_parameter_with_pattern("email", Some(complex_pattern.to_string()), true);
        let openapi = create_openapi_with_parameters(vec![param]);

        for _ in 0..100 {
            test_query_validation(&openapi, &[("email", "test@example.com")], true);
        }

        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 1000,
            "Pattern validation should be fast, took {:?}",
            duration
        );
    }
}
