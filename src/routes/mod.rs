use crate::handlers;
use axum::{
    Router,
    routing::{delete, get, patch, post},
};

pub fn auth_routes() -> Router {
    Router::new()
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
}

pub fn position_routes() -> Router {
    Router::new()
        .route(
            "/positions",
            get(handlers::positions::list_positions).post(handlers::positions::create_position),
        )
        .route(
            "/positions/:id",
            get(handlers::positions::get_position)
                .patch(handlers::positions::update_position)
                .delete(handlers::positions::delete_position),
        )
}

pub fn application_routes() -> Router {
    Router::new()
        .route(
            "/applications",
            post(handlers::applications::apply).get(handlers::applications::list_applications),
        )
        .route(
            "/applications/:id",
            patch(handlers::applications::update_status),
        )
}

pub fn upload_routes() -> Router {
    Router::new().route("/upload", post(handlers::upload::upload_cv_handler))
}
