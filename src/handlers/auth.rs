use crate::{
    errors::AppError,
    middleware::auth::create_token,
    models::{AuthResponse, LoginRequest, RegisterRequest, User, UserRole},
};
use axum::{Extension, Json};
use sqlx::PgPool;



pub async fn register(
    Extension(pool): Extension<PgPool>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_one(&pool)
        .await?;
    if exists > 0 {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    let password_hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash, full_name, role)
         VALUES ($1, $2, $3, $4)
         RETURNING id, email, full_name, role",
    )
    .bind(&body.email)
    .bind(&password_hash)
    .bind(&body.full_name)
    .bind(&body.role)
    .fetch_one(&pool)
    .await?;

    // 4. Crear el JWT
    let token = create_token(user.id, &user.email, &user.role)?;

    Ok(Json(AuthResponse { token, user }))
}

// POST /auth/login
pub async fn login(
    Extension(pool): Extension<PgPool>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // We return the same error whether the email doesn't exist
    // or the password is wrong — this prevents user enumeration attacks
    // (attackers should not know which emails are registered)
    let row = sqlx::query_as::<_, (uuid::Uuid, String, String, UserRole, String)>(
        "SELECT id, email, full_name, role, password_hash
         FROM users WHERE email = $1",
    )
    .bind(&body.email)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    // row is now a tuple — we access each field by position
    let (id, email, full_name, role, password_hash) = row;

    // Verify the password against the stored hash
    let valid = bcrypt::verify(&body.password, &password_hash)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let user = User {
        id,
        email,
        full_name,
        role,
    };

    let token = create_token(user.id, &user.email, &user.role)?;

    Ok(Json(AuthResponse { token, user }))
}
