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
use crate::model::parse::{ComponentsObject, Format, In, Method, OpenAPI};
use anyhow::{anyhow, Context, Result};
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
    let path_item = open_api.paths.get(path).context("Path not found")?;
    let empty_vec = vec![];
    let parameters = path_item
        .get(&Method::Get)
        .and_then(|p| p.parameters.as_ref())
        .unwrap_or(&empty_vec);

    for parameter in parameters {
        if parameter._in != In::Path {
            continue;
        }

        validate_field_format(
            &parameter.name,
            &Value::from(uri),
            parameter.schema.format.clone(),
        )?
    }

    Ok(())
}

pub fn query(path: &str, query_pairs: HashMap<String, String>, open_api: &OpenAPI) -> Result<()> {
    let path_base = open_api.paths.get(path).context("Path not found")?;
    let empty_vec = vec![];
    let parameters = path_base
        .get(&Method::Get)
        .and_then(|p| p.parameters.as_ref())
        .unwrap_or(&empty_vec);

    let mut requireds: HashSet<String> = HashSet::new();

    for parameter in parameters {
        if parameter._in != In::Query {
            continue;
        }

        match query_pairs.get(&parameter.name) {
            Some(value) => {
                if parameter.required && value.is_empty() {
                    return Err(anyhow!("This field [{}] is required", parameter.name));
                }

                validate_field_format(
                    &parameter.name,
                    &Value::from(value.as_str()),
                    parameter.schema.format.clone(),
                )?;
            }
            None if parameter.required => {
                return Err(anyhow!("This field [{}] is required", parameter.name));
            }
            _ => {}
        }

        for schema_ref in collect_refs(&parameter.schema) {
            if let Some(components) = &open_api.components {
                let fields: HashMap<String, Value> = query_pairs
                    .iter()
                    .map(|(k, v)| (k.clone(), Value::from(v.clone())))
                    .collect();
                requireds.extend(extract_required_and_validate_props(
                    &fields, schema_ref, components,
                )?);
            }
        }
    }

    for key in &requireds {
        if !query_pairs.contains_key(key) {
            return Err(anyhow!("Missing required query parameter: '{}'", key));
        }
    }

    Ok(())
}

pub fn body(path: &str, request_fields: HashMap<String, Value>, open_api: &OpenAPI) -> Result<()> {
    let path_base = open_api.paths.get(path).context("Path not found")?;
    let request = path_base
        .get(&Method::Post)
        .and_then(|p| p.request.as_ref());

    if let Some(request) = request {
        for (key, media_type) in &request.content {
            if let Some(field) = request_fields.get(key) {
                validate_field_format(key, field, media_type.schema.format.clone())?;
            }
        }

        let refs: Vec<&str> = request
            .content
            .values()
            .flat_map(|media| collect_refs(&media.schema))
            .collect();

        let mut requireds = HashSet::new();

        if let Some(components) = &open_api.components {
            for schema_ref in refs {
                requireds.extend(extract_required_and_validate_props(
                    &request_fields,
                    schema_ref,
                    components,
                )?);
            }
        }

        for key in &requireds {
            if !request_fields.contains_key(key) {
                return Err(anyhow!("Missing required request body field: '{}'", key));
            }
        }
    }

    Ok(())
}

fn validate_field_format(key: &str, value: &Value, format: Option<Format>) -> Result<()> {
    let str_val = match value.as_str() {
        Some(s) => s,
        None => return Err(anyhow::anyhow!("this value must be string '{}'", key)),
    };
    match format {
        Some(Format::Email) => {
            if !validator::validate_email(str_val) {
                return Err(format_error("Email", key, str_val));
            }
        }
        Some(Format::Time) => {
            NaiveTime::parse_from_str(str_val, "%H:%M:%S")
                .map_err(|_| format_error("Time", key, str_val))?;
        }
        Some(Format::Date) => {
            NaiveDate::parse_from_str(str_val, "%Y-%m-%d")
                .map_err(|_| format_error("Date", key, str_val))?;
        }
        Some(Format::DateTime) => {
            DateTime::parse_from_rfc3339(str_val)
                .map_err(|_| format_error("DateTime", key, str_val))?;
        }
        Some(Format::UUID) => {
            uuid::Uuid::parse_str(str_val).map_err(|_| format_error("UUID", key, str_val))?;
        }
        Some(Format::IPV4) => {
            str_val
                .parse::<Ipv4Addr>()
                .map_err(|_| format_error("IPv4", key, str_val))?;
        }
        Some(Format::IPV6) => {
            str_val
                .parse::<Ipv6Addr>()
                .map_err(|_| format_error("IPV6", key, str_val))?;
        }
        None => {}
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

fn format_error(kind: &str, key: &str, value: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "Invalid {} format for query parameter '{}': '{}'",
        kind,
        key,
        value
    )
}

fn extract_required_and_validate_props(
    input_fields: &HashMap<String, Value>,
    schema_ref: &str,
    components: &ComponentsObject,
) -> Result<HashSet<String>> {
    let filename = schema_ref
        .rsplit('/')
        .next()
        .ok_or_else(|| anyhow!("Invalid schema reference: '{}'", schema_ref))?;

    let mut requireds = HashSet::new();

    if let Some(schema) = components.schemas.get(filename) {
        requireds.extend(schema.required.iter().cloned());

        if let Some(properties) = &schema.properties {
            for (key, prop) in properties {
                if let Some(value) = input_fields.get(key) {
                    validate_field_format(key, value, prop.format.clone())?;
                }
            }
        }
    }

    Ok(requireds)
}

fn collect_refs(schema: &parse::Schema) -> Vec<&str> {
    let mut refs = Vec::new();
    if let Some(r) = &schema._ref {
        refs.push(r.as_str());
    }
    if let Some(one_of) = &schema.one_of {
        for s in one_of {
            if let Some(r) = &s._ref {
                refs.push(r.as_str());
            }
        }
    }
    if let Some(all_of) = &schema.all_of {
        for s in all_of {
            if let Some(r) = &s._ref {
                refs.push(r.as_str());
            }
        }
    }
    refs
}
