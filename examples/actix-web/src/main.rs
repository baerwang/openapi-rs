use actix_web::{get, post};
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use openapi_rs::observability::init_logger;
use openapi_rs::request::actix_web::OpenApiValidation;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    path: Option<String>,
}

// User related handlers
#[get("/users")]
async fn get(query: web::Query<UserQuery>) -> Result<HttpResponse> {
    let page = query.page;
    let limit = query.limit;
    // Mock user list with pagination
    let all_users = vec![
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
    Ok(HttpResponse::Ok().json(all_users))
}

#[post("/users")]
async fn create(user: web::Json<User>) -> Result<HttpResponse> {
    // Additional business logic validation if needed
    if user.name.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Business validation failed".to_string(),
            message: "Name cannot be empty".to_string(),
            path: Some("/users".to_string()),
        }));
    }

    // Mock user creation
    let mut new_user = user.into_inner();
    new_user.id = Some(rand::random::<u32>() % 1000 + 1000);

    println!("Create user: {:?}", new_user);
    Ok(HttpResponse::Ok().json(new_user))
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "openapi-rs-actix-web-example"
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let content = std::fs::read_to_string("api.yaml")?;
    let validation = OpenApiValidation::from_yaml(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    println!("🚀 Server started, access URL: http://127.0.0.1:8080");
    println!("📝 API endpoints:");
    println!("  - GET  /health           - Health check (no validation)");
    println!("  - GET  /users?page=1&limit=10 - Get users list (with OpenAPI validation)");
    println!("  - POST /users            - Create user (with OpenAPI validation)");

    HttpServer::new(move || {
        App::new()
            .wrap(validation.clone())
            .service(get)
            .service(create)
            .route("/health", web::get().to(health_check))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
