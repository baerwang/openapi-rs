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
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAPI {
    pub openapi: String,
    pub info: InfoObject,
    #[serde(default)]
    pub servers: Vec<ServerObject>,
    pub paths: HashMap<String, HashMap<Method, PathBase>>,
    pub components: Option<ComponentsObject>,
    #[serde(default)]
    pub security: Vec<HashMap<String, SecurityRequirementObject>>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl OpenAPI {
    pub fn yaml(contents: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(contents)
    }

    pub fn validator(&self, valid: impl ValidateRequest) -> Result<(), String> {
        if self.openapi.is_empty() {
            return Err("OpenAPI version is required".to_string());
        }
        if self.info.title.is_empty() {
            return Err("Title is required".to_string());
        }
        if self.info.version.is_empty() {
            return Err("Version is required".to_string());
        }
        if self.paths.is_empty() {
            return Err("Paths are required".to_string());
        }
        if valid.path(self).is_err() {
            return Err("Path validation failed".to_string());
        }
        if valid.query(self).is_err() {
            return Err("Query validation failed".to_string());
        }
        if valid.body(self).is_err() {
            return Err("Body validation failed".to_string());
        }
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
    pub description: String,
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
    pub operation_id: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
    #[serde(rename = "requestBody")]
    pub request: Option<Request>,
    pub responses: Option<HashMap<String, Response>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub _in: In,
    #[serde(default)]
    pub required: bool,
    pub description: Option<String>,
    pub example: Option<serde_yaml::Value>,
    pub schema: Schema,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "type", default)]
    pub _type: Type,
    #[serde(default)]
    pub format: Format,
    pub example: Option<serde_yaml::Value>,
    pub examples: Option<Vec<String>>,
    #[serde(rename = "$ref")]
    pub _ref: Option<String>,
    #[serde(rename = "allOf")]
    pub all_of: Option<Vec<ComponentProperties>>,
    #[serde(rename = "oneOf")]
    pub one_of: Option<Vec<ComponentProperties>>,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub content: HashMap<String, BaseContent>,
    pub description: Option<String>,
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
    #[serde(rename = "type", default)]
    pub _type: Type,
    pub properties: Option<HashMap<String, Properties>>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(rename = "allOf")]
    pub all_of: Option<Vec<ComponentProperties>>,
    #[serde(rename = "oneOf")]
    pub one_of: Option<Vec<ComponentProperties>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentProperties {
    #[serde(rename = "type", default)]
    pub _type: Type,
    pub description: Option<String>,
    #[serde(default)]
    pub properties: HashMap<String, Properties>,
    #[serde(rename = "$ref")]
    pub _ref: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Properties {
    #[serde(rename = "type", default)]
    pub _type: Type,
    pub description: Option<String>,
    #[serde(default)]
    pub format: Format,
    pub example: Option<serde_yaml::Value>,
    #[serde(default)]
    pub minimum: i64,
    #[serde(default)]
    pub maximum: i64,
    pub properties: Option<HashMap<String, Properties>>,
    #[serde(default)]
    pub required: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentsObject {
    pub schemas: HashMap<String, ComponentSchemaBase>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    #[default]
    Undefined,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Patch,
    Options,
    Trace,
}

impl FromStr for Method {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "HEAD" => Ok(Method::Head),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "CONNECT" => Ok(Method::Connect),
            "PATCH" => Ok(Method::Patch),
            "OPTIONS" => Ok(Method::Options),
            "TRACE" => Ok(Method::Trace),
            _ => Err(format!("Invalid method: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum In {
    Query,
    Header,
    Path,
    Cookie,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum Format {
    URI,
    #[serde(rename = "uri-reference")]
    URIReference,
    Regex,
    Email,
    Time,
    Date,
    DateTime,
    UUID,
    Hostname,
    IPV4,
    IPV6,
    Password,
    #[serde(rename = "json-pointer")]
    JsonPointer,
    #[default]
    Undefined,
}
