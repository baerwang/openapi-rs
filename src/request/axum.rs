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
use crate::model::parse::{Format, In, Method, OpenAPI};
use crate::request::validator::ValidateRequest;
use anyhow::{Context, Result};
use axum::body::{Body, Bytes};
use axum::http::Request;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

#[allow(dead_code)]
pub struct RequestData {
    pub path: String,
    pub inner: Request<Body>,
    pub body: Bytes,
}

impl ValidateRequest for RequestData {
    fn header(&self, _: &OpenAPI) -> Result<()> {
        Ok(())
    }

    fn method(&self, open_api: &OpenAPI) -> Result<()> {
        let path = open_api
            .paths
            .get(self.path.as_str())
            .context("Path not found")?;

        let method =
            Method::from_str(self.inner.method().as_str()).map_err(|e| anyhow::anyhow!(e))?;

        if path.get(&method).is_none() {
            return Err(anyhow::anyhow!("Path is empty"));
        }
        Ok(())
    }

    fn query(&self, open_api: &OpenAPI) -> Result<()> {
        let path = open_api
            .paths
            .get(self.path.as_str())
            .context("Path not found")?;

        let path_base = path
            .get(&Method::Get)
            .context("GET method not defined for this path")?;

        let query_str = self.inner.uri().query().unwrap_or_default();
        let query_pairs: HashMap<_, _> = url::form_urlencoded::parse(query_str.as_bytes())
            .into_owned()
            .collect();

        let mut requireds: HashSet<String> = HashSet::new();

        if let Some(parameters) = &path_base.parameters {
            for parameter in parameters {
                if parameter._in != In::Query {
                    continue;
                }

                if let Some(value) = query_pairs.get(&parameter.name) {
                    validate_format(&parameter.schema.format, value, &parameter.name)?;
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
                                        validate_format(&prop.format, value, key)?;
                                    }
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

        Ok(())
    }

    fn path(&self, open_api: &OpenAPI) -> Result<()> {
        let path = open_api
            .paths
            .get(self.path.as_str())
            .context("Path not found")?;

        let path_base = path
            .get(&Method::Get)
            .context("GET method not defined for this path")?;

        let uri = self.inner.uri();

        if let Some(parameters) = &path_base.parameters {
            if let Some(last_segment) = uri.path().rsplit('/').find(|s| !s.is_empty()) {
                for parameter in parameters {
                    if parameter._in != In::Path {
                        continue;
                    }
                    validate_format(&parameter.schema.format, last_segment, &parameter.name)?;
                }
            }
        }
        Ok(())
    }

    fn body(&self, _: &OpenAPI) -> Result<()> {
        Ok(())
    }
}

fn validate_format(format: &Format, value: &str, key: &str) -> Result<()> {
    match format {
        Format::UUID => {
            uuid::Uuid::parse_str(value).map_err(|_| {
                anyhow::anyhow!(
                    "Invalid UUID format for query parameter '{}': '{}'",
                    key,
                    value
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
