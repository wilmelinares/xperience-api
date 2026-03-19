use crate::{
    errors::AppError,
    middleware::auth::AuthUser,
    models::{Application, ApplyRequest, UpdateApplicationStatusRequest, UserRole},
};
use axum::{Extension, Json, extract::Path};
use sqlx::{PgPool, postgres::PgRow};
use uuid::Uuid;

// POST /applications — students only
pub async fn apply(
    AuthUser(claims): AuthUser,
    Extension(pool): Extension<PgPool>,
    Json(body): Json<ApplyRequest>,
) -> Result<Json<Application>, AppError> {
    if claims.role != UserRole::Student {
        return Err(AppError::Forbidden(
            "Only students can apply to positions".to_string(),
        ));
    }

    let student_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InternalError("Invalid user ID".to_string()))?;

    // Verify the position exists and is open
    let position_available = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM positions WHERE id = $1 AND status = 'open'",
    )
    .bind(body.position_id)
    .fetch_one(&pool)
    .await?;

    if position_available == 0 {
        return Err(AppError::BadRequest(
            "This position is not available".to_string(),
        ));
    }

    // Insert the application — UNIQUE constraint prevents duplicates
    let application: Application = sqlx::query_as(
        "INSERT INTO applications (student_id, position_id, cv_url, cover_letter)
         VALUES ($1, $2, $3, $4)
         RETURNING *",
    )
    .bind(student_id)
    .bind(body.position_id)
    .bind(&body.cv_url)
    .bind(&body.cover_letter)
    .fetch_one(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("unique") {
            AppError::Conflict("You already applied to this position".to_string())
        } else {
            AppError::from(e)
        }
    })?;

    Ok(Json(application))
}

// GET /applications — students see their own, recruiters see theirs
pub async fn list_applications(
    AuthUser(claims): AuthUser,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<Application>>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InternalError("Invalid user ID".to_string()))?;

    let applications: Vec<Application> = match claims.role {
        UserRole::Student => {
            sqlx::query_as(
                "SELECT * FROM applications
                 WHERE student_id = $1
                 ORDER BY applied_at DESC",
            )
            .bind(user_id)
            .fetch_all(&pool)
            .await?
        }

        UserRole::Recruiter => {
            sqlx::query_as(
                "SELECT a.* FROM applications a
                 JOIN positions p ON a.position_id = p.id
                 WHERE p.recruiter_id = $1
                 ORDER BY a.applied_at DESC",
            )
            .bind(user_id)
            .fetch_all(&pool)
            .await?
        }
    };

    Ok(Json(applications))
}

// PATCH /applications/:id — recruiters only
pub async fn update_status(
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    Json(body): Json<UpdateApplicationStatusRequest>,
) -> Result<Json<Application>, AppError> {
    if claims.role != UserRole::Recruiter {
        return Err(AppError::Forbidden(
            "Only recruiters can update application status".to_string(),
        ));
    }

    let recruiter_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InternalError("Invalid user ID".to_string()))?;

    // Subquery ensures recruiter can only update their own positions' applications
    let application: Option<Application> = sqlx::query_as(
        "UPDATE applications SET status = $1
         WHERE id = $2
           AND position_id IN (
               SELECT id FROM positions WHERE recruiter_id = $3
           )
         RETURNING *",
    )
    .bind(&body.status)
    .bind(id)
    .bind(recruiter_id)
    .fetch_optional(&pool)
    .await?;

    match application {
        Some(app) => Ok(Json(app)),
        None => Err(AppError::NotFound(
            "Application not found or you don't have permission".to_string(),
        )),
    }
}
