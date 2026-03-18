use crate::{
    errors::AppError,
    middleware::auth::AuthUser,
    models::{CreatePositionRequest, Position, UpdatePositionRequest, UserRole},
};
use axum::{Extension, Json, extract::Path};
use sqlx::PgPool;
use uuid::Uuid;

// GET /positions — public, no token required
// Returns only open positions ordered by newest first
pub async fn list_positions(
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<Position>>, AppError> {
    let positions = sqlx::query_as::<_, Position>(
        "SELECT * FROM positions WHERE status = 'open' ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(positions))
}

// GET /positions/:id — public
// Path(id) extracts :id from the URL and converts it to Uuid automatically
pub async fn get_position(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Position>, AppError> {
    let position = sqlx::query_as::<_, Position>("SELECT * FROM positions WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Position not found".to_string()))?;

    Ok(Json(position))
}

// POST /positions — recruiters only
// AuthUser validates the JWT automatically before this handler runs
pub async fn create_position(
    AuthUser(claims): AuthUser,
    Extension(pool): Extension<PgPool>,
    Json(body): Json<CreatePositionRequest>,
) -> Result<Json<Position>, AppError> {
    if claims.role != UserRole::Recruiter {
        return Err(AppError::Forbidden(
            "Only recruiters can create positions".to_string(),
        ));
    }

    let recruiter_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InternalError("Invalid user ID".to_string()))?;

    // RETURNING * sends back the full inserted row
    // This avoids doing a separate SELECT after the INSERT
    let position = sqlx::query_as::<_, Position>(
        "INSERT INTO positions
            (recruiter_id, title, description, location,
             is_remote, has_salary, salary_amount, salary_currency)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING *",
    )
    .bind(recruiter_id)
    .bind(&body.title)
    .bind(&body.description)
    .bind(&body.location)
    .bind(body.is_remote)
    .bind(body.has_salary)
    .bind(body.salary_amount)
    .bind(body.salary_currency.unwrap_or("DOP".to_string()))
    .fetch_one(&pool)
    .await?;

    Ok(Json(position))
}

// PATCH /positions/:id — only the recruiter who owns the position
// COALESCE keeps the current value if the new value is NULL
pub async fn update_position(
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    Json(body): Json<UpdatePositionRequest>,
) -> Result<Json<Position>, AppError> {
    if claims.role != UserRole::Recruiter {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let recruiter_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InternalError("Invalid user ID".to_string()))?;

    // We check recruiter_id in the WHERE clause so a recruiter
    // cannot edit another recruiter's position
    let position = sqlx::query_as::<_, Position>(
        "UPDATE positions SET
            title         = COALESCE($1, title),
            description   = COALESCE($2, description),
            status        = COALESCE($3, status),
            has_salary    = COALESCE($4, has_salary),
            salary_amount = COALESCE($5, salary_amount)
         WHERE id = $6 AND recruiter_id = $7
         RETURNING *",
    )
    .bind(body.title)
    .bind(body.description)
    .bind(body.status)
    .bind(body.has_salary)
    .bind(body.salary_amount)
    .bind(id)
    .bind(recruiter_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Position not found or you don't own it".to_string()))?;

    Ok(Json(position))
}

// DELETE /positions/:id — only the recruiter who owns the position
pub async fn delete_position(
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != UserRole::Recruiter {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let recruiter_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InternalError("Invalid user ID".to_string()))?;

    let result = sqlx::query("DELETE FROM positions WHERE id = $1 AND recruiter_id = $2")
        .bind(id)
        .bind(recruiter_id)
        .execute(&pool)
        .await?;

    // rows_affected() returns how many rows were deleted
    // If 0, the position either doesn't exist or belongs to another recruiter
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Position not found or you don't own it".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({ "message": "Position deleted" })))
}
