use std::path::Path;

use crate::types::{CreatePluginRequest, PluginDetail, UpdatePluginRequest};
use systemprompt::identifiers::{AgentId, McpServerId, SkillId};
use systemprompt_web_shared::error::MarketplaceError;

pub fn find_plugin_detail(
    services_path: &Path,
    plugin_id: &str,
) -> Result<Option<PluginDetail>, MarketplaceError> {
    use crate::repositories::plugin_resolvers::resolve_all_plugin_skill_ids;

    let Some(p) = super::plugin_loader::find_plugin(plugin_id)? else {
        return Ok(None);
    };
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    let skills: Vec<SkillId> = resolve_all_plugin_skill_ids(&p, &skills_path, &agents_path)
        .into_iter()
        .map(SkillId::from)
        .collect();
    Ok(Some(PluginDetail {
        id: p.id.to_string(),
        name: p.name,
        description: p.description,
        version: p.version,
        enabled: p.enabled,
        category: p.category,
        keywords: p.keywords,
        author_name: p.author.name,
        roles: Vec::new(),
        skills,
        agents: p.agents.include.into_iter().map(AgentId::from).collect(),
        mcp_servers: p
            .mcp_servers
            .into_iter()
            .filter_map(|s| McpServerId::try_new(s).ok())
            .collect(),
    }))
}

pub fn create_plugin(
    _services_path: &Path,
    _req: &CreatePluginRequest,
) -> Result<PluginDetail, MarketplaceError> {
    Err(MarketplaceError::Internal(
        "create_plugin is disabled; edit services/plugins/*.yaml directly".to_string(),
    ))
}

pub fn update_plugin(
    _services_path: &Path,
    _plugin_id: &str,
    _req: &UpdatePluginRequest,
) -> Result<Option<PluginDetail>, MarketplaceError> {
    Err(MarketplaceError::Internal(
        "update_plugin is disabled; edit services/plugins/*.yaml directly".to_string(),
    ))
}

pub fn delete_plugin(_services_path: &Path, _plugin_id: &str) -> Result<bool, MarketplaceError> {
    Err(MarketplaceError::Internal(
        "delete_plugin is disabled; edit services/plugins/*.yaml directly".to_string(),
    ))
}
