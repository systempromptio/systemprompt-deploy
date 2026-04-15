use std::collections::HashMap;
use std::path::Path;

use systemprompt::identifiers::{McpServerId, SkillId};

use crate::types::{PluginOnboardingConfig, PluginOverview, ROLE_ADMIN};
use crate::repositories::plugin_resolvers::{
    resolve_all_plugin_skill_ids, resolve_plugin_agents, resolve_plugin_skills,
};
use systemprompt_web_shared::error::MarketplaceError;

pub fn list_all_skill_ids(services_path: &Path) -> Result<Vec<String>, MarketplaceError> {
    let skills_path = services_path.join("skills");
    let mut ids = Vec::new();
    if !skills_path.exists() {
        return Ok(ids);
    }
    for entry in std::fs::read_dir(&skills_path)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            ids.push(stem.to_string());
        }
    }
    ids.sort();
    ids.dedup();
    Ok(ids)
}

pub fn list_plugin_skill_ids(
    services_path: &Path,
    plugin_id: &str,
) -> Result<Vec<String>, MarketplaceError> {
    let plugin = super::plugin_loader::find_plugin(plugin_id)?.ok_or_else(|| {
        MarketplaceError::NotFound(format!("Plugin not found: {plugin_id}"))
    })?;
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    Ok(resolve_all_plugin_skill_ids(&plugin, &skills_path, &agents_path))
}

pub fn update_plugin_skills(
    _services_path: &Path,
    _plugin_id: &str,
    _skill_ids: &[SkillId],
) -> Result<(), MarketplaceError> {
    Err(MarketplaceError::Internal(
        "update_plugin_skills is disabled; edit services/plugins/*.yaml directly".to_string(),
    ))
}

#[derive(Debug, Clone, Copy)]
pub struct MarketplaceCounts {
    pub total_plugins: usize,
    pub total_skills: usize,
    pub agents_count: usize,
    pub mcp_count: usize,
}

pub fn count_marketplace_items(
    services_path: &Path,
    roles: &[String],
) -> Result<MarketplaceCounts, MarketplaceError> {
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    let mut counts = MarketplaceCounts {
        total_plugins: 0,
        total_skills: 0,
        agents_count: 0,
        mcp_count: 0,
    };

    let is_admin = roles.iter().any(|r| r == ROLE_ADMIN);
    for (_id, plugin) in super::plugin_loader::load_all_plugins()? {
        if !plugin.base.enabled {
            continue;
        }
        if !is_admin && !plugin.roles.is_empty() && !plugin.roles.iter().any(|r| roles.contains(r))
        {
            continue;
        }
        counts.total_plugins += 1;
        counts.total_skills +=
            resolve_all_plugin_skill_ids(&plugin.base, &skills_path, &agents_path).len();
        counts.agents_count += plugin.base.agents.include.len();
        counts.mcp_count += plugin.base.mcp_servers.len();
    }

    Ok(counts)
}

pub fn list_plugins_for_roles(
    services_path: &Path,
    roles: &[String],
) -> Result<Vec<PluginOverview>, MarketplaceError> {
    list_plugins_for_roles_full(services_path, roles)
}

pub fn list_plugins_for_roles_full(
    services_path: &Path,
    roles: &[String],
) -> Result<Vec<PluginOverview>, MarketplaceError> {
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    let is_admin = roles.iter().any(|r| r == ROLE_ADMIN);
    let mut overviews = Vec::new();
    for (_id, plugin) in super::plugin_loader::load_all_plugins()? {
        if !plugin.base.enabled && !is_admin {
            continue;
        }
        if !is_admin && !plugin.roles.is_empty() && !plugin.roles.iter().any(|r| roles.contains(r))
        {
            continue;
        }
        let skill_infos = resolve_plugin_skills(&plugin.base, &skills_path, &agents_path);
        let agent_infos = resolve_plugin_agents(&plugin.base, &agents_path);
        overviews.push(PluginOverview {
            id: plugin.base.id.to_string(),
            name: plugin.base.name,
            description: plugin.base.description,
            enabled: plugin.base.enabled,
            skills: skill_infos,
            agents: agent_infos,
            mcp_servers: plugin
                .base
                .mcp_servers
                .into_iter()
                .filter_map(|s| McpServerId::try_new(s).ok())
                .collect(),
            hooks: vec![],
            depends: plugin.depends,
        });
    }
    Ok(overviews)
}

#[must_use]
pub fn load_plugin_onboarding_configs() -> HashMap<String, PluginOnboardingConfig> {
    HashMap::new()
}
