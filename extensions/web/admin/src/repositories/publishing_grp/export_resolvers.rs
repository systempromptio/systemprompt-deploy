use std::collections::HashSet;
use std::path::Path;

use crate::repositories::plugin_resolvers::collect_agent_skills;
use systemprompt_web_shared::error::MarketplaceError;

pub(super) fn resolve_export_skills(
    plugin: &systemprompt::models::PluginConfig,
    skills_path: &Path,
    agents_path: &Path,
) -> Result<Vec<(String, std::path::PathBuf)>, MarketplaceError> {
    let mut resolved = Vec::new();

    if plugin.skills.source == systemprompt::models::ComponentSource::Explicit {
        for skill_id in &plugin.skills.include {
            let yaml_path = skills_path.join(format!("{skill_id}.yaml"));
            if yaml_path.exists() {
                resolved.push((skill_id.clone(), yaml_path));
            }
        }
    } else if skills_path.exists() {
        for entry in std::fs::read_dir(skills_path)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                continue;
            }
            let Some(skill_id) = path.file_stem().and_then(|s| s.to_str()).map(String::from) else {
                continue;
            };
            if plugin.skills.exclude.contains(&skill_id) {
                continue;
            }
            resolved.push((skill_id, path));
        }
    }

    let existing: HashSet<String> = resolved.iter().map(|(id, _)| id.clone()).collect();
    for agent_skill in collect_agent_skills(&plugin.agents.include, agents_path) {
        if !existing.contains(&agent_skill) {
            let yaml_path = skills_path.join(format!("{agent_skill}.yaml"));
            if yaml_path.exists() {
                resolved.push((agent_skill, yaml_path));
            }
        }
    }

    resolved.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(resolved)
}

pub(super) fn build_skill_md(
    skill_id: &str,
    skill_yaml_path: &Path,
    skill_hooks_yaml: Option<&str>,
) -> Result<String, MarketplaceError> {
    let description = if skill_yaml_path.exists() {
        let cfg_text = std::fs::read_to_string(skill_yaml_path)?;
        let cfg: serde_yaml::Value = serde_yaml::from_str(&cfg_text)?;
        cfg.get("skills")
            .and_then(|s| s.get("skills"))
            .and_then(|m| m.get(skill_id))
            .and_then(|e| e.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };

    let md_path = skill_yaml_path.with_extension("md");
    let body = if md_path.exists() {
        strip_frontmatter(&std::fs::read_to_string(&md_path)?)
    } else {
        format!(
            "$(systemprompt core skills show {skill_id} --raw 2>/dev/null || echo \"Skill not available\")",
        )
    };

    let hooks_section = skill_hooks_yaml.map_or_else(String::new, |h| format!("{h}\n"));

    let kebab_name = skill_id.replace('_', "-");
    Ok(format!(
        "---\nname: {}\ndescription: \"{}\"\n{}---\n\n{}\n",
        kebab_name,
        description.replace('"', "\\\""),
        hooks_section,
        body.trim()
    ))
}

fn strip_frontmatter(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return content.to_string();
    }
    let parts: Vec<&str> = trimmed.splitn(3, "---").collect();
    if parts.len() >= 3 {
        parts[2].trim().to_string()
    } else {
        content.to_string()
    }
}

pub(super) const fn collect_skill_auxiliary_files(
    _skill_id: &str,
    _skill_yaml_path: &Path,
) -> Vec<(String, String, bool)> {
    Vec::new()
}

pub(super) fn resolve_export_agents(
    plugin: &systemprompt::models::PluginConfig,
    services_path: &Path,
) -> Result<Vec<String>, MarketplaceError> {
    if plugin.agents.source == systemprompt::models::ComponentSource::Explicit {
        return Ok(plugin.agents.include.clone());
    }
    let agents_dir = services_path.join("agents");
    if !agents_dir.exists() {
        return Ok(Vec::new());
    }
    let mut ids = Vec::new();
    for entry in std::fs::read_dir(&agents_dir)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let content = std::fs::read_to_string(&path)?;
        let config: serde_yaml::Value = match serde_yaml::from_str(&content) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if let Some(agents) = config.get("agents").and_then(|a| a.as_mapping()) {
            for (key, _) in agents {
                if let Some(name) = key.as_str() {
                    if !plugin.agents.exclude.contains(&name.to_string()) {
                        ids.push(name.to_string());
                    }
                }
            }
        }
    }
    ids.sort();
    Ok(ids)
}

pub(super) fn build_agent_md(
    agent_id: &str,
    agents_dir: &Path,
) -> Result<String, MarketplaceError> {
    if !agents_dir.exists() {
        return Ok(format!(
            "---\nname: {agent_id}\ndescription: \"{agent_id} agent\"\n---\n\nYou are the {agent_id} agent.\n",
        ));
    }

    let mut description = format!("{agent_id} agent");
    let mut system_prompt = String::new();

    for entry in std::fs::read_dir(agents_dir)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let content = std::fs::read_to_string(&path)?;
        let config: serde_yaml::Value = match serde_yaml::from_str(&content) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if let Some(agent_val) = config.get("agents").and_then(|a| a.get(agent_id)) {
            if let Some(desc) = agent_val
                .get("card")
                .and_then(|c| c.get("description"))
                .and_then(|d| d.as_str())
            {
                description = desc.to_string();
            }
            if let Some(prompt) = agent_val
                .get("metadata")
                .and_then(|m| m.get("systemPrompt"))
                .and_then(|s| s.as_str())
            {
                system_prompt = prompt.to_string();
            }
            break;
        }
    }

    let escaped_desc = description.replace('"', "\\\"");
    let trimmed_prompt = system_prompt.trim();
    Ok(format!(
        "---\nname: {agent_id}\ndescription: \"{escaped_desc}\"\n---\n\n{trimmed_prompt}\n",
    ))
}
