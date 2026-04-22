use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::repositories::cowork_grp;
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
pub struct IssueApiKeyRequest {
    pub name: String,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct IssueApiKeyResponse {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub secret: String,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub async fn issue_pat(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<IssueApiKeyRequest>,
) -> Response {
    match cowork_grp::issue_api_key(&pool, &user_ctx.user_id, &body.name, body.expires_at).await {
        Ok(issued) => Json(IssueApiKeyResponse {
            id: issued.id,
            name: issued.name,
            key_prefix: issued.key_prefix,
            secret: issued.secret,
            created_at: issued.created_at,
            expires_at: issued.expires_at,
        })
        .into_response(),
        Err(cowork_grp::CoworkRepoError::Validation(msg)) => {
            (StatusCode::BAD_REQUEST, msg).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to issue PAT");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to issue PAT").into_response()
        }
    }
}

pub async fn revoke_pat(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> Response {
    match cowork_grp::revoke_api_key(&pool, &user_ctx.user_id, &id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "PAT not found").into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to revoke PAT");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to revoke PAT").into_response()
        }
    }
}

pub async fn revoke_cert(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> Response {
    match cowork_grp::revoke_device_cert(&pool, &user_ctx.user_id, &id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "cert not found").into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to revoke device cert");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to revoke cert").into_response()
        }
    }
}
