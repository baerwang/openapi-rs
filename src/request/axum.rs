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

use crate::request::validator::ValidateRequest;
use axum::body::{Body, Bytes};
use axum::http::Request;
use std::io::Error;

#[allow(dead_code)]
pub struct RequestData {
    inner: Request<Body>,
    body: Bytes,
}

impl ValidateRequest for RequestData {
    fn header(&self) -> Result<(), Error> {
        Ok(())
    }

    fn query(&self) -> Result<(), Error> {
        Ok(())
    }

    fn path(&self) -> Result<(), Error> {
        Ok(())
    }

    fn body(&self) -> Result<(), Error> {
        Ok(())
    }
}
