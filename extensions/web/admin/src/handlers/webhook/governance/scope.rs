use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use systemprompt::models::ProfileBootstrap;

static SCOPE_CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();

pub(super) fn resolve_agent_scope(agent_id: &str) -> String {
    let map = SCOPE_CACHE.get_or_init(load_all_agent_scopes);
    map.get(agent_id)
        .cloned()
        .unwrap_or_else(|| "unknown".to_string())
}

fn load_all_agent_scopes() -> HashMap<String, String> {
    let mut scopes = HashMap::new();

    let Ok(services_path) = ProfileBootstrap::get().map(|p| PathBuf::from(&p.paths.services))
    else {
        return scopes;
    };

    let agents_dir = services_path.join("agents");
    let Ok(entries) = std::fs::read_dir(&agents_dir) else {
        return scopes;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let config_path = path.join("config.yaml");
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            continue;
        };
        let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
            continue;
        };
        if let Some(scope) = extract_scope_for_agent(&value) {
            scopes.insert(dir_name.to_string(), scope);
        }
    }

    scopes
}

fn extract_scope_for_agent(agent_val: &serde_yaml::Value) -> Option<String> {
    if let Some(scope) = agent_val
        .get("oauth")
        .and_then(|o| o.get("scopes"))
        .and_then(|s| s.as_sequence())
        .and_then(|seq| seq.first())
        .and_then(|s| s.as_str())
    {
        return Some(scope.to_string());
    }

    let security = agent_val
        .get("card")
        .and_then(|c| c.get("security"))
        .and_then(|s| s.as_sequence())?;

    for sec in security {
        if let Some(scope) = sec
            .get("oauth2")
            .and_then(|o| o.as_sequence())
            .and_then(|seq| seq.first())
            .and_then(|s| s.as_str())
        {
            return Some(scope.to_string());
        }
    }

    None
}
