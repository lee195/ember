//! Profile management: multiple isolated Claude Code config directories the
//! user can try one-off or run side-by-side.
//!
//! Each profile is a self-contained `CLAUDE_CONFIG_DIR` under the app-data dir
//! (`<app_data>/profiles/<id>/`); an index (`<app_data>/profiles.json`) maps
//! `id -> {name, created_at}`. Launching opens a Terminal running `claude` with
//! `CLAUDE_CONFIG_DIR` set — `~/.claude` is never touched. Profiles hold only
//! config (no credentials; macOS auth stays in the Keychain).
//!
//! Core logic takes a `base: &Path` (the app-data dir) so it is unit-testable;
//! the `#[tauri::command]` wrappers resolve `app.path().app_data_dir()`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::portable;

// ---------- DTOs ----------

#[derive(Debug, Serialize)]
pub struct ProfileMeta {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub config_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IndexEntry {
    id: String,
    name: String,
    created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CreateSource {
    Blank,
    Clone { id: String },
    ImportBundle { path: String },
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PluginToggle {
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProfileConfig {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub default_mode: Option<String>,
    #[serde(default)]
    pub claude_md: String,
    #[serde(default)]
    pub enabled_plugins: Vec<PluginToggle>,
    #[serde(default)]
    pub skills: Vec<String>,
}

// ---------- helpers ----------

fn e2s<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

fn profiles_dir(base: &Path) -> PathBuf {
    base.join("profiles")
}
fn index_file(base: &Path) -> PathBuf {
    base.join("profiles.json")
}
fn config_dir(base: &Path, id: &str) -> PathBuf {
    profiles_dir(base).join(id)
}

fn load_index(base: &Path) -> Vec<IndexEntry> {
    std::fs::read_to_string(index_file(base))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_index(base: &Path, idx: &[IndexEntry]) -> Result<(), String> {
    std::fs::create_dir_all(base).map_err(e2s)?;
    std::fs::write(
        index_file(base),
        serde_json::to_string_pretty(idx).map_err(e2s)?,
    )
    .map_err(e2s)
}

/// Restrict ids to `[a-z0-9-]` so they're safe in paths and shell/AppleScript.
fn slugify(name: &str) -> String {
    let mut s = String::new();
    let mut prev_dash = false;
    for ch in name.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            s.push(ch);
            prev_dash = false;
        } else if !s.is_empty() && !prev_dash {
            s.push('-');
            prev_dash = true;
        }
    }
    let s = s.trim_matches('-').to_string();
    if s.is_empty() {
        "profile".to_string()
    } else {
        s
    }
}

fn unique_id(base: &Path, slug: &str) -> String {
    let idx = load_index(base);
    let taken = |c: &str| idx.iter().any(|e| e.id == c) || config_dir(base, c).exists();
    if !taken(slug) {
        return slug.to_string();
    }
    let mut n = 2;
    loop {
        let cand = format!("{slug}-{n}");
        if !taken(&cand) {
            return cand;
        }
        n += 1;
    }
}

/// POSIX single-quote a string for safe shell embedding.
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

fn read_settings(dir: &Path) -> serde_json::Value {
    std::fs::read_to_string(dir.join("settings.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}))
}

// ---------- core operations (testable) ----------

fn list(base: &Path) -> Vec<ProfileMeta> {
    load_index(base)
        .into_iter()
        .map(|e| ProfileMeta {
            config_dir: config_dir(base, &e.id).to_string_lossy().into_owned(),
            id: e.id,
            name: e.name,
            created_at: e.created_at,
        })
        .collect()
}

fn create(
    base: &Path,
    name: &str,
    source: CreateSource,
    now: String,
) -> Result<ProfileMeta, String> {
    let id = unique_id(base, &slugify(name));
    let dir = config_dir(base, &id);
    std::fs::create_dir_all(&dir).map_err(e2s)?;

    match source {
        CreateSource::Blank => {
            let settings = serde_json::json!({ "permissions": { "defaultMode": "default" } });
            std::fs::write(
                dir.join("settings.json"),
                serde_json::to_string_pretty(&settings).map_err(e2s)?,
            )
            .map_err(e2s)?;
        }
        CreateSource::Clone { id: src } => {
            let src_dir = config_dir(base, &src);
            if !src_dir.exists() {
                return Err(format!("source profile '{src}' not found"));
            }
            portable::copy_tree(&src_dir, &dir).map_err(e2s)?;
            // Stale launch script (baked with the source path) — regenerated on launch.
            let _ = std::fs::remove_file(dir.join("launch.command"));
        }
        CreateSource::ImportBundle { path } => {
            let bundle = PathBuf::from(&path);
            for item in portable::ASSETS {
                let src = bundle.join(item);
                if src.exists() {
                    portable::copy_tree(&src, &dir.join(item)).map_err(e2s)?;
                }
            }
            if !dir.join("settings.json").exists() {
                std::fs::write(dir.join("settings.json"), "{}\n").map_err(e2s)?;
            }
        }
    }

    let name = name.trim().to_string();
    let mut idx = load_index(base);
    idx.push(IndexEntry {
        id: id.clone(),
        name: name.clone(),
        created_at: now.clone(),
    });
    save_index(base, &idx)?;
    Ok(ProfileMeta {
        config_dir: dir.to_string_lossy().into_owned(),
        id,
        name,
        created_at: now,
    })
}

fn rename(base: &Path, id: &str, name: &str) -> Result<(), String> {
    let mut idx = load_index(base);
    let entry = idx
        .iter_mut()
        .find(|e| e.id == id)
        .ok_or_else(|| "profile not found".to_string())?;
    entry.name = name.trim().to_string();
    save_index(base, &idx)
}

fn delete(base: &Path, id: &str) -> Result<(), String> {
    let dir = config_dir(base, id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(e2s)?;
    }
    let mut idx = load_index(base);
    idx.retain(|e| e.id != id);
    save_index(base, &idx)
}

fn get_config(base: &Path, id: &str) -> Result<ProfileConfig, String> {
    let dir = config_dir(base, id);
    if !dir.exists() {
        return Err("profile not found".to_string());
    }
    let v = read_settings(&dir);
    let model = v.get("model").and_then(|x| x.as_str()).map(String::from);
    let default_mode = v
        .get("permissions")
        .and_then(|p| p.get("defaultMode"))
        .and_then(|x| x.as_str())
        .map(String::from);
    let enabled_plugins = v
        .get("enabledPlugins")
        .and_then(|x| x.as_object())
        .map(|o| {
            o.iter()
                .map(|(k, val)| PluginToggle {
                    name: k.clone(),
                    enabled: val.as_bool().unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default();
    let claude_md = std::fs::read_to_string(dir.join("CLAUDE.md")).unwrap_or_default();
    let mut skills = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir.join("skills")) {
        for e in rd.flatten() {
            if e.path().is_dir() {
                skills.push(e.file_name().to_string_lossy().into_owned());
            }
        }
    }
    skills.sort();
    Ok(ProfileConfig {
        model,
        default_mode,
        claude_md,
        enabled_plugins,
        skills,
    })
}

fn set_config(base: &Path, id: &str, cfg: ProfileConfig) -> Result<(), String> {
    let dir = config_dir(base, id);
    if !dir.exists() {
        return Err("profile not found".to_string());
    }
    let mut v = read_settings(&dir);
    if !v.is_object() {
        v = serde_json::json!({});
    }
    let obj = v.as_object_mut().unwrap();

    match cfg.model.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(m) => {
            obj.insert("model".into(), serde_json::json!(m));
        }
        None => {
            obj.remove("model");
        }
    }

    {
        let perms = obj
            .entry("permissions")
            .or_insert_with(|| serde_json::json!({}));
        if !perms.is_object() {
            *perms = serde_json::json!({});
        }
        let pobj = perms.as_object_mut().unwrap();
        match cfg
            .default_mode
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            Some(dm) => {
                pobj.insert("defaultMode".into(), serde_json::json!(dm));
            }
            None => {
                pobj.remove("defaultMode");
            }
        }
        if pobj.is_empty() {
            obj.remove("permissions");
        }
    }

    if cfg.enabled_plugins.is_empty() {
        obj.remove("enabledPlugins");
    } else {
        let mut m = serde_json::Map::new();
        for p in &cfg.enabled_plugins {
            m.insert(p.name.clone(), serde_json::json!(p.enabled));
        }
        obj.insert("enabledPlugins".into(), serde_json::Value::Object(m));
    }

    std::fs::write(
        dir.join("settings.json"),
        serde_json::to_string_pretty(&v).map_err(e2s)?,
    )
    .map_err(e2s)?;

    let md_path = dir.join("CLAUDE.md");
    if cfg.claude_md.trim().is_empty() {
        let _ = std::fs::remove_file(&md_path);
    } else {
        std::fs::write(&md_path, &cfg.claude_md).map_err(e2s)?;
    }

    // Reconcile skills: remove any folder not in the list (never adds).
    let keep: HashSet<&str> = cfg.skills.iter().map(|s| s.as_str()).collect();
    if let Ok(rd) = std::fs::read_dir(dir.join("skills")) {
        for e in rd.flatten() {
            if e.path().is_dir() {
                let nm = e.file_name().to_string_lossy().into_owned();
                if !keep.contains(nm.as_str()) {
                    let _ = std::fs::remove_dir_all(e.path());
                }
            }
        }
    }
    Ok(())
}

fn launch_cmd_string(base: &Path, id: &str) -> String {
    let dir = config_dir(base, id);
    format!(
        "CLAUDE_CONFIG_DIR={} claude",
        shell_quote(&dir.to_string_lossy())
    )
}

fn write_launch_script(base: &Path, id: &str) -> Result<PathBuf, String> {
    let dir = config_dir(base, id);
    if !dir.exists() {
        return Err("profile not found".to_string());
    }
    let script = dir.join("launch.command");
    let body = format!(
        "#!/bin/bash\nexport CLAUDE_CONFIG_DIR={}\nexec claude\n",
        shell_quote(&dir.to_string_lossy())
    );
    std::fs::write(&script, body).map_err(e2s)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script).map_err(e2s)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script, perms).map_err(e2s)?;
    }
    Ok(script)
}

// ---------- commands ----------

fn base(app: &AppHandle) -> Result<PathBuf, String> {
    let d = app.path().app_data_dir().map_err(e2s)?;
    std::fs::create_dir_all(&d).map_err(e2s)?;
    Ok(d)
}

#[tauri::command]
pub async fn list_profiles(app: AppHandle) -> Result<Vec<ProfileMeta>, String> {
    Ok(list(&base(&app)?))
}

#[tauri::command]
pub async fn create_profile(
    app: AppHandle,
    name: String,
    source: CreateSource,
) -> Result<ProfileMeta, String> {
    create(&base(&app)?, &name, source, chrono::Utc::now().to_rfc3339())
}

#[tauri::command]
pub async fn rename_profile(app: AppHandle, id: String, name: String) -> Result<(), String> {
    rename(&base(&app)?, &id, &name)
}

#[tauri::command]
pub async fn delete_profile(app: AppHandle, id: String) -> Result<(), String> {
    delete(&base(&app)?, &id)
}

#[tauri::command]
pub async fn get_profile_config(app: AppHandle, id: String) -> Result<ProfileConfig, String> {
    get_config(&base(&app)?, &id)
}

#[tauri::command]
pub async fn set_profile_config(
    app: AppHandle,
    id: String,
    config: ProfileConfig,
) -> Result<(), String> {
    set_config(&base(&app)?, &id, config)
}

#[tauri::command]
pub async fn profile_launch_command(app: AppHandle, id: String) -> Result<String, String> {
    Ok(launch_cmd_string(&base(&app)?, &id))
}

/// Open one Terminal window per profile running `claude` with its CLAUDE_CONFIG_DIR.
#[tauri::command]
pub async fn launch_profiles(app: AppHandle, ids: Vec<String>) -> Result<(), String> {
    let b = base(&app)?;
    for id in &ids {
        let script = write_launch_script(&b, id)?;
        // AppleScript string: the shell command Terminal will run (the quoted script path).
        let shell_cmd = shell_quote(&script.to_string_lossy());
        let escaped = shell_cmd.replace('\\', "\\\\").replace('"', "\\\"");
        let applescript = format!("tell application \"Terminal\" to do script \"{escaped}\"");
        std::process::Command::new("osascript")
            .arg("-e")
            .arg(&applescript)
            .arg("-e")
            .arg("tell application \"Terminal\" to activate")
            .output()
            .map_err(|e| format!("couldn't open Terminal: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_profile_folder(app: AppHandle, id: String) -> Result<(), String> {
    let dir = config_dir(&base(&app)?, &id);
    if !dir.exists() {
        return Err("profile not found".to_string());
    }
    std::process::Command::new("open")
        .arg(&dir)
        .output()
        .map_err(|e| format!("couldn't open folder: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ember-prof-{}-{}", std::process::id(), tag));
        let _ = std::fs::remove_dir_all(&d);
        d
    }
    const NOW: &str = "2026-06-29T00:00:00+00:00";

    #[test]
    fn create_blank_then_list() {
        let base = tmp("blank");
        let m = create(&base, "My Test", CreateSource::Blank, NOW.into()).unwrap();
        assert_eq!(m.id, "my-test");
        assert!(config_dir(&base, "my-test").join("settings.json").exists());
        let listed = list(&base);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "My Test");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn clone_copies_files() {
        let base = tmp("clone");
        create(&base, "src", CreateSource::Blank, NOW.into()).unwrap();
        std::fs::write(config_dir(&base, "src").join("CLAUDE.md"), "hello").unwrap();
        create(&base, "copy", CreateSource::Clone { id: "src".into() }, NOW.into()).unwrap();
        let md = std::fs::read_to_string(config_dir(&base, "copy").join("CLAUDE.md")).unwrap();
        assert_eq!(md, "hello");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn import_bundle_brings_config() {
        let base = tmp("import");
        let bundle = tmp("import-bundle");
        std::fs::create_dir_all(bundle.join("skills/foo")).unwrap();
        std::fs::write(bundle.join("settings.json"), r#"{"model":"x"}"#).unwrap();
        std::fs::write(bundle.join("CLAUDE.md"), "md").unwrap();
        create(&base, "imp", CreateSource::ImportBundle { path: bundle.to_string_lossy().into() }, NOW.into()).unwrap();
        let dir = config_dir(&base, "imp");
        assert!(dir.join("settings.json").exists());
        assert!(dir.join("CLAUDE.md").exists());
        assert!(dir.join("skills/foo").is_dir());
        let _ = std::fs::remove_dir_all(&base);
        let _ = std::fs::remove_dir_all(&bundle);
    }

    #[test]
    fn config_roundtrip_preserves_unknown_keys() {
        let base = tmp("cfg");
        create(&base, "p", CreateSource::Blank, NOW.into()).unwrap();
        // an unknown key that must survive edits
        std::fs::write(
            config_dir(&base, "p").join("settings.json"),
            r#"{"theme":"dark","permissions":{"defaultMode":"default"}}"#,
        )
        .unwrap();
        set_config(
            &base,
            "p",
            ProfileConfig {
                model: Some("claude-opus-4-8".into()),
                default_mode: Some("plan".into()),
                claude_md: "be terse".into(),
                enabled_plugins: vec![PluginToggle { name: "rust-analyzer".into(), enabled: true }],
                skills: vec![],
            },
        )
        .unwrap();
        let got = get_config(&base, "p").unwrap();
        assert_eq!(got.model.as_deref(), Some("claude-opus-4-8"));
        assert_eq!(got.default_mode.as_deref(), Some("plan"));
        assert_eq!(got.claude_md, "be terse");
        assert_eq!(got.enabled_plugins.len(), 1);
        // unknown key preserved
        let v = read_settings(&config_dir(&base, "p"));
        assert_eq!(v.get("theme").and_then(|x| x.as_str()), Some("dark"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn launch_command_is_quoted() {
        let base = tmp("launch");
        let cmd = launch_cmd_string(&base, "p");
        assert!(cmd.starts_with("CLAUDE_CONFIG_DIR='"));
        assert!(cmd.ends_with("/profiles/p' claude"));
    }
}
