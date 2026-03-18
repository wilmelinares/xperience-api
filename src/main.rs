use axum::{Extension, Json, Router, routing::get};
use dotenvy::dotenv;
use serde_json::json;
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL debe estar en .env");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("No se pudo conectar a PostgreSQL");

    tracing::info!("Conectado a PostgreSQL");

    let app = Router::new()
        .route("/health", get(health_check))
        .layer(Extension(pool));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    tracing::info!("Servidor en http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn health_check(Extension(pool): Extension<PgPool>) -> Json<serde_json::Value> {
    let db_ok = sqlx::query("SELECT 1").execute(&pool).await.is_ok();

    Json(json!({
        "status": if db_ok { "ok" } else { "error" },
        "database": if db_ok { "connected" } else { "unreachable" }
    }))
}
