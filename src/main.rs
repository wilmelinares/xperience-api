use axum::extract::DefaultBodyLimit;
use axum::{Extension, Json, Router, routing::get};
use dotenvy::dotenv;
use serde_json::json;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

  // Add SSL options for Railway's PostgreSQL
// Railway requires SSL connections in production
let pool = sqlx::postgres::PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await
    .expect("Failed to connect to PostgreSQL");

    // Run migrations automatically on startup
    // In production we don't have terminal access, so this ensures
    // the database schema is always up to date when the server starts
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database migrations applied successfully");
    tracing::info!("Connected to PostgreSQL");

    // Read PORT from environment — Railway injects this dynamically
    // Falls back to 8080 for local development
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/health", get(health_check))
        .merge(routes::auth_routes())
        .merge(routes::position_routes())
        .merge(routes::application_routes())
        .merge(routes::upload_routes())
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(Extension(pool))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Server running at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health_check(Extension(pool): Extension<PgPool>) -> Json<serde_json::Value> {
    let db_ok = sqlx::query("SELECT 1").execute(&pool).await.is_ok();
    Json(json!({
        "status": if db_ok { "ok" } else { "error" },
        "database": if db_ok { "connected" } else { "unreachable" }
    }))
}
