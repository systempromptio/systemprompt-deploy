use systemprompt::loader::ConfigLoader;
use systemprompt::models::{PluginConfig, ServicesConfig};

use crate::types::PlatformPluginConfig;
use systemprompt_web_shared::error::MarketplaceError;

pub fn load_services_config() -> Result<ServicesConfig, MarketplaceError> {
    ConfigLoader::load().map_err(|e| {
        tracing::error!(error = %e, "Failed to load services config");
        MarketplaceError::Internal(format!("Failed to load services config: {e}"))
    })
}

pub fn load_all_plugins() -> Result<Vec<(String, PlatformPluginConfig)>, MarketplaceError> {
    let services = load_services_config()?;
    let mut out: Vec<(String, PlatformPluginConfig)> = services
        .plugins
        .into_iter()
        .map(|(id, cfg)| (id, PlatformPluginConfig::from_base(cfg)))
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

pub fn find_plugin(plugin_id: &str) -> Result<Option<PluginConfig>, MarketplaceError> {
    Ok(load_services_config()?.plugins.remove(plugin_id))
}
