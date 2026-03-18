use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Student,
    Recruiter,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "position_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]

pub enum PositionStatus {
    Draft,
    Open,
    Closed,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[sqlx(type_name = "application_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ApplicationStatus {
    Pending,
    Reviewing,
    Accepted,
    Rejected,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role: UserRole,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub role: UserRole,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

// ── POSITION ───────────────────────────────────────────────────

// What comes from the database when we query a position
// FromRow automatically converts the PostgreSQL row into this struct

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Position {
    pub id: Uuid,
    pub recruiter_id: Uuid,
    pub title: String,
    pub description: String,
    pub location: String,
    pub is_remote: bool,
    pub has_salary: bool,
    pub salary_amount: Option<Decimal>,
    pub salary_currency: String,
    pub status: PositionStatus,
}

// Data sent by the recruiter to create a position
#[derive(Debug, Deserialize)]
pub struct CreatePositionRequest {
    pub title: String,
    pub description: String,
    pub location: String,
    pub is_remote: bool,
    pub has_salary: bool,
    pub salary_amount: Option<Decimal>,
    pub salary_currency: Option<String>,
}

// Data sent by the recruiter to update a position
// All fields are optional — we only update the ones provided
// If only {"status": "open"} is sent, only the status is updated
#[derive(Debug, Deserialize)]
pub struct UpdatePositionRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<PositionStatus>,
    pub has_salary: Option<bool>,
    pub salary_amount: Option<Decimal>,
}
