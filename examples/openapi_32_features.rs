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

//! OpenAPI 3.2 Features Example
//!
//! This example demonstrates how to parse and use OpenAPI 3.2 specific features:
//! - $self: Base URI for the API document
//! - info.summary: Short API summary
//! - querystring: Complex query parameter type
//! - query: HTTP method for complex queries

use anyhow::Result;
use openapi_rs::model::parse::{In, OpenAPI};

fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     OpenAPI 3.2 Features Demonstration                     ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Example 1: $self field
    println!("🔗 Example 1: $self Field (OpenAPI 3.2)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demonstrate_self_field()?;

    println!();

    // Example 2: info.summary
    println!("📝 Example 2: info.summary (OpenAPI 3.2)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demonstrate_info_summary()?;

    println!();

    // Example 3: querystring parameter type
    println!("🔍 Example 3: querystring Parameter Type (OpenAPI 3.2)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demonstrate_querystring_parameter()?;

    println!();

    // Example 4: QUERY HTTP method
    println!("📊 Example 4: QUERY HTTP Method (OpenAPI 3.2)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demonstrate_query_method()?;

    println!();

    // Example 5: Complete 3.2 spec
    println!("🎯 Example 5: Complete OpenAPI 3.2 Spec");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demonstrate_complete_32_spec()?;

    println!();
    println!("✅ All OpenAPI 3.2 features demonstrated successfully!");

    Ok(())
}

/// Demonstrates the $self field (base URI for the API)
fn demonstrate_self_field() -> Result<()> {
    let self_yaml = r#"
openapi: 3.2.0
$self: https://api.example.com/v3
info:
  title: Cloud API
  version: '3.0.0'
paths:
  /instances:
    get:
      responses:
        '200':
          description: OK
    "#;

    let openapi: OpenAPI = OpenAPI::yaml(self_yaml)?;

    println!("   OpenAPI Version: {}", openapi.openapi);
    println!(
        "   Version Detection: 3.1 = {}, 3.2 = {}",
        openapi.is_31(),
        openapi.is_32()
    );

    // Access $self field
    match &openapi.self_ref {
        Some(self_ref) => {
            println!("   📍 Base URL ($self): {}", self_ref);

            // Parse and display components
            if let Ok(url) = url::Url::parse(self_ref) {
                println!("      Scheme: {}", url.scheme());
                println!("      Host: {}", url.host_str().unwrap_or("unknown"));
                if let Some(path) = url.path_segments() {
                    let path: Vec<&str> = path.collect();
                    if !path.is_empty() {
                        println!("      Path: /{}", path.join("/"));
                    }
                }
            }
        }
        None => {
            println!("   ℹ️  No $self field specified");
        }
    }

    Ok(())
}

/// Demonstrates the info.summary field
fn demonstrate_info_summary() -> Result<()> {
    let summary_yaml = r#"
openapi: 3.2.0
$self: https://api.example.com/v1
info:
  title: User Management API
  summary: Complete user lifecycle management
  description: |
    This API provides endpoints for managing users throughout their lifecycle,
    including registration, authentication, profile management, and deletion.
  version: '2.0.0'
paths:
  /users:
    get:
      responses:
        '200':
          description: OK
    "#;

    let openapi: OpenAPI = OpenAPI::yaml(summary_yaml)?;

    println!("   API Title: {}", openapi.info.title);

    // Access summary
    match &openapi.info.summary {
        Some(summary) => {
            println!("   📋 Summary: {}", summary);
            println!("   (Short description separate from full description)");
        }
        None => {
            println!("   ℹ️  No summary provided");
        }
    }

    // Also show description for comparison
    if let Some(description) = &openapi.info.description {
        let desc_preview: String = description.chars().take(60).collect();
        println!("   📄 Description Preview: {}...", desc_preview);
    }

    Ok(())
}

/// Demonstrates the querystring parameter type
fn demonstrate_querystring_parameter() -> Result<()> {
    let querystring_yaml = r#"
openapi: 3.2.0
info:
  title: Advanced Search API
  summary: Complex search with structured parameters
  version: '1.0.0'
paths:
  /search:
    get:
      summary: Advanced search
      parameters:
        - name: filter
          in: querystring
          description: Complex filter object
          required: false
          content:
            application/json:
              schema:
                type: object
                properties:
                  status:
                    type: string
                  price:
                    type: object
                    properties:
                      min:
                        type: number
                      max:
                        type: number
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
        - name: format
          in: query
          description: Response format
          schema:
            type: string
            enum: [json, xml]
      responses:
        '200':
          description: OK
    "#;

    let openapi: OpenAPI = OpenAPI::yaml(querystring_yaml)?;

    println!("   Analyzing parameters for /search endpoint:");

    let search_path = openapi.paths.get("/search").unwrap();
    let get_operation = search_path.operations.get("get").unwrap();

    if let Some(params) = &get_operation.parameters {
        println!("   Total parameters: {}", params.len());

        for (i, param) in params.iter().enumerate() {
            println!();
            println!(
                "   {}. Parameter: {}",
                i + 1,
                param.name.as_deref().unwrap_or("unknown")
            );

            match &param.r#in {
                Some(In::QueryString) => {
                    println!("      Location: querystring (OpenAPI 3.2 feature)");
                    println!("      └─ Complex structured parameter");
                }
                Some(In::Query) => {
                    println!("      Location: query (traditional)");
                    println!("      └─ Simple scalar parameter");
                }
                other => {
                    println!("      Location: {:?}", other);
                }
            }

            if let Some(description) = &param.description {
                let preview: String = description.chars().take(50).collect();
                println!("      Description: {}...", preview);
            }
        }
    }

    Ok(())
}

/// Demonstrates the QUERY HTTP method
fn demonstrate_query_method() -> Result<()> {
    let query_method_yaml = r#"
openapi: 3.2.0
info:
  title: Resource Query API
  summary: Demonstrates QUERY HTTP method
  version: '1.0.0'
paths:
  /users:
    get:
      summary: List users (simple)
      parameters:
        - name: status
          in: query
          schema:
            type: string
      responses:
        '200':
          description: OK
    query:
      summary: Complex user query
      description: Execute complex queries with filtering, sorting, and aggregation
      operationId: queryUsers
      parameters:
        - name: include
          in: query
          schema:
            type: array
            items:
              type: string
      requestBody:
        required: true
        description: Query DSL
        content:
          application/json:
            schema:
              type: object
              properties:
                filter:
                  type: object
                sort:
                  type: array
                  items:
                    type: object
                pagination:
                  type: object
      responses:
        '200':
          description: Query results
    "#;

    let openapi: OpenAPI = OpenAPI::yaml(query_method_yaml)?;

    println!("   Endpoint: /users");
    println!();

    let users_path = openapi.paths.get("/users").unwrap();

    // Show traditional GET method
    if let Some(get_op) = users_path.operations.get("get") {
        println!("   GET Method (traditional):");
        if let Some(summary) = &get_op.summary {
            println!("      Summary: {}", summary);
        }
        if let Some(params) = &get_op.parameters {
            println!("      Parameters: {} (simple query params)", params.len());
        }
    }

    println!();

    // Show QUERY method
    if let Some(query_op) = &users_path.query {
        println!("   QUERY Method (OpenAPI 3.2):");
        if let Some(summary) = &query_op.summary {
            println!("      Summary: {}", summary);
        }
        if let Some(description) = &query_op.description {
            let preview: String = description.chars().take(60).collect();
            println!("      Description: {}...", preview);
        }
        if let Some(operation_id) = &query_op.operation_id {
            println!("      Operation ID: {}", operation_id);
        }
        if query_op.request.is_some() {
            println!("      Request Body: ✅ (complex query DSL)");
        }
        if let Some(params) = &query_op.parameters {
            println!("      Parameters: {} (additional options)", params.len());
        }
    }

    Ok(())
}

/// Demonstrates a complete OpenAPI 3.2 specification
fn demonstrate_complete_32_spec() -> Result<()> {
    let complete_yaml = r#"
openapi: 3.2.0
$self: https://api.example.com/v3
jsonSchemaDialect: https://spec.openapis.org/oas/3.2/dialect/base
info:
  title: Complete API
  summary: All OpenAPI 3.2 features
  description: API demonstrating all 3.2 features
  version: '3.0.0'
webhooks:
  event:
    post:
      summary: Event webhook
      responses:
        '200':
          description: OK
paths:
  /resources:
    get:
      summary: List resources
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
    query:
      summary: Query resources
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
      responses:
        '200':
          description: OK
    "#;

    let openapi: OpenAPI = OpenAPI::yaml(complete_yaml)?;

    println!("   ════════════════════════════════════════════════════");
    println!("   OpenAPI 3.2 Feature Checklist:");
    println!("   ════════════════════════════════════════════════════");

    // Version detection
    println!();
    println!("   ✅ Version: 3.2");
    println!("      - is_31(): {}", openapi.is_31());
    println!("      - is_32(): {}", openapi.is_32());

    // $self field
    println!();
    if openapi.self_ref.is_some() {
        println!("   ✅ $self field: {}", openapi.self_ref.as_ref().unwrap());
    } else {
        println!("   ❌ $self field: not present");
    }

    // info.summary
    println!();
    if openapi.info.summary.is_some() {
        println!(
            "   ✅ info.summary: {}",
            openapi.info.summary.as_ref().unwrap()
        );
    } else {
        println!("   ❌ info.summary: not present");
    }

    // jsonSchemaDialect (3.1 but still usable in 3.2)
    println!();
    if openapi.json_schema_dialect.is_some() {
        println!(
            "   ✅ jsonSchemaDialect: {}",
            openapi.json_schema_dialect.as_ref().unwrap()
        );
    } else {
        println!("   ❌ jsonSchemaDialect: not present");
    }

    // webhooks
    println!();
    if openapi.webhooks.is_some() {
        println!(
            "   ✅ webhooks: {} defined",
            openapi.webhooks.as_ref().unwrap().len()
        );
    } else {
        println!("   ❌ webhooks: not present");
    }

    // querystring parameter
    println!();
    let resources_path = openapi.paths.get("/resources").unwrap();
    let get_op = resources_path.operations.get("get").unwrap();
    let has_querystring = get_op
        .parameters
        .as_ref()
        .map(|p| p.iter().any(|param| param.r#in == Some(In::QueryString)))
        .unwrap_or(false);
    if has_querystring {
        println!("   ✅ querystring parameter: present in GET /resources");
    } else {
        println!("   ❌ querystring parameter: not found");
    }

    // QUERY method
    println!();
    if resources_path.query.is_some() {
        println!("   ✅ QUERY method: defined on /resources");
    } else {
        println!("   ❌ QUERY method: not found");
    }

    println!();
    println!("   ════════════════════════════════════════════════════");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_self_field() {
        let yaml = r#"
openapi: 3.2.0
$self: https://api.example.com
info:
  title: Test
  version: '1.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
        "#;

        let openapi: OpenAPI = OpenAPI::yaml(yaml).unwrap();
        assert_eq!(
            openapi.self_ref.as_ref().unwrap(),
            "https://api.example.com"
        );
        assert!(openapi.is_32());
    }

    #[test]
    fn test_parse_info_summary() {
        let yaml = r#"
openapi: 3.2.0
info:
  title: Test
  summary: Test summary
  version: '1.0'
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
        "#;

        let openapi: OpenAPI = OpenAPI::yaml(yaml).unwrap();
        assert_eq!(openapi.info.summary.as_ref().unwrap(), "Test summary");
    }

    #[test]
    fn test_parse_querystring_parameter() {
        let yaml = r#"
openapi: 3.2.0
info:
  title: Test
  version: '1.0'
paths:
  /test:
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

        let openapi: OpenAPI = OpenAPI::yaml(yaml).unwrap();
        let test_path = openapi.paths.get("/test").unwrap();
        let get_op = test_path.operations.get("get").unwrap();
        let params = get_op.parameters.as_ref().unwrap();
        assert_eq!(params[0].r#in, Some(In::QueryString));
    }

    #[test]
    fn test_parse_query_method() {
        let yaml = r#"
openapi: 3.2.0
info:
  title: Test
  version: '1.0'
paths:
  /test:
    query:
      summary: Test query
      responses:
        '200':
          description: OK
        "#;

        let openapi: OpenAPI = OpenAPI::yaml(yaml).unwrap();
        let test_path = openapi.paths.get("/test").unwrap();
        assert!(test_path.query.is_some());
        assert_eq!(
            test_path.query.as_ref().unwrap().summary.as_ref().unwrap(),
            "Test query"
        );
    }
}
