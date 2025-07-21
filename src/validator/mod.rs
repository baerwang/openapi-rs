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

mod enum_test;
mod validator_test;

use crate::model::parse;
use crate::model::parse::{
    ComponentsObject, Format, In, OpenAPI, Properties, Request, Type, TypeOrUnion,
};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, NaiveDate, NaiveTime};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::string::String;

pub trait ValidateRequest {
    fn header(&self, _: &OpenAPI) -> Result<()>;
    fn method(&self, _: &OpenAPI) -> Result<()>;
    fn query(&self, _: &OpenAPI) -> Result<()>;
    fn path(&self, _: &OpenAPI) -> Result<()>;
    fn body(&self, _: &OpenAPI) -> Result<()>;
}

pub fn method(path: &str, method: &str, open_api: &OpenAPI) -> Result<()> {
    let path = open_api.paths.get(path).context("Path not found")?;

    if !path.operations.contains_key(method) {
        return Err(anyhow::anyhow!("Path is empty"));
    }

    Ok(())
}

pub fn path(path: &str, uri: &str, open_api: &OpenAPI) -> Result<()> {
    let path_item = open_api.paths.get(path).context("Path not found")?;
    let empty_vec = vec![];
    let parameters = path_item
        .operations
        .get("get")
        .and_then(|p| p.parameters.as_ref())
        .unwrap_or(&empty_vec);

    for parameter in parameters {
        if parameter.r#ref.is_some() {
            // TODO: handle parameter references
            continue;
        }

        if let (Some(name), Some(r#in)) = (&parameter.name, &parameter.r#in) {
            if *r#in != In::Path {
                continue;
            }
            if let Some(schema) = &parameter.schema {
                validate_field_format(name, &Value::from(uri), schema.format.clone())?;
            }
        }
    }

    Ok(())
}

fn process_schema_refs(
    schema: &parse::Schema,
    fields: &Map<String, Value>,
    requireds: &mut HashSet<String>,
    open_api: &OpenAPI,
) -> Result<()> {
    if let Some(components) = &open_api.components {
        for schema_ref in collect_refs(schema) {
            requireds.extend(extract_required_and_validate_props(
                fields, schema_ref, components,
            )?);
        }
    }
    Ok(())
}

fn validate_required_fields(
    requireds: &HashSet<String>,
    query_pairs: &HashMap<String, String>,
) -> Result<()> {
    for key in requireds {
        if !query_pairs.contains_key(key) {
            return Err(anyhow!("Missing required query parameter: '{}'", key));
        }
    }
    Ok(())
}

pub fn query(path: &str, query_pairs: HashMap<String, String>, open_api: &OpenAPI) -> Result<()> {
    let path_base = open_api.paths.get(path).context("Path not found")?;
    let empty_vec = vec![];
    let parameters = path_base
        .operations
        .get("get")
        .and_then(|p| p.parameters.as_ref())
        .unwrap_or(&empty_vec);

    let fields: Map<String, Value> = query_pairs
        .iter()
        .map(|(k, v)| (k.clone(), Value::from(v.clone())))
        .collect();

    let mut requireds: HashSet<String> = HashSet::new();

    for parameter in parameters {
        if let Some(param_ref) = &parameter.r#ref {
            if let Some(components) = &open_api.components {
                requireds.extend(extract_required_and_validate_props(
                    &fields, param_ref, components,
                )?);
            }
        }

        let name = match (&parameter.name, &parameter.r#in) {
            (Some(name), Some(In::Query)) => name,
            _ => continue,
        };

        match query_pairs.get(name) {
            Some(value) => {
                if parameter.required && value.trim().is_empty() {
                    return Err(anyhow!("This field [{}] is required", name));
                }

                let json_value = Value::from(value.as_str());

                if let Some(enum_values) = &parameter.r#enum {
                    validate_enum_value(name, &json_value, enum_values)?;
                }

                if let Some(schema) = &parameter.schema {
                    validate_field_format(name, &json_value, schema.format.clone())?;

                    if let Some(enum_values) = &schema.r#enum {
                        validate_enum_value(name, &json_value, enum_values)?;
                    }

                    process_schema_refs(schema, &fields, &mut requireds, open_api)?;
                }
            }
            None => {
                if parameter.required {
                    return Err(anyhow!("This field [{}] is required", name));
                }
            }
        }
    }

    validate_required_fields(&requireds, &query_pairs)?;

    Ok(())
}

pub fn body(path: &str, fields: Value, open_api: &OpenAPI) -> Result<()> {
    let path_base = open_api.paths.get(path).context("Path not found")?;
    let request = path_base
        .operations
        .get("post")
        .and_then(|p| p.request.as_ref());

    if let Some(request) = request {
        let refs: Vec<&str> = request
            .content
            .values()
            .flat_map(|media| collect_refs(&media.schema))
            .collect();

        let schema_info = get_schema_info(&refs, open_api);
        let expected_type = schema_info
            .as_ref()
            .and_then(|schema| schema.r#type.clone());

        match fields {
            Value::Object(ref map) => {
                ensure_type(&expected_type, Type::Object)?;
                validate_map(map, request, &refs, open_api)?;
            }
            Value::Array(ref arr) => {
                ensure_type(&expected_type, Type::Array)?;

                if let Some(schema) = &schema_info {
                    validate_array_length_with_schema(arr.len(), schema)?;
                }

                validate_array_items(arr, request, &refs, open_api)?;
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported request body type: must be object or array"
                ));
            }
        }
    }

    Ok(())
}

fn get_schema_info<'a>(
    refs: &[&str],
    open_api: &'a OpenAPI,
) -> Option<&'a parse::ComponentSchemaBase> {
    open_api.components.as_ref().and_then(|components| {
        refs.iter().find_map(|schema_ref| {
            schema_ref
                .rsplit('/')
                .next()
                .and_then(|schema_name| components.schemas.get(schema_name))
        })
    })
}

fn validate_array_items(
    arr: &[Value],
    request: &Request,
    refs: &[&str],
    open_api: &OpenAPI,
) -> Result<()> {
    for (index, item) in arr.iter().enumerate() {
        let map = item
            .as_object()
            .with_context(|| format!("Array item at index {index} must be an object"))?;
        validate_map(map, request, refs, open_api)?;
    }
    Ok(())
}

fn validate_array_length_with_schema(
    length: usize,
    schema: &parse::ComponentSchemaBase,
) -> Result<()> {
    if let Some(min) = schema.min_items {
        if length < min as usize {
            return Err(anyhow!(
                "The array must have at least {} items, but got {}",
                min,
                length
            ));
        }
    }

    if let Some(max) = schema.max_items {
        if length > max as usize {
            return Err(anyhow!(
                "The array must have at most {} items, but got {}",
                max,
                length
            ));
        }
    }

    Ok(())
}

fn ensure_type(actual: &Option<TypeOrUnion>, expected: Type) -> Result<()> {
    if let Some(type_or_union) = actual {
        match type_or_union {
            TypeOrUnion::Single(t) => {
                if *t != expected {
                    return Err(anyhow!(
                        "Expected request body to be a {:?}, got {:?}",
                        expected,
                        t
                    ));
                }
            }
            TypeOrUnion::Union(types) => {
                if !types.contains(&expected) {
                    return Err(anyhow!(
                        "Expected request body to be a {:?}, but union types {:?} don't include it",
                        expected,
                        types
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_map(
    fields: &Map<String, Value>,
    request: &Request,
    refs: &[&str],
    open_api: &OpenAPI,
) -> Result<()> {
    for (key, media_type) in &request.content {
        if let Some(field) = fields.get(key) {
            let type_or_union = media_type.schema.r#type.clone();
            validate_field_type(key, field, type_or_union)?;
            if media_type.schema.r#type == Some(TypeOrUnion::Single(Type::String)) {
                validate_field_format(key, field, media_type.schema.format.clone())?;
            }
        }
    }

    let mut requireds = HashSet::new();

    if let Some(components) = &open_api.components {
        for schema_ref in refs {
            requireds.extend(extract_required_and_validate_props(
                fields, schema_ref, components,
            )?);
        }
    }

    for key in &requireds {
        if !fields.contains_key(key) {
            return Err(anyhow!("Missing required request body field: '{}'", key));
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

fn validate_enum_value(key: &str, value: &Value, enum_values: &[serde_yaml::Value]) -> Result<()> {
    for enum_val in enum_values {
        if values_equal(value, enum_val) {
            return Ok(());
        }
    }

    let enum_strings: Vec<String> = enum_values.iter().map(format_yaml_value).collect();

    Err(anyhow!(
        "Value '{}' for field '{}' is not in allowed enum values: [{}]",
        format_json_value(value),
        key,
        enum_strings.join(", ")
    ))
}

fn values_equal(json_val: &Value, yaml_val: &serde_yaml::Value) -> bool {
    match (json_val, yaml_val) {
        (Value::String(s1), serde_yaml::Value::String(s2)) => s1 == s2,
        (Value::Number(n1), serde_yaml::Value::Number(n2)) => {
            if let (Some(i1), Some(i2)) = (n1.as_i64(), n2.as_i64()) {
                i1 == i2
            } else if let (Some(f1), Some(f2)) = (n1.as_f64(), n2.as_f64()) {
                (f1 - f2).abs() < f64::EPSILON
            } else {
                false
            }
        }
        (Value::Bool(b1), serde_yaml::Value::Bool(b2)) => b1 == b2,
        (Value::Null, serde_yaml::Value::Null) => true,
        (Value::String(s), serde_yaml::Value::Number(n)) => {
            if let Ok(parsed_int) = s.parse::<i64>() {
                if let Some(yaml_int) = n.as_i64() {
                    return parsed_int == yaml_int;
                }
            }
            if let Ok(parsed_float) = s.parse::<f64>() {
                if let Some(yaml_float) = n.as_f64() {
                    return (parsed_float - yaml_float).abs() < f64::EPSILON;
                }
            }
            false
        }
        (Value::String(s), serde_yaml::Value::Bool(b)) => match s.to_lowercase().as_str() {
            "true" => *b,
            "false" => !*b,
            _ => false,
        },
        (Value::Number(n), serde_yaml::Value::String(s)) => {
            if let Some(int_val) = n.as_i64() {
                s == &int_val.to_string()
            } else if let Some(float_val) = n.as_f64() {
                s == &float_val.to_string()
            } else {
                false
            }
        }
        (Value::Bool(b), serde_yaml::Value::String(s)) => match s.to_lowercase().as_str() {
            "true" => *b,
            "false" => !*b,
            _ => false,
        },
        _ => false,
    }
}

fn format_yaml_value(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => format!("\"{s}\""),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => "null".to_string(),
        _ => format!("{value:?}"),
    }
}

fn format_json_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{s}\""),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => format!("{value:?}"),
    }
}
fn validate_field_type(key: &str, value: &Value, field_type: Option<TypeOrUnion>) -> Result<()> {
    use Type::*;

    match field_type {
        Some(TypeOrUnion::Single(Object)) => {
            if !value.is_object() {
                return Err(anyhow!("the value of '{}' must be an Object", key));
            }
        }
        Some(TypeOrUnion::Single(String)) => {
            if !value.is_string() {
                return Err(anyhow!("the value of '{}' must be a String", key));
            }
        }
        Some(TypeOrUnion::Single(Integer)) => {
            if !value.is_i64() {
                return Err(anyhow!("the value of '{}' must be an Integer", key));
            }
        }
        Some(TypeOrUnion::Single(Number)) => {
            if !value.is_number() {
                return Err(anyhow!("the value of '{}' must be a Number", key));
            }
        }
        Some(TypeOrUnion::Single(Array)) => {
            if !value.is_array() {
                return Err(anyhow!("the value of '{}' must be an Array", key));
            }
        }
        Some(TypeOrUnion::Single(Boolean)) => {
            if !value.is_boolean() {
                return Err(anyhow!("the value of '{}' must be a Boolean", key));
            }
        }
        Some(TypeOrUnion::Single(Null)) => {
            if !value.is_null() {
                return Err(anyhow!("the value of '{}' must be Null", key));
            }
        }
        Some(TypeOrUnion::Single(Base64)) => {
            let str_val = value
                .as_str()
                .ok_or_else(|| anyhow!("the value of '{}' must be a string", key))?;

            if str_val.trim().is_empty() {
                return Err(anyhow!("the value of '{}' must not be empty", key));
            }

            if general_purpose::STANDARD.decode(str_val).is_err() {
                return Err(anyhow!("the value of '{}' must be valid Base64", key));
            }
        }
        Some(TypeOrUnion::Single(Binary)) => {
            if !value.is_string() {
                return Err(anyhow!(
                    "the value of '{}' must be a String for binary data",
                    key
                ));
            }
        }
        Some(TypeOrUnion::Union(types)) => {
            let mut valid = false;
            for single_type in types {
                if validate_single_type_match(value, &single_type) {
                    valid = true;
                    break;
                }
            }
            if !valid {
                return Err(anyhow!(
                    "the value of '{}' must match one of the union types",
                    key
                ));
            }
        }
        None => {}
    }

    Ok(())
}

fn validate_single_type_match(value: &Value, field_type: &Type) -> bool {
    use Type::*;
    match field_type {
        Object => value.is_object(),
        String => value.is_string(),
        Integer => value.is_i64(),
        Number => value.is_number(),
        Array => value.is_array(),
        Boolean => value.is_boolean(),
        Null => value.is_null(),
        Base64 => {
            if let Some(str_val) = value.as_str() {
                !str_val.trim().is_empty() && general_purpose::STANDARD.decode(str_val).is_ok()
            } else {
                false
            }
        }
        Binary => value.is_string(),
    }
}

fn validate_field_length_limit(key: &str, value: &Value, properties: &Properties) -> Result<()> {
    use TypeOrUnion::*;

    match &properties.r#type {
        Some(Single(type_)) => {
            validate_single_type(key, value, type_, properties)?;
        }
        Some(Union(types)) => {
            validate_union_types(key, value, types, properties)?;
        }
        None => {}
    }

    Ok(())
}

fn validate_single_type(
    key: &str,
    value: &Value,
    type_: &Type,
    properties: &Properties,
) -> Result<()> {
    use Type::*;

    match type_ {
        String | Base64 | Binary => {
            let str_val = value
                .as_str()
                .ok_or_else(|| anyhow!("The value of '{}' must be a String", key))?;
            validate_string_length(key, str_val, properties)?;
        }
        Integer => {
            let int_val = value
                .as_i64()
                .ok_or_else(|| anyhow!("The value of '{}' must be an Integer", key))?;
            validate_numeric_range(key, int_val as f64, properties)?;
        }
        Number => {
            let num_val = value
                .as_f64()
                .ok_or_else(|| anyhow!("The value of '{}' must be a Number", key))?;
            validate_numeric_range(key, num_val, properties)?;
        }
        Array => {
            if !value.is_array() {
                return Err(anyhow!("The value of '{}' must be an Array", key));
            }
            let arr_len = value.as_array().unwrap().len();
            validate_array_length(key, arr_len, properties)?;
        }
        Boolean => {
            if !value.is_boolean() {
                return Err(anyhow!("The value of '{}' must be a Boolean", key));
            }
        }
        Null => {
            if !value.is_null() {
                return Err(anyhow!("The value of '{}' must be null", key));
            }
        }
        Object => {
            if !value.is_object() {
                return Err(anyhow!("The value of '{}' must be an Object", key));
            }
        }
    }

    Ok(())
}

fn validate_union_types(
    key: &str,
    value: &Value,
    types: &[Type],
    properties: &Properties,
) -> Result<()> {
    let mut validation_errors = Vec::new();
    let mut type_matched = false;

    for type_ in types {
        match validate_single_type(key, value, type_, properties) {
            Ok(()) => {
                type_matched = true;
                break;
            }
            Err(e) => {
                validation_errors.push(e.to_string());
            }
        }
    }

    if !type_matched {
        let type_names: Vec<String> = types.iter().map(|t| format!("{t:?}")).collect();
        return Err(anyhow!(
            "The value of '{}' does not match any of the union types [{}]. Validation errors: {}",
            key,
            type_names.join(", "),
            validation_errors.join("; ")
        ));
    }

    Ok(())
}

fn validate_string_length(key: &str, str_val: &str, properties: &Properties) -> Result<()> {
    let length = str_val.len();

    if let Some(min) = properties.min_length {
        if length < min as usize {
            return Err(anyhow!(
                "The length of '{}' must be at least {} characters, but got {}",
                key,
                min,
                length
            ));
        }
    }

    if let Some(max) = properties.max_length {
        if length > max as usize {
            return Err(anyhow!(
                "The length of '{}' must be at most {} characters, but got {}",
                key,
                max,
                length
            ));
        }
    }

    Ok(())
}

fn validate_numeric_range(key: &str, value: f64, properties: &Properties) -> Result<()> {
    if let Some(min) = properties.minimum {
        if value < min {
            return Err(anyhow!(
                "The value of '{}' must be >= {}, but got {}",
                key,
                min,
                value
            ));
        }
    }

    if let Some(max) = properties.maximum {
        if value > max {
            return Err(anyhow!(
                "The value of '{}' must be <= {}, but got {}",
                key,
                max,
                value
            ));
        }
    }

    Ok(())
}

fn validate_array_length(key: &str, length: usize, properties: &Properties) -> Result<()> {
    if let Some(min) = properties.min_items {
        if length < min as usize {
            return Err(anyhow!(
                "The array '{}' must have at least {} items, but got {}",
                key,
                min,
                length
            ));
        }
    }

    if let Some(max) = properties.max_items {
        if length > max as usize {
            return Err(anyhow!(
                "The array '{}' must have at most {} items, but got {}",
                key,
                max,
                length
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
    fields: &Map<String, Value>,
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
        validate_properties(fields, &schema.properties)?;

        if let Some(items) = &schema.items {
            requireds.extend(items.required.iter().cloned());
            validate_properties(fields, &items.properties)?;
        }
    }

    Ok(requireds)
}

fn validate_properties(
    fields: &Map<String, Value>,
    properties: &Option<HashMap<String, Properties>>,
) -> Result<()> {
    if let Some(properties) = properties {
        for (key, prop) in properties {
            if let Some(value) = fields.get(key) {
                validate_field_type(key, value, prop.r#type.clone())?;

                if let Some(TypeOrUnion::Single(Type::String)) = prop.r#type {
                    validate_field_format(key, value, prop.format.clone())?;
                }

                if let Some(enum_values) = &prop.r#enum {
                    validate_enum_value(key, value, enum_values)?;
                }

                validate_field_length_limit(key, value, prop)?;
            }
            validate_properties(fields, &prop.properties)?;
        }
    }

    Ok(())
}

fn collect_refs(schema: &parse::Schema) -> Vec<&str> {
    let mut refs = Vec::new();
    if let Some(r) = &schema.r#ref {
        refs.push(r.as_str());
    }
    if let Some(one_of) = &schema.one_of {
        for s in one_of {
            if let Some(r) = &s.r#ref {
                refs.push(r.as_str());
            }
        }
    }
    if let Some(all_of) = &schema.all_of {
        for s in all_of {
            if let Some(r) = &s.r#ref {
                refs.push(r.as_str());
            }
        }
    }
    refs
}
