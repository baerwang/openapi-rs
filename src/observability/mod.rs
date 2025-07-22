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

use std::time::Instant;

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub method: String,
    pub path: String,
}

impl RequestContext {
    pub fn new(method: String, path: String) -> Self {
        Self { method, path }
    }
}

pub struct ValidationMetrics {
    start_time: Instant,
    method: String,
    path: String,
}

impl ValidationMetrics {
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            start_time: Instant::now(),
            method: method.to_string(),
            path: path.to_string(),
        }
    }

    pub fn from_context(context: &RequestContext) -> Self {
        Self::new(&context.method, &context.path)
    }

    pub fn record_success(self) {
        let duration_ms = self.start_time.elapsed().as_millis();
        let timestamp = chrono::Utc::now().timestamp_millis();

        log::info!(
            "openapi_validation method=\"{}\" path=\"{}\" success=true duration_ms={} timestamp={}",
            self.method,
            self.path,
            duration_ms,
            timestamp
        );
    }

    pub fn record_failure(self, error: String) {
        let duration_ms = self.start_time.elapsed().as_millis();
        let timestamp = chrono::Utc::now().timestamp_millis();

        log::warn!(
            "openapi_validation method=\"{}\" path=\"{}\" success=false duration_ms={} error=\"{}\" timestamp={}",
            self.method,
            self.path,
            duration_ms,
            error,
            timestamp
        );
    }
}

pub fn init_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}
