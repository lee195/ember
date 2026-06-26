//! Reads and aggregates local Claude Code usage data from `~/.claude/`.
//!
//! Source of truth is the per-session transcript: `projects/<enc>/<uuid>.jsonl`,
//! one JSON object per line. We only deserialize the handful of fields we need
//! and ignore everything else (the schema has many line types). No credential
//! files are read here — only transcripts and their non-secret metadata.
//!
//! A single `scan_all()` walk fills one `Scan` accumulator; both the usage
//! `Overview` (Phase 1) and the style `StyleProfile` (Phase 2, in `profile.rs`)
//! are derived from that one scan, so the transcripts are parsed only once.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};

// ---------- Raw transcript shapes (lenient) ----------

#[derive(Debug, Deserialize)]
struct RawLine {
    #[serde(rename = "type")]
    kind: Option<String>,
    timestamp: Option<String>,
    cwd: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    #[serde(rename = "permissionMode")]
    permission_mode: Option<String>,
    #[serde(rename = "isMeta")]
    is_meta: Option<bool>,
    message: Option<RawMessage>,
}

#[derive(Debug, Deserialize)]
struct RawMessage {
    #[allow(dead_code)]
    role: Option<String>,
    model: Option<String>,
    usage: Option<RawUsage>,
    #[serde(default)]
    content: serde_json::Value,
}

#[derive(Debug, Default, Deserialize)]
struct RawUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
    #[serde(default)]
    server_tool_use: Option<ServerToolUse>,
}

#[derive(Debug, Default, Deserialize)]
struct ServerToolUse {
    #[serde(default)]
    web_search_requests: u64,
    #[serde(default)]
    web_fetch_requests: u64,
}

// ---------- DTOs returned to the frontend ----------

#[derive(Debug, Default, Serialize)]
pub struct Totals {
    pub sessions: u64,
    pub projects: u64,
    pub user_messages: u64,
    pub assistant_messages: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub tool_calls: u64,
    pub web_searches: u64,
    pub web_fetches: u64,
}

#[derive(Debug, Serialize)]
pub struct DailyPoint {
    pub date: String, // YYYY-MM-DD
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub messages: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelUsage {
    pub model: String,
    pub messages: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
}

#[derive(Debug, Serialize)]
pub struct ProjectSummary {
    pub path: String,
    pub name: String,
    pub sessions: u64,
    pub messages: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub last_active: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolUsage {
    pub name: String,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct Overview {
    pub claude_dir: String,
    pub found: bool,
    pub totals: Totals,
    pub daily: Vec<DailyPoint>,
    pub hourly: Vec<u64>, // length 24, hour-of-day (UTC)
    pub models: Vec<ModelUsage>,
    pub projects: Vec<ProjectSummary>,
    pub tools: Vec<ToolUsage>,
    pub date_range: Option<(String, String)>,
}

impl Overview {
    fn empty(claude_dir: String, found: bool) -> Self {
        Overview {
            claude_dir,
            found,
            totals: Totals::default(),
            daily: Vec::new(),
            hourly: vec![0; 24],
            models: Vec::new(),
            projects: Vec::new(),
            tools: Vec::new(),
            date_range: None,
        }
    }
}

// ---------- Aggregation accumulators ----------

#[derive(Default)]
pub struct DayAcc {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub messages: u64,
}

#[derive(Default)]
pub struct ModelAcc {
    pub messages: u64,
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
}

#[derive(Default)]
pub struct ProjAcc {
    pub sessions: HashSet<String>,
    pub messages: u64,
    pub input: u64,
    pub output: u64,
    pub last_active: Option<DateTime<Utc>>,
}

/// Aggregate stats over real user prompts. Only derived numbers are kept —
/// raw prompt text is never stored.
#[derive(Default)]
pub struct PromptStats {
    pub prompts: u64,
    pub total_words: u64,
    pub total_chars: u64,
    pub questions: u64,       // contains '?'
    pub short_commands: u64,  // 1..=5 words
    pub polite: u64,          // please / thanks / could you ...
    pub specific: u64,        // references code: backticks, paths, filenames
    pub unique_words: HashSet<String>,
}

impl PromptStats {
    fn add(&mut self, text: &str) {
        let t = text.trim();
        if t.is_empty() {
            return;
        }
        self.prompts += 1;
        self.total_chars += t.chars().count() as u64;
        let words: Vec<&str> = t.split_whitespace().collect();
        let wc = words.len() as u64;
        self.total_words += wc;
        if t.contains('?') {
            self.questions += 1;
        }
        if (1..=5).contains(&wc) {
            self.short_commands += 1;
        }
        let lower = t.to_lowercase();
        if ["please", "thanks", "thank you", "could you", "would you", "appreciate"]
            .iter()
            .any(|k| lower.contains(k))
        {
            self.polite += 1;
        }
        if t.contains('`')
            || t.contains('/')
            || words.iter().any(|w| w.len() > 3 && w.contains('.'))
        {
            self.specific += 1;
        }
        for w in words {
            let cleaned: String = w
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            if !cleaned.is_empty() {
                self.unique_words.insert(cleaned);
            }
        }
    }
}

/// Everything collected in one transcript walk.
#[derive(Default)]
pub struct Scan {
    pub totals: Totals,
    pub session_files: u64,
    pub days: BTreeMap<String, DayAcc>,
    pub hourly: Vec<u64>,  // len 24
    pub weekday: Vec<u64>, // len 7, Mon=0
    pub models: HashMap<String, ModelAcc>,
    pub projects: HashMap<String, ProjAcc>,
    pub tools: HashMap<String, u64>,
    pub min_date: Option<String>,
    pub max_date: Option<String>,
    pub permission_modes: HashMap<String, u64>,
    /// All message timestamps per session, used to estimate active working time.
    pub session_times: HashMap<String, Vec<DateTime<Utc>>>,
    pub file_exts: HashMap<String, u64>,
    pub prompt: PromptStats,
}

impl Scan {
    fn new() -> Self {
        Scan {
            hourly: vec![0; 24],
            weekday: vec![0; 7],
            ..Default::default()
        }
    }

    fn ingest(&mut self, line: RawLine) {
        let kind = line.kind.as_deref().unwrap_or("");
        let ts = line
            .timestamp
            .as_deref()
            .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let date_key = ts.map(|t| t.format("%Y-%m-%d").to_string());

        if let Some(ref d) = date_key {
            if self.min_date.as_ref().map(|m| d < m).unwrap_or(true) {
                self.min_date = Some(d.clone());
            }
            if self.max_date.as_ref().map(|m| d > m).unwrap_or(true) {
                self.max_date = Some(d.clone());
            }
        }
        if let Some(t) = ts {
            self.hourly[t.hour() as usize] += 1;
            self.weekday[t.weekday().num_days_from_monday() as usize] += 1;
        }

        if let Some(pm) = &line.permission_mode {
            *self.permission_modes.entry(pm.clone()).or_insert(0) += 1;
        }

        let session_id = line.session_id.clone().unwrap_or_default();
        if !session_id.is_empty() {
            if let Some(t) = ts {
                self.session_times
                    .entry(session_id.clone())
                    .or_default()
                    .push(t);
            }
        }

        let proj_path = line.cwd.clone();

        match kind {
            "user" => {
                self.totals.user_messages += 1;
                touch_project(&mut self.projects, &proj_path, &session_id, ts, 1, 0, 0);
                if let Some(ref d) = date_key {
                    self.days.entry(d.clone()).or_default().messages += 1;
                }
                // Prompt-text heuristics — skip injected/meta and command wrappers.
                if line.is_meta != Some(true) {
                    if let Some(msg) = &line.message {
                        if let Some(text) = extract_prompt_text(&msg.content) {
                            let trimmed = text.trim_start();
                            if !trimmed.starts_with('<') {
                                self.prompt.add(&text);
                            }
                        }
                    }
                }
            }
            "assistant" => {
                self.totals.assistant_messages += 1;
                let msg = match line.message {
                    Some(m) => m,
                    None => return,
                };
                let model = msg.model.clone().unwrap_or_else(|| "unknown".to_string());

                let mut in_tok = 0;
                let mut out_tok = 0;
                let mut cache_read = 0;
                if let Some(u) = &msg.usage {
                    in_tok = u.input_tokens;
                    out_tok = u.output_tokens;
                    cache_read = u.cache_read_input_tokens;
                    self.totals.input_tokens += u.input_tokens;
                    self.totals.output_tokens += u.output_tokens;
                    self.totals.cache_creation_tokens += u.cache_creation_input_tokens;
                    self.totals.cache_read_tokens += u.cache_read_input_tokens;
                    if let Some(st) = &u.server_tool_use {
                        self.totals.web_searches += st.web_search_requests;
                        self.totals.web_fetches += st.web_fetch_requests;
                    }
                }

                // Tool-use calls (and the files they touch) live in the content array.
                if let Some(items) = msg.content.as_array() {
                    for item in items {
                        if item.get("type").and_then(|v| v.as_str()) != Some("tool_use") {
                            continue;
                        }
                        self.totals.tool_calls += 1;
                        let name = item
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        *self.tools.entry(name.to_string()).or_insert(0) += 1;

                        if matches!(name, "Edit" | "Write" | "Read" | "NotebookEdit") {
                            if let Some(fp) = item
                                .get("input")
                                .and_then(|i| i.get("file_path").or_else(|| i.get("path")))
                                .and_then(|v| v.as_str())
                            {
                                if let Some(ext) = file_ext(fp) {
                                    *self.file_exts.entry(ext).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                }

                let m = self.models.entry(model).or_default();
                m.messages += 1;
                m.input += in_tok;
                m.output += out_tok;
                m.cache_read += cache_read;

                if let Some(ref d) = date_key {
                    let day = self.days.entry(d.clone()).or_default();
                    day.input += in_tok;
                    day.output += out_tok;
                    day.cache_read += cache_read;
                    day.messages += 1;
                }
                touch_project(
                    &mut self.projects,
                    &proj_path,
                    &session_id,
                    ts,
                    1,
                    in_tok,
                    out_tok,
                );
            }
            _ => {}
        }
    }
}

pub fn claude_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude"))
}

/// Derive a human-friendly project name from its filesystem path.
pub fn project_name(path: &str) -> String {
    path.rsplit('/')
        .find(|s| !s.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn file_ext(path: &str) -> Option<String> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let (_, ext) = name.rsplit_once('.')?;
    if ext.is_empty() || ext.len() > 8 || ext.contains(' ') {
        return None;
    }
    Some(ext.to_lowercase())
}

fn extract_prompt_text(content: &serde_json::Value) -> Option<String> {
    match content {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Array(items) => {
            let mut buf = String::new();
            for it in items {
                if it.get("type").and_then(|v| v.as_str()) == Some("text") {
                    if let Some(t) = it.get("text").and_then(|v| v.as_str()) {
                        buf.push_str(t);
                        buf.push(' ');
                    }
                }
            }
            if buf.trim().is_empty() {
                None
            } else {
                Some(buf)
            }
        }
        _ => None,
    }
}

fn touch_project(
    projects: &mut HashMap<String, ProjAcc>,
    proj_path: &Option<String>,
    session_id: &str,
    ts: Option<DateTime<Utc>>,
    messages: u64,
    input: u64,
    output: u64,
) {
    let path = match proj_path {
        Some(p) if !p.is_empty() => p.clone(),
        _ => return,
    };
    let acc = projects.entry(path).or_default();
    if !session_id.is_empty() {
        acc.sessions.insert(session_id.to_string());
    }
    acc.messages += messages;
    acc.input += input;
    acc.output += output;
    if let Some(t) = ts {
        if acc.last_active.map(|cur| t > cur).unwrap_or(true) {
            acc.last_active = Some(t);
        }
    }
}

/// Walk every top-level `projects/*/*.jsonl` transcript once.
/// Returns the `~/.claude` path, whether it was found, and the filled scan.
pub fn scan_all() -> (String, bool, Scan) {
    let dir = match claude_dir() {
        Some(d) => d,
        None => return (String::new(), false, Scan::new()),
    };
    let dir_str = dir.to_string_lossy().to_string();
    let projects_dir = dir.join("projects");
    if !projects_dir.is_dir() {
        return (dir_str, false, Scan::new());
    }

    let mut scan = Scan::new();
    let entries = match std::fs::read_dir(&projects_dir) {
        Ok(e) => e,
        Err(_) => return (dir_str, true, scan),
    };

    for proj_entry in entries.flatten() {
        let proj_path = proj_entry.path();
        if !proj_path.is_dir() {
            continue;
        }
        let session_iter = match std::fs::read_dir(&proj_path) {
            Ok(i) => i,
            Err(_) => continue,
        };
        for file_entry in session_iter.flatten() {
            let fpath = file_entry.path();
            if fpath.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            scan.session_files += 1;
            let content = match std::fs::read_to_string(&fpath) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(parsed) = serde_json::from_str::<RawLine>(line) {
                    scan.ingest(parsed);
                }
            }
        }
    }

    scan.totals.sessions = scan.session_files;
    scan.totals.projects = scan.projects.len() as u64;
    (dir_str, true, scan)
}

// ---------- Shared derived views (used by both overview & profile) ----------

pub fn sorted_models(scan: &Scan) -> Vec<ModelUsage> {
    let mut v: Vec<ModelUsage> = scan
        .models
        .iter()
        .map(|(model, a)| ModelUsage {
            model: model.clone(),
            messages: a.messages,
            input_tokens: a.input,
            output_tokens: a.output,
            cache_read_tokens: a.cache_read,
        })
        .collect();
    v.sort_by(|a, b| {
        (b.input_tokens + b.output_tokens).cmp(&(a.input_tokens + a.output_tokens))
    });
    v
}

pub fn sorted_tools(scan: &Scan) -> Vec<ToolUsage> {
    let mut v: Vec<ToolUsage> = scan
        .tools
        .iter()
        .map(|(name, count)| ToolUsage {
            name: name.clone(),
            count: *count,
        })
        .collect();
    v.sort_by(|a, b| b.count.cmp(&a.count));
    v
}

// ---------- Overview (Phase 1) ----------

pub fn build_overview() -> Overview {
    let (dir_str, found, scan) = scan_all();
    if !found {
        return Overview::empty(dir_str, found);
    }

    let daily: Vec<DailyPoint> = scan
        .days
        .iter()
        .map(|(date, a)| DailyPoint {
            date: date.clone(),
            input_tokens: a.input,
            output_tokens: a.output,
            cache_read_tokens: a.cache_read,
            messages: a.messages,
        })
        .collect();

    let mut project_vec: Vec<ProjectSummary> = scan
        .projects
        .iter()
        .map(|(path, a)| ProjectSummary {
            name: project_name(path),
            path: path.clone(),
            sessions: a.sessions.len() as u64,
            messages: a.messages,
            input_tokens: a.input,
            output_tokens: a.output,
            last_active: a.last_active.map(|t| t.to_rfc3339()),
        })
        .collect();
    project_vec.sort_by(|a, b| {
        (b.input_tokens + b.output_tokens).cmp(&(a.input_tokens + a.output_tokens))
    });

    let date_range = match (&scan.min_date, &scan.max_date) {
        (Some(lo), Some(hi)) => Some((lo.clone(), hi.clone())),
        _ => None,
    };

    Overview {
        claude_dir: dir_str,
        found: true,
        models: sorted_models(&scan),
        tools: sorted_tools(&scan),
        hourly: scan.hourly.clone(),
        totals: scan.totals,
        daily,
        projects: project_vec,
        date_range,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test against the real local ~/.claude data. Prints a summary so the
    /// numbers can be cross-checked against an independent count.
    #[test]
    fn overview_against_real_data() {
        let o = build_overview();
        eprintln!("found={} dir={}", o.found, o.claude_dir);
        eprintln!(
            "sessions={} projects={} user_msgs={} assistant_msgs={}",
            o.totals.sessions, o.totals.projects, o.totals.user_messages, o.totals.assistant_messages
        );
        eprintln!(
            "input={} output={} cache_read={} tool_calls={}",
            o.totals.input_tokens, o.totals.output_tokens, o.totals.cache_read_tokens, o.totals.tool_calls
        );
        let hourly_sum: u64 = o.hourly.iter().sum();
        eprintln!("hourly_sum={} daily_points={}", hourly_sum, o.daily.len());
        assert_eq!(o.hourly.len(), 24);
        if o.found {
            assert!(o.totals.assistant_messages > 0);
        }
    }
}
