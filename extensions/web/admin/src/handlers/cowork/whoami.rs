use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::handlers::shared;

struct CoworkUserRow {
    id: String,
    name: String,
    email: String,
    display_name: Option<String>,
    roles: Vec<String>,
}

async fn fetch_user(pool: &PgPool, user_id: &UserId) -> Result<Option<CoworkUserRow>, sqlx::Error> {
    sqlx::query!(
        r#"SELECT id, name, email, display_name,
                  COALESCE(roles, '{}') as "roles!: Vec<String>"
           FROM users WHERE id = $1"#,
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await
    .map(|opt| {
        opt.map(|r| CoworkUserRow {
            id: r.id,
            name: r.name,
            email: r.email,
            display_name: r.display_name,
            roles: r.roles,
        })
    })
}

pub(super) async fn load_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<CoworkUserRow>, sqlx::Error> {
    fetch_user(pool, user_id).await
}

pub(super) fn user_to_json(u: &CoworkUserRow) -> serde_json::Value {
    json!({
        "id": u.id,
        "name": u.name,
        "email": u.email,
        "display_name": u.display_name,
        "roles": u.roles,
    })
}

pub async fn handle(State(pool): State<Arc<PgPool>>, headers: HeaderMap) -> Response {
    let user_id = match super::validate_cowork_jwt(&headers) {
        Ok(id) => id,
        Err(r) => return *r,
    };

    let user = match fetch_user(&pool, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return shared::error_response(StatusCode::NOT_FOUND, "User not found");
        },
        Err(e) => {
            tracing::error!(error = %e, "user lookup failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "User lookup failed",
            );
        },
    };

    Json(json!({
        "user": user_to_json(&user),
        "capabilities": ["plugins", "skills", "agents", "mcp", "user"],
    }))
    .into_response()
}
