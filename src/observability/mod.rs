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

use std::path::Path;
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

/// Log configuration structure
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log file path (optional)
    pub log_file: Option<String>,
    /// Enable console output
    pub console_output: bool,
    /// Show timestamp
    pub show_timestamp: bool,
    /// Show code location information
    pub show_target: bool,
    /// Show thread information
    pub show_thread: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            log_file: None,
            console_output: true,
            show_timestamp: true,
            show_target: false,
            show_thread: false,
        }
    }
}

impl LogConfig {
    /// Create new log configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set log level
    pub fn with_level(mut self, level: &str) -> Self {
        self.level = level.to_string();
        self
    }

    /// Set log file path
    pub fn with_log_file<P: AsRef<Path>>(mut self, file: P) -> Self {
        self.log_file = Some(file.as_ref().to_string_lossy().to_string());
        self
    }

    /// Enable/disable console output
    pub fn with_console_output(mut self, enabled: bool) -> Self {
        self.console_output = enabled;
        self
    }

    /// Enable/disable timestamp display
    pub fn with_timestamp(mut self, enabled: bool) -> Self {
        self.show_timestamp = enabled;
        self
    }

    /// Enable/disable target information display
    pub fn with_target(mut self, enabled: bool) -> Self {
        self.show_target = enabled;
        self
    }

    /// Enable/disable thread information display
    pub fn with_thread(mut self, enabled: bool) -> Self {
        self.show_thread = enabled;
        self
    }
}

/// Initialize logger with default configuration
pub fn init_logger() {
    init_logger_with_config(LogConfig::default());
}

/// Initialize logger with specified configuration
pub fn init_logger_with_config(config: LogConfig) {
    let log_level = match config.level.as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };

    let mut dispatch = fern::Dispatch::new()
        .format(move |out, message, record| {
            let mut format_str = String::new();

            if config.show_timestamp {
                format_str.push_str(&format!(
                    "{} ",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f")
                ));
            }

            format_str.push_str(&format!("[{}]", record.level()));

            if config.show_thread {
                format_str.push_str(&format!(
                    " [{}]",
                    std::thread::current().name().unwrap_or("main")
                ));
            }

            if config.show_target {
                format_str.push_str(&format!(" {}", record.target()));
            }

            format_str.push_str(&format!(" - {message}"));

            out.finish(format_args!("{format_str}"))
        })
        .level(log_level);

    // Console output
    if config.console_output {
        dispatch = dispatch.chain(std::io::stdout());
    }

    // File output
    if let Some(log_file) = &config.log_file {
        // Ensure log file directory exists
        if let Some(parent) = Path::new(log_file).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("Failed to create log directory {parent:?}: {e}");
                return;
            }
        }

        match fern::log_file(log_file) {
            Ok(file) => {
                dispatch = dispatch.chain(file);
            }
            Err(e) => {
                eprintln!("Failed to create log file {log_file}: {e}");
                return;
            }
        }
    }

    // Apply configuration
    if let Err(e) = dispatch.apply() {
        eprintln!("Failed to initialize logger: {e}");
    } else {
        log::info!("Logger initialized with config: {config:?}");
    }
}
