use std::path::Path;

use crate::types::PlatformPluginConfig;
use crate::types::ROLE_ADMIN;
use systemprompt_web_shared::error::MarketplaceError;

pub fn _load_authorized_plugin_configs(
    _plugins_path: &Path,
    roles: &[String],
) -> Result<Vec<(String, PlatformPluginConfig)>, MarketplaceError> {
    let all_plugins = crate::repositories::plugins_grp::plugin_loader::load_all_plugins()?;

    let is_admin = roles.iter().any(|r| r == ROLE_ADMIN);
    let mut authorized: Vec<(String, PlatformPluginConfig)> = all_plugins
        .iter()
        .filter(|(_, plugin)| {
            if !plugin.base.enabled {
                return false;
            }
            if is_admin || plugin.roles.is_empty() {
                return true;
            }
            plugin.roles.iter().any(|r| roles.contains(r))
        })
        .cloned()
        .collect();

    let mut seen_ids: std::collections::HashSet<String> = authorized
        .iter()
        .map(|(_, p)| p.base.id.to_string())
        .collect();
    let mut i = 0;
    while i < authorized.len() {
        let deps = authorized[i].1.depends.clone();
        for dep_id in &deps {
            if seen_ids.contains(dep_id) {
                continue;
            }
            if let Some(dep) = all_plugins
                .iter()
                .find(|(_, p)| p.base.id.as_str() == dep_id)
            {
                if !dep.1.base.enabled {
                    return Err(MarketplaceError::Internal(format!(
                        "Plugin '{}' depends on '{}' which is disabled",
                        authorized[i].1.base.id, dep_id
                    )));
                }
                seen_ids.insert(dep_id.clone());
                authorized.push(dep.clone());
            } else {
                return Err(MarketplaceError::Internal(format!(
                    "Plugin '{}' depends on '{}' which was not found",
                    authorized[i].1.base.id, dep_id
                )));
            }
        }
        i += 1;
    }

    authorized.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(authorized)
}

pub fn load_plugin_configs_by_ids(
    _plugins_path: &Path,
    authorized_ids: &std::collections::HashSet<String>,
) -> Result<Vec<(String, PlatformPluginConfig)>, MarketplaceError> {
    let all_plugins = crate::repositories::plugins_grp::plugin_loader::load_all_plugins()?;

    let mut authorized: Vec<(String, PlatformPluginConfig)> = all_plugins
        .iter()
        .filter(|(dir_name, plugin)| plugin.base.enabled && authorized_ids.contains(dir_name))
        .cloned()
        .collect();

    let mut seen_ids: std::collections::HashSet<String> = authorized
        .iter()
        .map(|(_, p)| p.base.id.to_string())
        .collect();
    let mut i = 0;
    while i < authorized.len() {
        let deps = authorized[i].1.depends.clone();
        for dep_id in &deps {
            if seen_ids.contains(dep_id) {
                continue;
            }
            if let Some(dep) = all_plugins
                .iter()
                .find(|(_, p)| p.base.id.as_str() == dep_id)
            {
                if !dep.1.base.enabled {
                    return Err(MarketplaceError::Internal(format!(
                        "Plugin '{}' depends on '{}' which is disabled",
                        authorized[i].1.base.id, dep_id
                    )));
                }
                seen_ids.insert(dep_id.clone());
                authorized.push(dep.clone());
            } else {
                return Err(MarketplaceError::Internal(format!(
                    "Plugin '{}' depends on '{}' which was not found",
                    authorized[i].1.base.id, dep_id
                )));
            }
        }
        i += 1;
    }

    authorized.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(authorized)
}
