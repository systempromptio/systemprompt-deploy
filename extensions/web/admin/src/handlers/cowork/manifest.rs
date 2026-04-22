use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::models::AppPaths;
use systemprompt::security::manifest_signing;

use crate::handlers::shared;
use crate::repositories::user_agents::list_user_agents;
use crate::repositories::user_skills::list_user_skills;
use crate::repositories::users_grp::user_mcp_servers::list_user_mcp_servers;
use crate::repositories::users_grp::user_plugins::list_user_plugins;

#[derive(Serialize)]
struct PluginFileEntry {
    path: String,
    sha256: String,
    size: u64,
}

#[derive(Serialize)]
struct PluginEntry {
    id: String,
    version: String,
    sha256: String,
    files: Vec<PluginFileEntry>,
}

#[derive(Serialize)]
struct SkillEntry {
    id: String,
    name: String,
    description: String,
    file_path: String,
    tags: Vec<String>,
    sha256: String,
    instructions: String,
}

#[derive(Serialize)]
struct AgentEntry {
    id: String,
    name: String,
    display_name: String,
    description: String,
    version: String,
    endpoint: String,
    enabled: bool,
    is_default: bool,
    is_primary: bool,
    provider: Option<String>,
    model: Option<String>,
    mcp_servers: Vec<String>,
    skills: Vec<String>,
    tags: Vec<String>,
    card: serde_json::Value,
}

#[derive(Serialize)]
struct ManagedMcpServer {
    name: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    transport: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    headers: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_policy: Option<BTreeMap<String, String>>,
}

pub async fn handle(State(pool): State<Arc<PgPool>>, headers: HeaderMap) -> Response {
    let user_id = match super::validate_cowork_jwt(&headers) {
        Ok(id) => id,
        Err(r) => return *r,
    };

    let user = match super::whoami::load_user(&pool, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return shared::error_response(StatusCode::NOT_FOUND, "User not found"),
        Err(e) => {
            tracing::error!(error = %e, "user lookup failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "User lookup failed",
            );
        },
    };

    let user_plugins = match list_user_plugins(&pool, &user_id).await {
        Ok(rs) => rs,
        Err(e) => {
            tracing::error!(error = %e, "list_user_plugins failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Plugin listing failed",
            );
        },
    };

    let user_skills_rows = match list_user_skills(&pool, &user_id).await {
        Ok(rs) => rs,
        Err(e) => {
            tracing::error!(error = %e, "list_user_skills failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Skill listing failed",
            );
        },
    };

    let user_agent_rows = match list_user_agents(&pool, &user_id).await {
        Ok(rs) => rs,
        Err(e) => {
            tracing::error!(error = %e, "list_user_agents failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Agent listing failed",
            );
        },
    };

    let user_mcp_rows = match list_user_mcp_servers(&pool, &user_id).await {
        Ok(rs) => rs,
        Err(e) => {
            tracing::error!(error = %e, "list_user_mcp_servers failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "MCP listing failed",
            );
        },
    };

    let plugins_root = match AppPaths::get() {
        Ok(p) => p.system().services().join("plugins"),
        Err(e) => {
            tracing::error!(error = %e, "AppPaths::get failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Service paths unavailable",
            );
        },
    };

    let plugins: Vec<PluginEntry> = user_plugins
        .iter()
        .filter(|p| p.enabled)
        .filter_map(|p| build_plugin_entry(&plugins_root, &p.plugin_id, &p.version))
        .collect();

    let skills: Vec<SkillEntry> = user_skills_rows
        .into_iter()
        .filter(|s| s.enabled)
        .map(|s| {
            let mut h = Sha256::new();
            h.update(s.skill_id.as_str().as_bytes());
            h.update(b"\0");
            h.update(s.content.as_bytes());
            let sha256 = hex_encode(&h.finalize());
            SkillEntry {
                id: s.skill_id.as_str().to_string(),
                name: s.name,
                description: s.description,
                file_path: String::new(),
                tags: s.tags,
                sha256,
                instructions: s.content,
            }
        })
        .collect();

    let agents: Vec<AgentEntry> = user_agent_rows
        .into_iter()
        .filter(|a| a.enabled)
        .map(|a| AgentEntry {
            id: a.agent_id.as_str().to_string(),
            name: a.name.clone(),
            display_name: a.name,
            description: a.description,
            version: "1.0.0".into(),
            endpoint: format!("/api/v1/agents/{}", a.agent_id.as_str()),
            enabled: a.enabled,
            is_default: false,
            is_primary: false,
            provider: None,
            model: None,
            mcp_servers: Vec::new(),
            skills: Vec::new(),
            tags: Vec::new(),
            card: json!({ "system_prompt": a.system_prompt }),
        })
        .collect();

    let managed_mcp_servers: Vec<ManagedMcpServer> = user_mcp_rows
        .into_iter()
        .filter(|m| m.enabled)
        .map(|m| ManagedMcpServer {
            name: m.name,
            url: m.endpoint,
            transport: Some("http".into()),
            headers: None,
            oauth: Some(m.oauth_required),
            tool_policy: None,
        })
        .collect();

    let manifest_version = format!(
        "{}-{}",
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
        short_hash(&plugins, &managed_mcp_servers, &skills, &agents),
    );
    let issued_at = chrono::Utc::now().to_rfc3339();
    let user_section = super::whoami::user_to_json(&user);

    let payload = json!({
        "manifest_version": manifest_version,
        "issued_at": issued_at,
        "user_id": user_id.as_str(),
        "tenant_id": serde_json::Value::Null,
        "user": user_section,
        "plugins": plugins,
        "skills": skills,
        "agents": agents,
        "managed_mcp_servers": managed_mcp_servers,
        "revocations": Vec::<String>::new(),
    });

    let canonical = match serde_json::to_string(&payload) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "canonical serialise failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Manifest serialisation failed",
            );
        },
    };

    let signature = match manifest_signing::sign_payload(canonical.as_bytes()) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "manifest signing failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Manifest signing failed",
            );
        },
    };

    let mut response = payload;
    if let Some(obj) = response.as_object_mut() {
        obj.insert("signature".into(), json!(signature));
    }

    (StatusCode::OK, Json(response)).into_response()
}

fn build_plugin_entry(plugins_root: &Path, plugin_id: &str, fallback_version: &str) -> Option<PluginEntry> {
    if !safe_id(plugin_id) {
        return None;
    }
    let plugin_dir = plugins_root.join(plugin_id);
    if !plugin_dir.is_dir() {
        return None;
    }
    let files = collect_plugin_files(&plugin_dir).ok()?;
    let dir_hash = directory_hash_from_files(&files);
    let version = read_plugin_version(&plugin_dir).unwrap_or_else(|| fallback_version.to_string());
    Some(PluginEntry {
        id: plugin_id.to_string(),
        version,
        sha256: dir_hash,
        files,
    })
}

fn safe_id(id: &str) -> bool {
    !id.is_empty()
        && !id.contains("..")
        && !id.contains('/')
        && !id.contains('\\')
        && !id.starts_with('.')
}

const BLOCKED: &[&str] = &["config.yaml", "config.yml"];

fn collect_plugin_files(root: &Path) -> Result<Vec<PluginFileEntry>, std::io::Error> {
    let mut out = Vec::new();
    walk(root, root, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn walk(base: &Path, dir: &Path, out: &mut Vec<PluginFileEntry>) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        let path = entry.path();
        if ft.is_dir() {
            walk(base, &path, out)?;
        } else if ft.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if BLOCKED.contains(&name) {
                    continue;
                }
            }
            let bytes = std::fs::read(&path)?;
            let mut h = Sha256::new();
            h.update(&bytes);
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push(PluginFileEntry {
                path: rel,
                sha256: hex_encode(&h.finalize()),
                size: bytes.len() as u64,
            });
        }
    }
    Ok(())
}

fn directory_hash_from_files(files: &[PluginFileEntry]) -> String {
    let mut hasher = Sha256::new();
    for f in files {
        hasher.update(f.path.as_bytes());
        hasher.update(b"\0");
        hasher.update(f.sha256.as_bytes());
        hasher.update(b"\0");
    }
    hex_encode(&hasher.finalize())
}

fn read_plugin_version(plugin_dir: &Path) -> Option<String> {
    let candidates = [
        plugin_dir.join("claude-plugin").join("version.json"),
        plugin_dir.join("claude-plugin").join("plugin.json"),
        plugin_dir.join("plugin.json"),
    ];
    for p in &candidates {
        if let Ok(bytes) = std::fs::read(p) {
            if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                if let Some(v) = value.get("version").and_then(|x| x.as_str()) {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

fn short_hash(
    plugins: &[PluginEntry],
    mcp: &[ManagedMcpServer],
    skills: &[SkillEntry],
    agents: &[AgentEntry],
) -> String {
    let mut h = Sha256::new();
    if let Ok(s) = serde_json::to_string(plugins) {
        h.update(s.as_bytes());
    }
    h.update(b"|");
    if let Ok(s) = serde_json::to_string(mcp) {
        h.update(s.as_bytes());
    }
    h.update(b"|");
    if let Ok(s) = serde_json::to_string(skills) {
        h.update(s.as_bytes());
    }
    h.update(b"|");
    if let Ok(s) = serde_json::to_string(agents) {
        h.update(s.as_bytes());
    }
    let digest = h.finalize();
    hex_encode(&digest[..4])
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

#[allow(dead_code)]
fn _ensure_pathbuf(_: PathBuf) {}
