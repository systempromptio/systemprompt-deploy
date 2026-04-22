use std::path::Path;

use sqlx::PgPool;
use systemprompt::models::Config;
use systemprompt_web_shared::error::MarketplaceError;

use super::types::SyncPluginsResponse;
use crate::repositories::export_auth::load_plugin_configs_by_ids;
use crate::repositories::export_scripts::build_marketplace;
use crate::repositories::export_validation::compute_export_totals;

pub async fn generate_org_marketplace_export_bundles(
    services_path: &Path,
    pool: &PgPool,
    marketplace_id: &str,
    _platform: &str,
) -> Result<SyncPluginsResponse, MarketplaceError> {
    let plugin_ids =
        crate::repositories::org_marketplaces::list_marketplace_plugin_ids(pool, marketplace_id)
            .await
            .map_err(|e| {
                MarketplaceError::Internal(format!("Failed to list marketplace plugins: {e}"))
            })?;

    let plugins_path = services_path.join("plugins");
    let skills_path = services_path.join("skills");
    let platform_url = Config::get().map_or_else(|_| String::new(), |c| c.api_external_url.clone());

    let all_configs =
        load_plugin_configs_by_ids(&plugins_path, &plugin_ids.iter().cloned().collect())?;
    let tokens = std::collections::HashMap::new();

    let bundles = super::build_org_bundles(
        &all_configs,
        &plugins_path,
        &skills_path,
        services_path,
        &platform_url,
        &tokens,
    )?;
    let totals = compute_export_totals(&bundles);
    let marketplace = build_marketplace(&all_configs, &bundles, marketplace_id, "")?;

    Ok(SyncPluginsResponse {
        plugins: bundles,
        marketplace,
        totals,
    })
}
