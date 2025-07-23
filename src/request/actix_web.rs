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
use crate::observability::RequestContext;
use crate::validator::{body, method, path, query, ValidateRequest};
use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    web::{Bytes, BytesMut},
    Error, HttpMessage, HttpRequest,
};
use anyhow::Result;
use futures_util::{future::LocalBoxFuture, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;

#[allow(dead_code)]
pub struct RequestData {
    pub path: String,
    pub method: String,
    pub query_string: String,
    pub body: Option<Bytes>,
}

impl ValidateRequest for RequestData {
    fn header(&self, _: &OpenAPI) -> Result<()> {
        Ok(())
    }

    fn method(&self, open_api: &OpenAPI) -> Result<()> {
        method(self.path.as_str(), self.method.as_str(), open_api)
    }

    fn query(&self, open_api: &OpenAPI) -> Result<()> {
        let query_pairs: HashMap<String, String> = if !self.query_string.is_empty() {
            self.query_string
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
        if let Some(last_segment) = self.path.rsplit('/').find(|s| !s.is_empty()) {
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

    fn context(&self) -> RequestContext {
        RequestContext::new(self.method.clone(), self.path.clone())
    }
}

/// OpenAPI validates middleware
///
/// Provides request validation based on OpenAPI specifications, supporting path, method, query parameters, and request body validation.
///
/// # example
///
/// ```rust
/// use actix_web::{web, App, HttpServer, HttpResponse, Result};
/// use openapi_rs::request::actix_web::OpenApiValidation;
///
/// async fn create_user() -> Result<HttpResponse> {
///     Ok(HttpResponse::Ok().json(serde_json::json!({"status": "created"})))
/// }
///
/// #[actix_web::main]
/// async fn main() -> Result<()> {
///     let yaml_content = include_str!("api.yaml");
///     let validation = OpenApiValidation::from_yaml(yaml_content)?;
///
///     HttpServer::new(move || {
///         App::new()
///             .wrap(validation.clone())
///             .route("/api/users", web::post().to(create_user))
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OpenApiValidation {
    openapi: Arc<OpenAPI>,
}

impl OpenApiValidation {
    pub fn new(openapi: OpenAPI) -> Self {
        Self {
            openapi: Arc::new(openapi),
        }
    }

    pub fn from_yaml(yaml_content: &str) -> Result<Self> {
        let openapi: OpenAPI = serde_yaml::from_str(yaml_content)?;
        Ok(Self::new(openapi))
    }
}

impl<S, B> Transform<S, ServiceRequest> for OpenApiValidation
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = OpenApiValidationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(OpenApiValidationMiddleware {
            service: Rc::new(service),
            openapi: self.openapi.clone(),
        }))
    }
}

pub struct OpenApiValidationMiddleware<S> {
    service: Rc<S>,
    openapi: Arc<OpenAPI>,
}

impl<S, B> Service<ServiceRequest> for OpenApiValidationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let openapi = Arc::clone(&self.openapi);

        Box::pin(async move {
            let path = req.path().to_string();
            let method = req.method().as_str().to_lowercase();
            let query_string = req.query_string().to_string();

            let (http_req, payload) = req.into_parts();

            let mut req_body = None;

            if Self::should_extract_body(&http_req) {
                match Self::extract_body_safely(payload, &http_req).await {
                    Ok(body) => req_body = body,
                    Err(e) => {
                        let error_req =
                            ServiceRequest::from_parts(http_req, Payload::from(Vec::<u8>::new()));
                        return Ok(error_req.error_response(e).map_into_right_body());
                    }
                }
            }

            let request_data = RequestData {
                path: path.clone(),
                method,
                query_string,
                body: req_body.clone(),
            };

            let rebuild_service_request = |http_req: HttpRequest, req_body: &Option<Bytes>| {
                if let Some(ref body_bytes) = req_body {
                    let req =
                        ServiceRequest::from_parts(http_req, Payload::from(body_bytes.clone()));
                    req.extensions_mut().insert(body_bytes.clone());
                    req
                } else {
                    ServiceRequest::from_parts(http_req, Payload::from(Vec::<u8>::new()))
                }
            };

            if let Err(e) = openapi.validator(request_data) {
                let validation_error =
                    actix_web::error::ErrorBadRequest(format!("OpenAPI validation failed: {e}"));

                let service_req = rebuild_service_request(http_req, &req_body);
                return Ok(service_req
                    .error_response(validation_error)
                    .map_into_right_body());
            }

            let service_req = rebuild_service_request(http_req, &req_body);

            service
                .call(service_req)
                .await
                .map(|res| res.map_into_left_body())
        })
    }
}

impl<S> OpenApiValidationMiddleware<S> {
    fn should_extract_body(req: &HttpRequest) -> bool {
        req.headers().contains_key("content-length")
            || req.headers().contains_key("transfer-encoding")
    }

    async fn extract_body_safely(
        mut payload: Payload,
        _req: &HttpRequest,
    ) -> Result<Option<Bytes>, Error> {
        let mut body = BytesMut::new();

        while let Some(chunk_result) = payload.next().await {
            let chunk = chunk_result.map_err(|e| {
                actix_web::error::ErrorBadRequest(format!("Error reading request chunk: {e}"))
            })?;

            body.extend_from_slice(&chunk);
        }

        if body.is_empty() {
            Ok(None)
        } else {
            Ok(Some(body.freeze()))
        }
    }
}

pub mod middleware {
    use super::OpenApiValidation;

    pub struct Validation;

    impl Validation {
        pub fn from_yaml(yaml_content: &str) -> anyhow::Result<OpenApiValidation> {
            OpenApiValidation::from_yaml(yaml_content)
        }

        pub fn from_openapi(openapi: crate::model::parse::OpenAPI) -> OpenApiValidation {
            OpenApiValidation::new(openapi)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{
        test::{self, TestRequest},
        web, App, HttpResponse, Result,
    };

    async fn dummy_handler() -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(serde_json::json!({"status": "ok"})))
    }

    #[actix_web::test]
    async fn test_middleware_with_valid_request() {
        let yaml_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /test:
    get:
      responses:
        '200':
          description: Success
"#;

        let validation = OpenApiValidation::from_yaml(yaml_content).unwrap();

        let app = test::init_service(
            App::new()
                .wrap(validation)
                .route("/test", web::get().to(dummy_handler)),
        )
        .await;

        let req = TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_middleware_with_post_request() {
        let yaml_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /test:
    post:
      requestBody:
        content:
          application/json:
            schema:
              type: object
      responses:
        '200':
          description: Success
"#;

        let validation = OpenApiValidation::from_yaml(yaml_content).unwrap();

        let app = test::init_service(
            App::new()
                .wrap(validation)
                .route("/test", web::post().to(dummy_handler)),
        )
        .await;

        let req = TestRequest::post()
            .uri("/test")
            .set_json(&serde_json::json!({"test": "value"}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[test]
    fn test_should_extract_body() {
        use actix_web::http::header;

        let req = TestRequest::post()
            .append_header((header::CONTENT_LENGTH, "100"))
            .to_http_request();

        assert!(OpenApiValidationMiddleware::<()>::should_extract_body(&req));

        let req = TestRequest::get().to_http_request();
        assert!(!OpenApiValidationMiddleware::<()>::should_extract_body(
            &req
        ));

        let req = TestRequest::post()
            .append_header((header::TRANSFER_ENCODING, "chunked"))
            .to_http_request();

        assert!(OpenApiValidationMiddleware::<()>::should_extract_body(&req));
    }
}

#[derive(Debug)]
pub struct PreExtractedBody(pub Bytes);

impl actix_web::FromRequest for PreExtractedBody {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        match req.extensions().get::<Bytes>() {
            Some(bytes) => std::future::ready(Ok(PreExtractedBody(bytes.clone()))),
            None => std::future::ready(Err(actix_web::error::ErrorBadRequest(
                "Request body not found in extensions - ensure OpenApiValidation middleware is applied"
            ))),
        }
    }
}

impl std::ops::Deref for PreExtractedBody {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
