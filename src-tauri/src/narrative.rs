//! Optional LLM-written narrative of the user's style profile.
//!
//! Provider-agnostic and local-first: a single OpenAI-compatible HTTP adapter
//! (`POST {base_url}/chat/completions`) works with Ollama, MLX (`mlx_lm.server`),
//! LM Studio, llama.cpp, and any OpenAI-compatible cloud gateway — configured by
//! base URL + model + optional API key. Local base URLs keep all data on-device.
//!
//! The API key (only needed for cloud endpoints) is stored in the OS Keychain,
//! never in the config file. The model only ever sees the already-aggregated,
//! non-secret StyleProfile — never transcripts or `.claude.json` secrets.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::profile::{self, StyleProfile};

const KEYRING_SERVICE: &str = "com.ember.desktop";
const KEYRING_USER: &str = "llm_api_key";

const SYSTEM_PROMPT: &str = "You write a short, warm, second-person personality profile of how a developer works with their AI coding assistant, based ONLY on the metrics provided — never invent facts, numbers, or tools. Write 2–3 short paragraphs of flowing prose: no headings, no bullet lists, no step-by-step analysis, and no preamble. Address the developer directly as \"you\". Output ONLY the finished profile wrapped exactly between <profile> and </profile> tags, with nothing before or after the tags. Do not show your reasoning.";

// ---------- settings ----------

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LlmSettings {
    pub base_url: String,
    pub model: String,
    /// Non-secret marker that a key is stored in the Keychain. Kept here so the
    /// common paths (opening settings, generating with a local provider) never
    /// touch the Keychain — only saving a key or generating with one does.
    #[serde(default)]
    pub has_api_key: bool,
}

/// What the frontend sees — note `has_api_key` instead of the key itself.
#[derive(Debug, Serialize)]
pub struct LlmSettingsView {
    pub base_url: String,
    pub model: String,
    pub has_api_key: bool,
}

fn config_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn settings_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(config_dir(app)?.join("llm.json"))
}

fn narrative_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(config_dir(app)?.join("narrative.txt"))
}

fn load_settings(app: &AppHandle) -> Result<LlmSettings, String> {
    let path = settings_path(app)?;
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).map_err(|e| e.to_string()),
        Err(_) => Ok(LlmSettings::default()), // not configured yet
    }
}

// ---------- keychain ----------

fn keyring_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| e.to_string())
}

fn get_api_key() -> Option<String> {
    keyring_entry()
        .ok()
        .and_then(|e| e.get_password().ok())
        .filter(|s| !s.is_empty())
}

fn store_api_key(key: &str) -> Result<(), String> {
    let entry = keyring_entry()?;
    if key.is_empty() {
        // Clearing the key: ignore "no entry" errors.
        let _ = entry.delete_credential();
        Ok(())
    } else {
        entry.set_password(key).map_err(|e| e.to_string())
    }
}

// ---------- OpenAI-compatible response parsing ----------

fn extract_content(v: &serde_json::Value) -> Option<String> {
    v.get("choices")?
        .get(0)?
        .get("message")?
        .get("content")?
        .as_str()
        .map(|s| s.to_string())
}

/// Remove every `open..close` block from `s` (e.g. `<think>…</think>`). An
/// unterminated block (truncated output) drops everything from `open` onward.
fn remove_blocks(s: &str, open: &str, close: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find(open) {
        out.push_str(&rest[..i]);
        let after = &rest[i + open.len()..];
        match after.find(close) {
            Some(j) => rest = &after[j + close.len()..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Reduce a raw model response to just the narrative prose: drop reasoning
/// blocks, then extract the `<profile>…</profile>` body if the model wrapped it
/// (tolerating a missing close tag from truncation). Falls back to the cleaned
/// whole response when no tags are present.
fn clean_narrative(raw: &str) -> String {
    let mut s = remove_blocks(raw, "<think>", "</think>");
    s = remove_blocks(&s, "<thinking>", "</thinking>");
    if let Some(start) = s.find("<profile>") {
        let inner_start = start + "<profile>".len();
        let inner = match s[inner_start..].find("</profile>") {
            Some(end) => &s[inner_start..inner_start + end],
            None => &s[inner_start..],
        };
        return inner.trim().to_string();
    }
    s.trim().to_string()
}

// ---------- prompt ----------

fn build_prompt(p: &StyleProfile) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Archetype: {}\nSummary: {}\n\nTenure: {} days with Claude Code, {} active days, {} launches.\n\nTraits:\n",
        p.archetype.title, p.archetype.summary, p.tenure_days, p.total_active_days, p.num_startups
    ));
    for t in &p.traits {
        out.push_str(&format!("- {}: {} — {} ({})\n", t.title, t.value, t.detail, t.evidence));
    }
    out.push_str("\nTech stack (by files touched): ");
    out.push_str(
        &p.stack
            .languages
            .iter()
            .map(|l| format!("{} {}", l.language, l.count))
            .collect::<Vec<_>>()
            .join(", "),
    );
    out.push_str(&format!(
        "\n\nRhythm: peak hours {:?} (UTC), busiest {}, ~{:.0} min/session, {:.1} sessions/active day.",
        p.rhythm.peak_hours, p.rhythm.busiest_weekday, p.rhythm.avg_session_minutes, p.rhythm.sessions_per_active_day
    ));
    out.push_str(&format!(
        "\nAutonomy: auto {}, plan {}, default {}, accept-edits {} (dominant: {}).",
        p.autonomy.auto, p.autonomy.plan, p.autonomy.default, p.autonomy.accept_edits, p.autonomy.dominant
    ));
    out.push_str(&format!(
        "\nCommunication: {} prompts, avg {:.0} words, {:.0}% questions, {:.0}% quick commands, {:.0}% polite.",
        p.communication.prompts,
        p.communication.avg_prompt_words,
        p.communication.question_ratio * 100.0,
        p.communication.short_command_ratio * 100.0,
        p.communication.politeness_ratio * 100.0
    ));
    let skills: Vec<_> = p.skills.iter().filter(|s| s.usage_count > 0).collect();
    if !skills.is_empty() {
        out.push_str("\nSkills used: ");
        out.push_str(
            &skills
                .iter()
                .map(|s| format!("{} ({}x)", s.name, s.usage_count))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    out
}

// ---------- commands ----------

#[tauri::command]
pub async fn get_llm_settings(app: AppHandle) -> Result<LlmSettingsView, String> {
    let s = load_settings(&app)?;
    Ok(LlmSettingsView {
        base_url: s.base_url,
        model: s.model,
        has_api_key: s.has_api_key, // from config — no Keychain access
    })
}

#[tauri::command]
pub async fn set_llm_settings(
    app: AppHandle,
    base_url: String,
    model: String,
    api_key: Option<String>,
) -> Result<(), String> {
    // Preserve the existing key flag when the caller doesn't touch the key.
    let mut settings = load_settings(&app).unwrap_or_default();
    settings.base_url = base_url.trim().to_string();
    settings.model = model.trim().to_string();
    // `Some` (including empty string to clear) updates the key; `None` leaves it.
    if let Some(key) = api_key {
        let key = key.trim();
        store_api_key(key)?; // the only Keychain write — a deliberate Save action
        settings.has_api_key = !key.is_empty();
    }
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(settings_path(&app)?, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_cached_narrative(app: AppHandle) -> Result<Option<String>, String> {
    Ok(std::fs::read_to_string(narrative_path(&app)?).ok())
}

#[tauri::command]
pub async fn generate_narrative(app: AppHandle) -> Result<String, String> {
    let settings = load_settings(&app)?;
    if settings.base_url.is_empty() || settings.model.is_empty() {
        return Err("No model configured — set a provider base URL and model first.".into());
    }

    let profile = tauri::async_runtime::spawn_blocking(profile::build_profile)
        .await
        .map_err(|e| e.to_string())?;
    if !profile.found {
        return Err("No Claude Code usage data found to describe.".into());
    }

    let url = format!("{}/chat/completions", settings.base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": settings.model,
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": build_prompt(&profile) },
        ],
        "max_tokens": 4096,
        "temperature": 0.7,
        "stream": false,
    });

    let client = reqwest::Client::new();
    let mut req = client.post(&url).json(&body);
    // Only touch the Keychain when a key was actually saved (cloud endpoints).
    // Local providers leave `has_api_key` false, so this never prompts.
    if settings.has_api_key {
        if let Some(key) = get_api_key() {
            req = req.bearer_auth(key);
        }
    }

    let resp = req.send().await.map_err(|e| {
        format!(
            "Couldn't reach the model at {} — is the server running? ({})",
            settings.base_url, e
        )
    })?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        let snippet: String = text.chars().take(300).collect();
        return Err(format!("Model server returned {}: {}", status.as_u16(), snippet));
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Unexpected response from model server: {e}"))?;
    let raw = extract_content(&parsed)
        .ok_or_else(|| "The model returned no content.".to_string())?;
    let content = clean_narrative(&raw);
    if content.is_empty() {
        return Err("The model returned an empty response.".into());
    }

    // Cache for next launch (best-effort).
    if let Ok(path) = narrative_path(&app) {
        let _ = std::fs::write(path, &content);
    }
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_openai_content() {
        // extract_content returns the raw content; trimming/cleaning is clean_narrative's job.
        let sample = r#"{"choices":[{"message":{"role":"assistant","content":"You are a builder."}}]}"#;
        let v: serde_json::Value = serde_json::from_str(sample).unwrap();
        assert_eq!(extract_content(&v).as_deref(), Some("You are a builder."));
    }

    #[test]
    fn missing_content_is_none() {
        let v: serde_json::Value = serde_json::from_str(r#"{"choices":[]}"#).unwrap();
        assert!(extract_content(&v).is_none());
    }

    #[test]
    fn clean_extracts_profile_and_strips_think() {
        let raw = "<think>plan plan plan</think>\nHere's a thinking process:\n<profile>You are a builder.\n\nYou ship code.</profile>\ntrailing junk";
        assert_eq!(clean_narrative(raw), "You are a builder.\n\nYou ship code.");
    }

    #[test]
    fn clean_handles_unterminated_profile() {
        assert_eq!(clean_narrative("<profile>partial that got cut"), "partial that got cut");
    }

    #[test]
    fn clean_drops_unterminated_think() {
        // Truncated mid-reasoning with no profile tag → nothing usable.
        assert_eq!(clean_narrative("<think>1. analyze 2. draft"), "");
    }

    #[test]
    fn clean_passthrough_without_tags() {
        assert_eq!(clean_narrative("  Just prose.  "), "Just prose.");
    }

    #[test]
    fn settings_roundtrip() {
        let s = LlmSettings {
            base_url: "http://localhost:11434/v1".into(),
            model: "llama3.1".into(),
            has_api_key: true,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: LlmSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.base_url, s.base_url);
        assert_eq!(back.model, s.model);
        assert!(back.has_api_key);
        // Older config files without the field deserialize with has_api_key = false.
        let legacy: LlmSettings =
            serde_json::from_str(r#"{"base_url":"x","model":"y"}"#).unwrap();
        assert!(!legacy.has_api_key);
    }
}
