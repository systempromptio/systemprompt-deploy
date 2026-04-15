use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::repositories;
use crate::types::access_control::{
    AccessControlQuery, BulkAssignRequest, UpdateEntityRulesRequest,
};

pub async fn list_access_rules_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AccessControlQuery>,
) -> Response {
    let result = if let (Some(ref et), Some(ref eid)) = (&query.entity_type, &query.entity_id) {
        repositories::access_control::list_rules_for_entity(&pool, et, eid).await
    } else {
        repositories::access_control::list_all_rules(&pool).await
    };

    match result {
        Ok(rules) => Json(serde_json::json!({ "rules": rules })).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list access control rules");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
                .into_response()
        }
    }
}

pub async fn update_entity_rules_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<UpdateEntityRulesRequest>,
) -> Response {
    if !["plugin", "agent", "mcp_server", "marketplace"].contains(&entity_type.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid entity_type. Must be plugin, agent, or mcp_server."})),
        )
            .into_response();
    }

    let result = repositories::access_control::set_entity_rules(
        &pool,
        &entity_type,
        &entity_id,
        &body.rules,
    )
    .await;

    match result {
        Ok(rules) => Json(serde_json::json!({ "rules": rules })).into_response(),
        Err(e) => {
            tracing::error!(error = %e, entity_type, entity_id, "Failed to update access control rules");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
                .into_response()
        }
    }
}

pub async fn bulk_assign_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<BulkAssignRequest>,
) -> Response {
    let entities: Vec<(String, String)> = body
        .entities
        .iter()
        .map(|e| (e.entity_type.clone(), e.entity_id.clone()))
        .collect();

    match repositories::access_control::bulk_set_rules(&pool, &entities, &body.rules).await {
        Ok(count) => Json(serde_json::json!({
            "updated_count": count,
            "rules_per_entity": body.rules.len(),
        }))
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to bulk assign access control rules");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
                .into_response()
        }
    }
}

pub async fn access_control_departments_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match repositories::fetch_department_stats(&pool).await {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to fetch department stats");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
                .into_response()
        }
    }
}
