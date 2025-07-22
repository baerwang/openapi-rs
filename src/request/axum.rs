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

use crate::model::parse::OpenAPI;
use crate::validator::{body, method, path, query, ValidateRequest};
use anyhow::Result;
use axum::body::{Body, Bytes};
use axum::http::Request;
use serde_json::Value;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct RequestData {
    pub path: String,
    pub inner: Request<Body>,
    pub body: Option<Bytes>,
}

impl ValidateRequest for RequestData {
    fn header(&self, _: &OpenAPI) -> Result<()> {
        Ok(())
    }

    fn method(&self, open_api: &OpenAPI) -> Result<()> {
        method(
            self.path.as_str(),
            self.inner.method().to_string().to_lowercase().as_str(),
            open_api,
        )
    }

    fn query(&self, open_api: &OpenAPI) -> Result<()> {
        let uri_parts: Vec<&str> = self
            .inner
            .uri()
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("")
            .split('?')
            .collect();

        let query_pairs = if uri_parts.len() > 1 {
            uri_parts[1]
                .split('&')
                .filter_map(|pair| {
                    let mut split = pair.split('=');
                    match (split.next(), split.next()) {
                        (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                        _ => None,
                    }
                })
                .collect()
        } else {
            HashMap::new()
        };

        query(self.path.as_str(), &query_pairs, open_api)
    }

    fn path(&self, open_api: &OpenAPI) -> Result<()> {
        if let Some(last_segment) = self.inner.uri().path().rsplit('/').find(|s| !s.is_empty()) {
            path(self.path.as_str(), last_segment, open_api)?
        }

        Ok(())
    }

    fn body(&self, open_api: &OpenAPI) -> Result<()> {
        if self.body.is_none() {
            return Ok(());
        }
        let self_body = self
            .body
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing body"))?;
        let request_fields: Value = serde_json::from_slice(self_body)?;
        body(self.path.as_str(), request_fields, open_api)
    }
}
