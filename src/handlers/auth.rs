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

pub async fn login(
    Extension(pool): Extension<PgPool>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let row = sqlx::query!(
        r#"SELECT id, email, full_name,
           role as "role: UserRole",
           password_hash
           FROM users WHERE email = $1"#,
        &body.email
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    let valid = bcrypt::verify(&body.password, &row.password_hash)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let user = User {
        id: row.id,
        email: row.email,
        full_name: row.full_name,
        role: row.role,
    };

    let token = create_token(user.id, &user.email, &user.role)?;

    Ok(Json(AuthResponse { token, user }))
}
