use crate::errors::AppError;

// Uploads a file to Supabase Storage and returns the public URL
// file_bytes: the raw bytes of the PDF file
// file_name: a unique name we generate for the file
pub async fn upload_cv(file_bytes: Vec<u8>, file_name: &str) -> Result<String, AppError> {
    // Read credentials from environment variables
    let supabase_url = std::env::var("SUPABASE_URL")
        .map_err(|_| AppError::InternalError("SUPABASE_URL not set".to_string()))?;

    let service_key = std::env::var("SUPABASE_SERVICE_KEY")
        .map_err(|_| AppError::InternalError("SUPABASE_SERVICE_KEY not set".to_string()))?;

    let bucket = std::env::var("SUPABASE_BUCKET")
        .map_err(|_| AppError::InternalError("SUPABASE_BUCKET not set".to_string()))?;

    // Build the upload URL
    // Supabase Storage API endpoint: /storage/v1/object/{bucket}/{filename}
    let upload_url = format!(
        "{}/storage/v1/object/{}/{}",
        supabase_url, bucket, file_name
    );

    // Create an HTTP client
    let client = reqwest::Client::new();

    // Send the file to Supabase
    // We set Content-Type to application/pdf so Supabase knows what kind of file it is
    let response = client
        .post(&upload_url)
        .header("Authorization", format!("Bearer {}", service_key))
        .header("Content-Type", "application/pdf")
        .body(file_bytes)
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to upload file: {}", e)))?;

    // Check if Supabase accepted the file
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::InternalError(format!(
            "Supabase rejected the upload: {}",
            error_text
        )));
    }

    // Build the public URL of the uploaded file
    // This is the URL that will be stored in the database and shown to recruiters
    let public_url = format!(
        "{}/storage/v1/object/public/{}/{}",
        supabase_url, bucket, file_name
    );

    Ok(public_url)
}
