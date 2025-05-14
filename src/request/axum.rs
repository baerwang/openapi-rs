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
use crate::model::parse::{Method, OpenAPI};
use crate::request::validator::ValidateRequest;
use anyhow::{Context, Result};
use axum::body::{Body, Bytes};
use axum::http::Request;
use std::str::FromStr;

#[allow(dead_code)]
pub struct RequestData {
    inner: Request<Body>,
    body: Bytes,
}

impl ValidateRequest for RequestData {
    fn header(&self, _: &OpenAPI) -> Result<()> {
        Ok(())
    }

    fn method(&self, open_api: &OpenAPI) -> Result<()> {
        let path = open_api
            .paths
            .get(self.inner.uri().path())
            .context("Path not found")?;

        let method =
            Method::from_str(self.inner.method().as_str()).map_err(|e| anyhow::anyhow!(e))?;

        if path.get(&method).is_none() {
            return Err(anyhow::anyhow!("Path is empty"));
        }
        Ok(())
    }

    fn query(&self, open_api: &OpenAPI) -> Result<()> {
        let uri = self.inner.uri();
        let path = open_api.paths.get(uri.path()).context("Path not found")?;

        let _ = path
            .get(&Method::Get)
            .context("GET method not defined for this path")?;

        Ok(())
    }

    fn path(&self, _: &OpenAPI) -> Result<()> {
        Ok(())
    }

    fn body(&self, _: &OpenAPI) -> Result<()> {
        Ok(())
    }
}
