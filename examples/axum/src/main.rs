use axum::{
    extract::{Query, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use openapi_rs::model::parse::OpenAPI;
use openapi_rs::request::axum::RequestData;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

// Application state containing OpenAPI instance
#[derive(Clone)]
struct AppState {
    openapi: Arc<OpenAPI>,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: Option<u32>,
    name: String,
    email: String,
    age: u32,
}

#[derive(Deserialize)]
struct UserQuery {
    page: u32,
    limit: u32,
}

// OpenAPI validation middleware
async fn openapi_middleware(
    State(state): State<AppState>,
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, Response> {
    // Get request path
    let path = request.uri().path().to_string();

    // Read request body (if exists)
    let (parts, body) = request.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to read request body: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid request body",
                    "message": "Failed to read request body"
                })),
            )
                .into_response());
        }
    };

    // Rebuild request
    let rebuilt_request =
        axum::http::Request::from_parts(parts.clone(), axum::body::Body::from(body_bytes.clone()));

    // Create request data for validation
    let request_data = RequestData {
        path: path.clone(),
        inner: rebuilt_request,
        body: if body_bytes.is_empty() {
            None
        } else {
            Some(body_bytes.clone())
        },
    };

    // Validate using cached OpenAPI instance
    if let Err(validation_error) = state.openapi.validator(request_data) {
        eprintln!(
            "OpenAPI validation failed - path: {}, error: {:?}",
            path, validation_error
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation failed",
                "message": format!("Request does not conform to OpenAPI specification: {}", validation_error),
                "path": path
            }))
        ).into_response());
    }

    // Rebuild request for next middleware
    let final_request = axum::http::Request::from_parts(parts, axum::body::Body::from(body_bytes));

    Ok(next.run(final_request).await)
}

// User related handlers
async fn get_users(Query(params): Query<UserQuery>) -> Json<Vec<User>> {
    let page = params.page;
    let limit = params.limit;

    // Mock user list
    let users = vec![
        User {
            id: Some(1),
            name: "John Doe".to_string(),
            email: "john.doe@example.com".to_string(),
            age: 25,
        },
        User {
            id: Some(2),
            name: "Jane Smith".to_string(),
            email: "jane.smith@example.com".to_string(),
            age: 30,
        },
    ];

    println!("Get users list - page: {}, limit: {}", page, limit);
    Json(users)
}

async fn create_user(Json(payload): Json<User>) -> Result<Json<User>, StatusCode> {
    // Mock user creation
    let mut new_user = payload;
    new_user.id = Some(3); // Mock assigned ID

    println!("Create user: {:?}", new_user);
    Ok(Json(new_user))
}

async fn health_check() -> &'static str {
    "Service is running"
}

#[tokio::main]
async fn main() {
    // Read and parse OpenAPI specification at startup
    let content = std::fs::read_to_string("api.yaml").expect("Unable to read api.yaml file");

    let openapi = OpenAPI::yaml(&content).expect("Unable to parse OpenAPI specification");

    // Create application state
    let app_state = AppState {
        openapi: Arc::new(openapi),
    };

    // Build routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/users", get(get_users).post(create_user))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            openapi_middleware,
        ))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    println!("🚀 Server started, access URL: http://127.0.0.1:8080");
    println!("📝 API endpoints:");
    println!("  - GET  /health - Health check");
    println!("  - GET  /users  - Get users list");
    println!("  - POST /users  - Create user");

    axum::serve(listener, app).await.unwrap();
}
