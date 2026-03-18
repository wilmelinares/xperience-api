use crate::handlers;
use axum::{Router, routing::post};

pub fn auth_routes() -> Router {
    Router::new()
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
}
