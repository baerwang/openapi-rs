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

mod validator_test;

use crate::model::parse;
use crate::model::parse::{Format, In, Method, OpenAPI};
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, NaiveTime};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

pub trait ValidateRequest {
    fn header(&self, _: &OpenAPI) -> Result<()>;
    fn method(&self, _: &OpenAPI) -> Result<()>;
    fn query(&self, _: &OpenAPI) -> Result<()>;
    fn path(&self, _: &OpenAPI) -> Result<()>;
    fn body(&self, _: &OpenAPI) -> Result<()>;
}

pub fn method(path: &str, method: &str, open_api: &OpenAPI) -> Result<()> {
    let path = open_api.paths.get(path).context("Path not found")?;

    let method = Method::from_str(method).map_err(|e| anyhow::anyhow!(e))?;

    if path.get(&method).is_none() {
        return Err(anyhow::anyhow!("Path is empty"));
    }

    Ok(())
}

pub fn path(path: &str, uri: &str, open_api: &OpenAPI) -> Result<()> {
    let path = open_api.paths.get(path).context("Path not found")?;

    if let Some(path_base) = path.get(&Method::Get) {
        if let Some(parameters) = &path_base.parameters {
            for parameter in parameters {
                if parameter._in != In::Path {
                    continue;
                }
                validate_field_format(
                    &parameter.name,
                    &Value::from(uri),
                    parameter.schema.format.clone(),
                )?;
            }
        }
    }

    Ok(())
}

pub fn query(path: &str, query_pairs: HashMap<String, String>, open_api: &OpenAPI) -> Result<()> {
    let path = open_api.paths.get(path).context("Path not found")?;

    if let Some(path_base) = path.get(&Method::Get) {
        if let Some(parameters) = &path_base.parameters {
            let mut requireds: HashSet<String> = HashSet::new();
            for parameter in parameters {
                if parameter._in != In::Query {
                    continue;
                }

                if let Some(value) = query_pairs.get(&parameter.name) {
                    validate_field_format(
                        &parameter.name,
                        &Value::from(value.as_str()),
                        parameter.schema.format.clone(),
                    )?;
                }

                let mut refs = Vec::new();
                if let Some(r) = &parameter.schema._ref {
                    refs.push(r.as_str());
                }
                if let Some(one_of) = &parameter.schema.one_of {
                    for s in one_of {
                        if let Some(r) = &s._ref {
                            refs.push(r.as_str());
                        }
                    }
                }
                if let Some(all_of) = &parameter.schema.all_of {
                    for s in all_of {
                        if let Some(r) = &s._ref {
                            refs.push(r.as_str());
                        }
                    }
                }

                for schema_ref in refs {
                    if let Some(components) = &open_api.components {
                        if let Some(schema) = components.schemas.get(schema_ref) {
                            if !schema.required.is_empty() {
                                requireds.extend(schema.required.clone());
                            }
                            if let Some(properties) = &schema.properties {
                                for (key, prop) in properties {
                                    if let Some(value) = query_pairs.get(key) {
                                        validate_field_format(
                                            key,
                                            &Value::from(value.as_str()),
                                            prop.format.clone(),
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            for key in &requireds {
                if !query_pairs.contains_key(key) {
                    return Err(anyhow::anyhow!(
                        "Missing required query parameter: '{}'",
                        key
                    ));
                }
            }
        }
    }

    Ok(())
}

pub fn body(path: &str, request_fields: HashMap<String, Value>, open_api: &OpenAPI) -> Result<()> {
    let path = open_api.paths.get(path).context("Path not found")?;

    if let Some(path_base) = path.get(&Method::Post) {
        if let Some(request) = &path_base.request {
            for (key, media_type) in &request.content {
                if let Some(field) = request_fields.get(key) {
                    validate_field_format(key, field, media_type.schema.format.clone())?;
                }
            }

            let mut requireds = HashSet::new();

            let refs = collect_schema_refs(&request.content);

            if let Some(components) = &open_api.components {
                for schema_ref in refs {
                    if let Some(last_slash_pos) = schema_ref.rfind('/') {
                        let filename = &schema_ref[last_slash_pos + 1..];
                        if let Some(schema) = components.schemas.get(filename) {
                            requireds.extend(schema.required.iter().cloned());
                            if let Some(properties) = &schema.properties {
                                validate_schema_properties(&request_fields, properties)?;
                            }
                        }
                    } else {
                        return Err(anyhow::anyhow!(
                            "Invalid schema reference: '{}'",
                            schema_ref
                        ));
                    }
                }
            }

            for key in &requireds {
                if !request_fields.contains_key(key) {
                    return Err(anyhow::anyhow!(
                        "Missing required query parameter: '{}'",
                        key
                    ));
                }
            }
        }
    }

    Ok(())
}

fn validate_field_format(key: &str, value: &Value, format: Format) -> Result<()> {
    let str_val = value
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("this value must string '{}'", key))?;
    match format {
        Format::Email => {
            if !validator::validate_email(str_val) {
                return Err(anyhow::anyhow!(
                    "Invalid email format for query parameter '{}': '{}'",
                    key,
                    str_val
                ));
            }
        }
        Format::Time => {
            NaiveTime::parse_from_str(str_val, "%H:%M:%S").map_err(|_| {
                anyhow::anyhow!(
                    "Invalid time format for query parameter '{}': '{}'",
                    key,
                    str_val
                )
            })?;
        }
        Format::Date => {
            NaiveDate::parse_from_str(str_val, "%Y-%m-%d").map_err(|_| {
                anyhow::anyhow!(
                    "Invalid RFC3339 full-date for query parameter '{}': '{}'",
                    key,
                    str_val
                )
            })?;
        }
        Format::DateTime => {
            DateTime::parse_from_rfc3339(str_val).map_err(|_| {
                anyhow::anyhow!(
                    "Invalid datetime format for query parameter '{}': '{}'",
                    key,
                    str_val
                )
            })?;
        }
        Format::UUID => {
            uuid::Uuid::parse_str(str_val).map_err(|_| {
                anyhow::anyhow!(
                    "Invalid UUID format for query parameter '{}': '{}'",
                    key,
                    str_val
                )
            })?;
        }
        Format::IPV4 => {
            str_val.parse::<Ipv4Addr>().map_err(|_| {
                anyhow::anyhow!(
                    "Invalid IPv4 format for query parameter '{}': '{}'",
                    key,
                    str_val
                )
            })?;
        }
        Format::IPV6 => {
            str_val.parse::<Ipv6Addr>().map_err(|_| {
                anyhow::anyhow!(
                    "Invalid IPv6 format for query parameter '{}': '{}'",
                    key,
                    str_val
                )
            })?;
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported format '{:?}' for query parameter '{}'",
                format,
                key
            ));
        }
    }
    Ok(())
}

fn collect_schema_refs(content: &HashMap<String, parse::BaseContent>) -> Vec<&str> {
    let mut refs = Vec::new();
    for media_type in content.values() {
        if let Some(r) = &media_type.schema._ref {
            refs.push(r.as_str());
        }
        if let Some(one_of) = &media_type.schema.one_of {
            refs.extend(one_of.iter().filter_map(|s| s._ref.as_deref()));
        }
        if let Some(all_of) = &media_type.schema.all_of {
            refs.extend(all_of.iter().filter_map(|s| s._ref.as_deref()));
        }
    }
    refs
}

fn validate_schema_properties(
    request_fields: &HashMap<String, Value>,
    properties: &HashMap<String, parse::Properties>,
) -> Result<()> {
    for (key, prop) in properties {
        if let Some(value) = request_fields.get(key) {
            validate_field_format(key, value, prop.format.clone())?;
        }
    }
    Ok(())
}
