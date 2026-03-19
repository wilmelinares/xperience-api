use crate::{errors::AppError, middleware::auth::AuthUser, services::storage::upload_cv};
use axum::{Extension, Json, extract::Multipart};
use uuid::Uuid;

// Response returned after a successful upload
#[derive(serde::Serialize)]
pub struct UploadResponse {
    pub url: String,
}

// POST /upload — students only
// Receives a PDF file via multipart/form-data
// Returns the public URL of the uploaded file
pub async fn upload_cv_handler(
    AuthUser(claims): AuthUser,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    // Only students upload CVs
    if claims.role != crate::models::UserRole::Student {
        return Err(AppError::Forbidden(
            "Only students can upload CVs".to_string(),
        ));
    }

    // Multipart means the request has multiple parts (fields + files)
    // We iterate through the parts looking for the file field
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart data: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        // We expect the file to be in a field called "file"
        if field_name == "file" {
            let content_type = field.content_type().unwrap_or("").to_string();

            // Validate that the uploaded file is actually a PDF
            // Never trust the client — always validate on the server
            if content_type != "application/pdf" {
                return Err(AppError::BadRequest(
                    "Only PDF files are allowed".to_string(),
                ));
            }

            // Read the file bytes
            let file_bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::InternalError(format!("Failed to read file: {}", e)))?;

            // Validate file size — max 5MB
            // 5 * 1024 * 1024 = 5,242,880 bytes
            if file_bytes.len() > 5 * 1024 * 1024 {
                return Err(AppError::BadRequest(
                    "File size must be under 5MB".to_string(),
                ));
            }

            // Generate a unique filename using UUID
            // This prevents filename collisions between different students
            // Format: cvs/{student_id}/{uuid}.pdf
            let file_name = format!("cvs/{}/{}.pdf", claims.sub, Uuid::new_v4());

            // Upload to Supabase Storage
            let url = upload_cv(file_bytes.to_vec(), &file_name).await?;

            return Ok(Json(UploadResponse { url }));
        }
    }

    // If we get here, no file field was found in the request
    Err(AppError::BadRequest(
        "No file found in request. Use field name 'file'".to_string(),
    ))
}
