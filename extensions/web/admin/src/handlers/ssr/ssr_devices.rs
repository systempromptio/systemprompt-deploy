use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::cowork_grp;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ssr_helpers::render_typed_page;

#[derive(Debug, Serialize)]
struct DevicesPageData {
    pats: Vec<PatView>,
    certs: Vec<CertView>,
}

#[derive(Debug, Serialize)]
struct PatView {
    id: String,
    name: String,
    key_prefix: String,
    created_at: Option<DateTime<Utc>>,
    last_used_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    revoked: bool,
}

#[derive(Debug, Serialize)]
struct CertView {
    id: String,
    fingerprint: String,
    label: String,
    enrolled_at: DateTime<Utc>,
    revoked: bool,
}

pub async fn devices_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let pats = cowork_grp::list_api_keys_for_user(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|row| PatView {
            id: row.id,
            name: row.name,
            key_prefix: row.key_prefix,
            created_at: row.created_at,
            last_used_at: row.last_used_at,
            expires_at: row.expires_at,
            revoked: row.revoked_at.is_some(),
        })
        .collect();

    let certs = cowork_grp::list_device_certs_for_user(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|row| CertView {
            id: row.id,
            fingerprint: row.fingerprint,
            label: row.label,
            enrolled_at: row.enrolled_at,
            revoked: row.revoked_at.is_some(),
        })
        .collect();

    let data = DevicesPageData { pats, certs };
    render_typed_page(&engine, "devices", &data, &user_ctx, &mkt_ctx)
}
