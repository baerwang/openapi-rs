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

use crate::validator::ValidateRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAPI {
    pub openapi: String,
    pub info: InfoObject,
    #[serde(default)]
    pub servers: Vec<ServerObject>,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<ComponentsObject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathItem {
    pub parameters: Option<Vec<Parameter>>, // Path-level parameters
    #[serde(flatten)]
    pub operations: HashMap<String, PathBase>, // For HTTP methods (get, post, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<ServerObject>, // Will be ignored during deserialization
    #[serde(flatten)]
    pub extra: serde_yaml::Value, // Catches any other fields
}

macro_rules! require_non_empty {
    ($field:expr, $msg:expr) => {
        if $field.is_empty() {
            return Err($msg.to_string());
        }
    };
}

impl OpenAPI {
    pub fn yaml(contents: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(contents)
    }

    pub fn validator(&self, valid: impl ValidateRequest) -> Result<(), String> {
        require_non_empty!(self.openapi, "OpenAPI version is required");
        require_non_empty!(self.info.title, "Title is required");
        require_non_empty!(self.info.version, "Version is required");
        require_non_empty!(self.paths, "Paths are required");
        valid
            .method(self)
            .map_err(|e| format!("Method validation failed: {e}"))?;
        valid
            .path(self)
            .map_err(|e| format!("Path validation failed: {e}"))?;
        valid
            .query(self)
            .map_err(|e| format!("Query validation failed: {e}"))?;
        valid
            .body(self)
            .map_err(|e| format!("Body validation failed: {e}"))?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityRequirementObject {
    #[serde(rename = "type", default)]
    pub _type: String,
    pub scheme: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoObject {
    pub title: String,
    pub description: Option<String>,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerObject {
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathBase {
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
    #[serde(rename = "requestBody")]
    pub request: Option<Request>,
    #[serde(default)]
    pub servers: Vec<ServerObject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameter {
    #[serde(rename = "$ref")]
    pub r#ref: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "in")]
    pub r#in: Option<In>,
    #[serde(default)]
    pub required: bool,
    pub description: Option<String>,
    pub example: Option<serde_yaml::Value>,
    #[serde(rename = "type")]
    pub r#type: Option<TypeOrUnion>,
    pub r#enum: Option<Vec<serde_yaml::Value>>,
    pub schema: Option<Box<Schema>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "type")]
    pub r#type: Option<TypeOrUnion>,
    pub format: Option<Format>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub r#enum: Option<Vec<serde_yaml::Value>>,
    pub properties: Option<HashMap<String, Properties>>,
    pub example: Option<serde_yaml::Value>,
    pub examples: Option<Vec<String>>,
    #[serde(rename = "$ref")]
    pub r#ref: Option<String>,
    #[serde(rename = "allOf")]
    pub all_of: Option<Vec<ComponentProperties>>,
    #[serde(rename = "oneOf")]
    pub one_of: Option<Vec<ComponentProperties>>,
    pub items: Option<Box<Schema>>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(rename = "minItems")]
    pub min_items: Option<u64>,
    #[serde(rename = "maxItems")]
    pub max_items: Option<u64>,
    #[serde(rename = "minLength")]
    pub min_length: Option<u64>,
    #[serde(rename = "maxLength")]
    pub max_length: Option<u64>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseContent {
    pub schema: Schema,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    #[serde(default)]
    pub required: bool,
    pub content: HashMap<String, BaseContent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SchemaOption {
    OneOf,
    AllOf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentSchemaBase {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<TypeOrUnion>,
    pub items: Option<Box<ComponentSchemaBase>>,
    pub properties: Option<HashMap<String, Properties>>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(rename = "allOf")]
    pub all_of: Option<Vec<ComponentProperties>>,
    #[serde(rename = "oneOf")]
    pub one_of: Option<Vec<ComponentProperties>>,
    #[serde(rename = "minItems")]
    pub min_items: Option<u64>,
    #[serde(rename = "maxItems")]
    pub max_items: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentProperties {
    #[serde(rename = "type")]
    pub r#type: Option<TypeOrUnion>,
    pub description: Option<String>,
    #[serde(default)]
    pub properties: HashMap<String, Properties>,
    #[serde(rename = "$ref")]
    pub r#ref: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Properties {
    #[serde(rename = "type")]
    pub r#type: Option<TypeOrUnion>,
    pub description: Option<String>,
    pub format: Option<Format>,
    pub example: Option<serde_yaml::Value>,
    #[serde(rename = "minLength")]
    pub min_length: Option<u64>,
    #[serde(rename = "maxLength")]
    pub max_length: Option<u64>,
    #[serde(rename = "minItems")]
    pub min_items: Option<u64>,
    #[serde(rename = "maxItems")]
    pub max_items: Option<u64>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub items: Option<Box<Properties>>,
    pub properties: Option<HashMap<String, Properties>>,
    #[serde(default)]
    pub required: Vec<String>,
    pub r#enum: Option<Vec<serde_yaml::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentsObject {
    #[serde(default)]
    pub schemas: HashMap<String, ComponentSchemaBase>,
    #[serde(default)]
    pub parameters: HashMap<String, Parameter>,
    #[serde(rename = "requestBodies", default)]
    pub request_bodies: HashMap<String, Request>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum Type {
    Object,
    String,
    Integer,
    Number,
    Array,
    Boolean,
    Null,
    Binary,
    Base64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypeOrUnion {
    Single(Type),
    Union(Vec<Type>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum In {
    Query,
    Header,
    Path,
    Cookie,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum Format {
    URI,
    #[serde(rename = "uri-reference")]
    URIReference,
    Regex,
    Email,
    Time,
    Date,
    #[serde(rename = "date-time")]
    DateTime,
    UUID,
    Hostname,
    IPV4,
    IPV6,
    Password,
    #[serde(rename = "json-pointer")]
    JsonPointer,
    Binary,
    #[serde(rename = "external-ip")]
    ExternalIP,
    #[serde(rename = "int32")]
    Int32,
    #[serde(rename = "int64")]
    Int64,
    Svg,
    #[serde(rename = "url")]
    Url,
    #[serde(other)]
    Unknown,
}
