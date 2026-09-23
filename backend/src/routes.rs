use std::sync::{Arc, Mutex};

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use rusqlite::Connection;
use serde::Deserialize;

use crate::db;
use crate::error::ApiError;
use crate::models::{Variable, format_dotenv};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
}

#[derive(Deserialize)]
pub struct ScopeQuery {
    scope: Option<String>,
}

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

pub async fn list_variables(
    State(state): State<AppState>,
    Query(query): Query<ScopeQuery>,
) -> Result<Json<Vec<Variable>>, ApiError> {
    let scope = normalize_scope_filter(query.scope)?;
    let conn = lock_db(&state)?;
    let variables = db::list(&conn, scope.as_deref())?;
    Ok(Json(variables))
}

pub async fn create_variable(
    State(state): State<AppState>,
    Json(body): Json<crate::models::UpsertVariable>,
) -> Result<impl IntoResponse, ApiError> {
    let input = body.normalize().map_err(ApiError::BadRequest)?;
    let conn = lock_db(&state)?;
    let created = db::insert(&conn, &input)?;
    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn update_variable(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<crate::models::UpsertVariable>,
) -> Result<Json<Variable>, ApiError> {
    let input = body.normalize().map_err(ApiError::BadRequest)?;
    let conn = lock_db(&state)?;
    Ok(Json(db::update(&conn, id, &input)?))
}

pub async fn delete_variable(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let conn = lock_db(&state)?;
    db::delete(&conn, id)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn export_variables(
    State(state): State<AppState>,
    Query(query): Query<ScopeQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let scope = normalize_scope_filter(query.scope)?;
    let conn = lock_db(&state)?;
    let variables = db::list(&conn, scope.as_deref())?;
    Ok((
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; charset=utf-8",
        )],
        format_dotenv(&variables),
    ))
}

fn normalize_scope_filter(scope: Option<String>) -> Result<Option<String>, ApiError> {
    match scope {
        Some(scope) => {
            let scope = scope.trim().to_string();
            if scope.is_empty() {
                Ok(None)
            } else {
                crate::models::validate_scope(&scope).map_err(ApiError::BadRequest)?;
                Ok(Some(scope))
            }
        }
        None => Ok(None),
    }
}

fn lock_db(state: &AppState) -> Result<std::sync::MutexGuard<'_, Connection>, ApiError> {
    state
        .db
        .lock()
        .map_err(|_| ApiError::Internal("数据库锁已损坏"))
}
