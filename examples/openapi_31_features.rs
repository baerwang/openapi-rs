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

//! OpenAPI 3.1 Features Example
//!
//! This example demonstrates how to parse and use OpenAPI 3.1 specific features:
//! - webhooks: Incoming webhook definitions
//! - jsonSchemaDialect: JSON Schema dialect specification

use anyhow::Result;
use openapi_rs::model::parse::OpenAPI;

fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     OpenAPI 3.1 Features Demonstration                     ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Example 1: Parse API with Webhooks
    println!("📡 Example 1: Parsing Webhooks (OpenAPI 3.1)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demonstrate_webhooks()?;

    println!();

    // Example 2: Parse API with JSON Schema Dialect
    println!("📋 Example 2: JSON Schema Dialect (OpenAPI 3.1)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demonstrate_json_schema_dialect()?;

    println!();
    println!("✅ All OpenAPI 3.1 features demonstrated successfully!");

    Ok(())
}

/// Demonstrates parsing and working with webhooks
fn demonstrate_webhooks() -> Result<()> {
    let webhook_yaml = r#"
openapi: 3.1.0
info:
  title: E-Commerce API with Webhooks
  version: '1.0.0'
webhooks:
  orderCreated:
    post:
      summary: New order created
      description: Fired when a new order is successfully created
      operationId: orderCreated
      tags:
        - Webhooks
        - Orders
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                order_id:
                  type: string
                customer_id:
                  type: string
                total:
                  type: number
                status:
                  type: string
                  enum: [pending, processing, shipped, delivered, cancelled]
      responses:
        '200':
          description: Webhook received
  orderStatusChanged:
    post:
      summary: Order status updated
      operationId: orderStatusChanged
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
      responses:
        '200':
          description: Webhook processed
  paymentCompleted:
    post:
      summary: Payment completed
      operationId: paymentCompleted
      parameters:
        - name: X-Webhook-Signature
          in: header
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
      responses:
        '200':
          description: Webhook acknowledged
paths:
  /orders:
    get:
      responses:
        '200':
          description: OK
    "#;

    let openapi: OpenAPI = OpenAPI::yaml(webhook_yaml)?;

    // Verify version detection
    println!("   OpenAPI Version: {}", openapi.openapi);
    println!("   Version Detection: 3.1 = {}", openapi.is_31());
    println!("   Version Detection: 3.2 = {}", openapi.is_32());

    // Access webhooks
    if let Some(webhooks) = &openapi.webhooks {
        println!("   📬 Total Webhooks Defined: {}", webhooks.len());

        for (name, path_item) in webhooks.iter() {
            println!();
            println!("   🔖 Webhook: {}", name);

            // Webhooks can have multiple methods (POST, GET, etc.)
            for (method, operation) in path_item.operations.iter() {
                println!("      Method: {}", method.to_uppercase());

                if let Some(summary) = &operation.summary {
                    println!("      Summary: {}", summary);
                }

                if let Some(description) = &operation.description {
                    println!("      Description: {}", description);
                }

                if let Some(operation_id) = &operation.operation_id {
                    println!("      Operation ID: {}", operation_id);
                }

                // Check for parameters
                if let Some(params) = &operation.parameters {
                    println!("      Parameters: {}", params.len());
                    for param in params {
                        println!(
                            "        - {} ({:?})",
                            param.name.as_deref().unwrap_or("unknown"),
                            param.r#in
                        );
                    }
                }

                // Check for request body
                if operation.request.is_some() {
                    println!("      Request Body: ✅");
                }
            }
        }
    } else {
        println!("   ℹ️  No webhooks defined in this spec");
    }

    Ok(())
}

/// Demonstrates parsing and working with JSON Schema Dialect
fn demonstrate_json_schema_dialect() -> Result<()> {
    let dialect_yaml = r#"
openapi: 3.1.0
jsonSchemaDialect: https://spec.openapis.org/oas/3.1/dialect/base
info:
  title: User Management API
  version: '1.0.0'
paths:
  /users:
    get:
      summary: List users
      parameters:
        - name: limit
          in: query
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 20
      responses:
        '200':
          description: OK
    "#;

    let openapi: OpenAPI = OpenAPI::yaml(dialect_yaml)?;

    println!("   OpenAPI Version: {}", openapi.openapi);

    // Access JSON Schema Dialect
    match &openapi.json_schema_dialect {
        Some(dialect) => {
            println!("   📐 JSON Schema Dialect: {}", dialect);

            // Parse and display info about the dialect
            if dialect.contains("3.1") {
                println!("      └─ Using OpenAPI 3.1 compatible dialect");
            } else if dialect.contains("2020-12") {
                println!("      └─ Using JSON Schema 2020-12");
            } else if dialect.contains("2019-09") {
                println!("      └─ Using JSON Schema 2019-09");
            } else {
                println!("      └─ Using custom dialect");
            }
        }
        None => {
            println!("   ℹ️  No JSON Schema Dialect specified (will use default)");
        }
    }

    // Demonstrate that other 3.0 features still work
    println!();
    println!("   📄 Paths Available: {}", openapi.paths.len());

    for (path, _path_item) in openapi.paths.iter() {
        println!("      - {}", path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_webhooks() {
        let yaml = r#"
openapi: 3.1.0
info:
  title: Test API
  version: '1.0.0'
webhooks:
  testEvent:
    post:
      summary: Test webhook
      responses:
        '200':
          description: OK
paths:
  /test:
    get:
      responses:
        '200':
          description: OK
        "#;

        let openapi: OpenAPI = OpenAPI::yaml(yaml).unwrap();
        assert!(openapi.webhooks.is_some());
        assert_eq!(openapi.webhooks.as_ref().unwrap().len(), 1);
        assert!(openapi.is_31());
    }

    #[test]
    fn test_parse_json_schema_dialect() {
        let yaml = r#"
openapi: 3.1.0
jsonSchemaDialect: https://spec.openapis.org/oas/3.1/dialect/base
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

        let openapi: OpenAPI = OpenAPI::yaml(yaml).unwrap();
        assert!(openapi.json_schema_dialect.is_some());
        assert_eq!(
            openapi.json_schema_dialect.as_ref().unwrap(),
            "https://spec.openapis.org/oas/3.1/dialect/base"
        );
    }
}
