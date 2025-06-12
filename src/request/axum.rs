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
use crate::request;
use crate::request::validator::{common_method, ValidateRequest};
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
        common_method(self.path.as_str(), self.inner.method().as_str(), open_api)
    }

    fn query(&self, open_api: &OpenAPI) -> Result<()> {
        let query_str = self.inner.uri().query().unwrap_or_default();

        let query_pairs: HashMap<_, _> = url::form_urlencoded::parse(query_str.as_bytes())
            .into_owned()
            .collect();

        request::validator::common_query(self.path.as_str(), query_pairs, open_api)
    }

    fn path(&self, open_api: &OpenAPI) -> Result<()> {
        if let Some(last_segment) = self.inner.uri().path().rsplit('/').find(|s| !s.is_empty()) {
            request::validator::common_path(self.path.as_str(), last_segment, open_api)?
        }

        Ok(())
    }

    fn body(&self, open_api: &OpenAPI) -> Result<()> {
        if self.body.is_none() {
            return Ok(());
        }
        let body = self
            .body
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing body"))?;
        let request_fields: HashMap<String, Value> = serde_json::from_slice(body)?;
        request::validator::common_body(self.path.as_str(), request_fields, open_api)
    }
}
