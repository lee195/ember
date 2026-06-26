//! Derives a deterministic, offline "personal style" profile from the shared
//! transcript `Scan` plus a few non-secret keys of `~/.claude.json`.
//!
//! No network, no LLM, no credentials: the `.claude.json` reader declares only
//! whitelisted fields, so serde silently drops everything else (including
//! `oauthAccount`, `userID`, API keys). Raw prompt text is never read here —
//! only the aggregate `PromptStats` collected during the scan.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::claude_data::{self, ModelUsage, Scan, ToolUsage};

// ---------- whitelisted ~/.claude.json metadata ----------

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeJsonMeta {
    #[serde(default)]
    num_startups: u64,
    first_start_time: Option<String>,
    claude_code_first_token_date: Option<String>,
    skill_usage: Option<HashMap<String, UsageEntry>>,
    plugin_usage: Option<HashMap<String, UsageEntry>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsageEntry {
    #[serde(default)]
    usage_count: u64,
    last_used_at: Option<i64>, // unix millis
}

fn read_claude_json_meta() -> ClaudeJsonMeta {
    let path = match dirs::home_dir() {
        Some(h) => h.join(".claude.json"),
        None => return ClaudeJsonMeta::default(),
    };
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return ClaudeJsonMeta::default(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

// ---------- DTOs ----------

#[derive(Debug, Serialize)]
pub struct StyleProfile {
    pub found: bool,
    pub claude_dir: String,
    pub tenure_days: i64,
    pub first_active: Option<String>,
    pub num_startups: u64,
    pub total_active_days: u64,
    pub archetype: Archetype,
    pub traits: Vec<Trait>,
    pub autonomy: Autonomy,
    pub rhythm: Rhythm,
    pub stack: Stack,
    pub tools_top: Vec<ToolUsage>,
    pub models_top: Vec<ModelUsage>,
    pub skills: Vec<NamedUsage>,
    pub plugins: Vec<NamedUsage>,
    pub communication: Communication,
    pub cache_efficiency: f64,
}

#[derive(Debug, Serialize)]
pub struct Archetype {
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Serialize)]
pub struct Trait {
    pub key: String,
    pub title: String,   // category, e.g. "Autonomy"
    pub value: String,   // verdict, e.g. "Hands-off operator"
    pub detail: String,  // one-line description
    pub evidence: String, // backing metric
}

#[derive(Debug, Default, Serialize)]
pub struct Autonomy {
    pub plan: u64,
    pub auto: u64,
    pub default: u64,
    pub accept_edits: u64,
    pub dominant: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Rhythm {
    pub peak_hours: Vec<usize>,
    pub busiest_weekday: String,
    pub avg_session_minutes: f64,
    pub sessions_per_active_day: f64,
    pub active_days: u64,
}

#[derive(Debug, Serialize)]
pub struct LangCount {
    pub language: String,
    pub count: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct Stack {
    pub languages: Vec<LangCount>,
    pub primary: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NamedUsage {
    pub name: String,
    pub usage_count: u64,
    pub last_used: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Communication {
    pub prompts: u64,
    pub avg_prompt_words: f64,
    pub question_ratio: f64,
    pub short_command_ratio: f64,
    pub politeness_ratio: f64,
    pub specificity_ratio: f64,
    pub vocabulary_richness: f64,
}

// ---------- command entry point ----------

pub fn build_profile() -> StyleProfile {
    let (dir_str, found, scan) = claude_data::scan_all();
    let meta = read_claude_json_meta();
    derive_profile(dir_str, found, &scan, &meta)
}

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

fn ext_to_language(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "rs" => "Rust",
        "ts" | "mts" | "cts" => "TypeScript",
        "tsx" => "TypeScript (React)",
        "js" | "mjs" | "cjs" => "JavaScript",
        "jsx" => "JavaScript (React)",
        "vue" => "Vue",
        "py" => "Python",
        "go" => "Go",
        "rb" => "Ruby",
        "java" => "Java",
        "kt" => "Kotlin",
        "swift" => "Swift",
        "c" | "h" => "C",
        "cpp" | "cc" | "hpp" => "C++",
        "cs" => "C#",
        "php" => "PHP",
        "sh" | "bash" | "fish" | "zsh" => "Shell",
        "sql" => "SQL",
        "prisma" => "Prisma",
        "md" | "mdx" => "Markdown",
        "css" | "scss" | "sass" => "CSS",
        "html" => "HTML",
        "json" | "toml" | "yaml" | "yml" => "Config",
        _ => return None,
    })
}

fn ratio(n: u64, d: u64) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 / d as f64
    }
}

fn ms_to_date(ms: i64) -> Option<String> {
    DateTime::from_timestamp_millis(ms).map(|dt| dt.format("%Y-%m-%d").to_string())
}

fn named_usage_sorted(map: &Option<HashMap<String, UsageEntry>>) -> Vec<NamedUsage> {
    let mut v: Vec<NamedUsage> = map
        .as_ref()
        .map(|m| {
            m.iter()
                .map(|(name, e)| NamedUsage {
                    name: name.clone(),
                    usage_count: e.usage_count,
                    last_used: e.last_used_at.and_then(ms_to_date),
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
    v
}

fn derive_profile(
    dir_str: String,
    found: bool,
    scan: &Scan,
    meta: &ClaudeJsonMeta,
) -> StyleProfile {
    // --- autonomy ---
    let pm = &scan.permission_modes;
    let autonomy = Autonomy {
        plan: *pm.get("plan").unwrap_or(&0),
        auto: *pm.get("auto").unwrap_or(&0),
        default: *pm.get("default").unwrap_or(&0),
        accept_edits: *pm.get("acceptEdits").unwrap_or(&0),
        dominant: pm
            .iter()
            .max_by_key(|(_, v)| **v)
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| "unknown".into()),
    };
    let autonomy_total =
        autonomy.plan + autonomy.auto + autonomy.default + autonomy.accept_edits;
    let auto_share = ratio(autonomy.auto + autonomy.accept_edits, autonomy_total);
    let plan_share = ratio(autonomy.plan, autonomy_total);

    // --- rhythm ---
    let mut hour_idx: Vec<usize> = (0..24).collect();
    hour_idx.sort_by(|&a, &b| scan.hourly[b].cmp(&scan.hourly[a]));
    let peak_hours: Vec<usize> = hour_idx.into_iter().take(3).collect();
    let busiest_weekday = scan
        .weekday
        .iter()
        .enumerate()
        .max_by_key(|(_, v)| **v)
        .map(|(i, _)| WEEKDAYS[i].to_string())
        .unwrap_or_default();
    // Active working time per session: sum gaps between consecutive messages,
    // ignoring gaps over the idle threshold (a resumed session can span days of
    // wall-clock, so raw first→last span hugely overstates real working time).
    const IDLE_MINUTES: f64 = 30.0;
    let mut total_minutes = 0.0;
    let mut counted = 0u64;
    for times in scan.session_times.values() {
        let mut ts: Vec<DateTime<Utc>> = times.clone();
        ts.sort();
        let mut active = 0.0;
        for w in ts.windows(2) {
            let gap = (w[1] - w[0]).num_seconds() as f64 / 60.0;
            if gap > 0.0 && gap <= IDLE_MINUTES {
                active += gap;
            }
        }
        total_minutes += active;
        counted += 1;
    }
    let avg_session_minutes = if counted > 0 {
        total_minutes / counted as f64
    } else {
        0.0
    };
    let active_days = scan.days.len() as u64;
    let rhythm = Rhythm {
        peak_hours: peak_hours.clone(),
        busiest_weekday: busiest_weekday.clone(),
        avg_session_minutes,
        sessions_per_active_day: ratio(scan.totals.sessions, active_days),
        active_days,
    };

    // --- stack ---
    let mut lang_counts: HashMap<&'static str, u64> = HashMap::new();
    for (ext, count) in &scan.file_exts {
        if let Some(lang) = ext_to_language(ext) {
            *lang_counts.entry(lang).or_insert(0) += count;
        }
    }
    let mut languages: Vec<LangCount> = lang_counts
        .into_iter()
        .map(|(language, count)| LangCount {
            language: language.to_string(),
            count,
        })
        .collect();
    languages.sort_by(|a, b| b.count.cmp(&a.count));
    // Primary = top non-doc, non-config language if available.
    let primary = languages
        .iter()
        .find(|l| l.language != "Markdown" && l.language != "Config")
        .or_else(|| languages.first())
        .map(|l| l.language.clone());
    let stack = Stack {
        languages: languages.into_iter().take(8).collect(),
        primary: primary.clone(),
    };

    // --- communication ---
    let p = &scan.prompt;
    let communication = Communication {
        prompts: p.prompts,
        avg_prompt_words: ratio(p.total_words, p.prompts),
        question_ratio: ratio(p.questions, p.prompts),
        short_command_ratio: ratio(p.short_commands, p.prompts),
        politeness_ratio: ratio(p.polite, p.prompts),
        specificity_ratio: ratio(p.specific, p.prompts),
        vocabulary_richness: ratio(p.unique_words.len() as u64, p.total_words),
    };

    // --- tools / models / skills / plugins ---
    let tools_top: Vec<ToolUsage> =
        claude_data::sorted_tools(scan).into_iter().take(10).collect();
    let models_top = claude_data::sorted_models(scan);
    let skills = named_usage_sorted(&meta.skill_usage);
    let plugins = named_usage_sorted(&meta.plugin_usage);

    let cache_efficiency = ratio(
        scan.totals.cache_read_tokens,
        scan.totals.cache_read_tokens + scan.totals.input_tokens,
    );

    // --- tenure ---
    let first_active = meta
        .first_start_time
        .clone()
        .or_else(|| meta.claude_code_first_token_date.clone());
    let tenure_days = first_active
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_days())
        .unwrap_or(0);

    // --- traits + archetype ---
    let mut traits: Vec<Trait> = Vec::new();

    // Autonomy
    let (autonomy_verdict, autonomy_adj) = if auto_share >= 0.5 {
        ("Hands-off operator", "Hands-off")
    } else if plan_share >= 0.3 {
        ("Deliberate planner", "Methodical")
    } else {
        ("Balanced driver", "Balanced")
    };
    traits.push(Trait {
        key: "autonomy".into(),
        title: "Autonomy".into(),
        value: autonomy_verdict.into(),
        detail: format!(
            "You run in auto/accept mode {:.0}% of the time and plan mode {:.0}%.",
            auto_share * 100.0,
            plan_share * 100.0
        ),
        evidence: format!(
            "auto {} · plan {} · default {} · accept {}",
            autonomy.auto, autonomy.plan, autonomy.default, autonomy.accept_edits
        ),
    });

    // Tooling: builder vs explorer
    let tool_count = |name: &str| scan.tools.get(name).copied().unwrap_or(0);
    let edits = tool_count("Edit") + tool_count("Write") + tool_count("NotebookEdit");
    let reads = tool_count("Read") + tool_count("Grep") + tool_count("Glob");
    let (tooling_verdict, tooling_adj) = if edits > reads {
        ("Builder", "Builder")
    } else {
        ("Explorer", "Explorer")
    };
    traits.push(Trait {
        key: "tooling".into(),
        title: "Working mode".into(),
        value: tooling_verdict.into(),
        detail: if edits > reads {
            "You spend more calls writing code than reading it.".into()
        } else {
            "You spend more calls reading and searching than writing.".into()
        },
        evidence: format!("{} edits · {} reads/searches", edits, reads),
    });

    // Time of day
    let peak = peak_hours.first().copied().unwrap_or(0);
    let time_verdict = if (20..24).contains(&peak) || peak < 6 {
        "Night owl"
    } else if (5..11).contains(&peak) {
        "Early bird"
    } else {
        "Daytime worker"
    };
    traits.push(Trait {
        key: "time".into(),
        title: "Rhythm".into(),
        value: time_verdict.into(),
        detail: format!(
            "Most active around {:02}:00 (UTC); busiest on {}.",
            peak, busiest_weekday
        ),
        evidence: format!(
            "peak hours {}",
            peak_hours
                .iter()
                .map(|h| format!("{:02}:00", h))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    });

    // Session length
    let session_verdict = if avg_session_minutes >= 45.0 {
        "Marathoner"
    } else if avg_session_minutes < 15.0 {
        "Sprinter"
    } else {
        "Steady worker"
    };
    traits.push(Trait {
        key: "session".into(),
        title: "Session length".into(),
        value: session_verdict.into(),
        detail: format!("Sessions average {:.0} minutes.", avg_session_minutes),
        evidence: format!(
            "{} sessions · {:.1}/active day",
            scan.totals.sessions, rhythm.sessions_per_active_day
        ),
    });

    // Communication
    let comm_verdict = if communication.avg_prompt_words < 12.0 {
        "Terse commander"
    } else if communication.avg_prompt_words > 35.0 {
        "Detailed director"
    } else {
        "Conversational"
    };
    traits.push(Trait {
        key: "communication".into(),
        title: "Communication".into(),
        value: comm_verdict.into(),
        detail: format!(
            "Prompts average {:.0} words; {:.0}% are quick commands.",
            communication.avg_prompt_words,
            communication.short_command_ratio * 100.0
        ),
        evidence: format!(
            "{} prompts · {:.0}% questions · {:.0}% polite",
            communication.prompts,
            communication.question_ratio * 100.0,
            communication.politeness_ratio * 100.0
        ),
    });

    // Stack
    if let Some(prim) = &primary {
        traits.push(Trait {
            key: "stack".into(),
            title: "Primary stack".into(),
            value: prim.clone(),
            detail: format!("Most of your file activity is in {}.", prim),
            evidence: stack
                .languages
                .iter()
                .take(4)
                .map(|l| format!("{} {}", l.language, l.count))
                .collect::<Vec<_>>()
                .join(" · "),
        });
    }

    // Skills power-user
    let skill_total: u64 = skills.iter().map(|s| s.usage_count).sum();
    if skill_total > 0 {
        let top_skill = skills
            .iter()
            .find(|s| s.usage_count > 0)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        traits.push(Trait {
            key: "skills".into(),
            title: "Skills".into(),
            value: if skill_total >= 10 {
                "Skill power-user".into()
            } else {
                "Skill explorer".into()
            },
            detail: format!("You've invoked custom skills {} times.", skill_total),
            evidence: if top_skill.is_empty() {
                String::new()
            } else {
                format!("top: {}", top_skill)
            },
        });
    }

    // Archetype: combine the strongest descriptors.
    let title = match &primary {
        Some(lang) => format!("The {} {} {}", autonomy_adj, lang, tooling_adj),
        None => format!("The {} {}", autonomy_adj, tooling_adj),
    };
    let summary = build_summary(
        tenure_days,
        &primary,
        &autonomy.dominant,
        auto_share,
        time_verdict,
        session_verdict,
        comm_verdict,
        scan.totals.sessions,
    );

    StyleProfile {
        found,
        claude_dir: dir_str,
        tenure_days,
        first_active,
        num_startups: meta.num_startups,
        total_active_days: active_days,
        archetype: Archetype { title, summary },
        traits,
        autonomy,
        rhythm,
        stack,
        tools_top,
        models_top,
        skills,
        plugins,
        communication,
        cache_efficiency,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_summary(
    tenure_days: i64,
    primary: &Option<String>,
    dominant_mode: &str,
    auto_share: f64,
    time_verdict: &str,
    session_verdict: &str,
    comm_verdict: &str,
    sessions: u64,
) -> String {
    let stack_phrase = match primary {
        Some(lang) => format!("mostly in {}", lang),
        None => "across a mix of files".to_string(),
    };
    let lifespan = if tenure_days > 0 {
        format!("Over {} days with Claude Code, ", tenure_days)
    } else {
        String::new()
    };
    format!(
        "{}you've run {} sessions, working {}. You favour {} mode ({:.0}% hands-off), tend to be a {} who runs {} sessions, and communicate as a {}.",
        lifespan,
        sessions,
        stack_phrase,
        dominant_mode,
        auto_share * 100.0,
        time_verdict.to_lowercase(),
        session_verdict.to_lowercase(),
        comm_verdict.to_lowercase(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_against_real_data() {
        let prof = build_profile();
        eprintln!("found={} tenure_days={}", prof.found, prof.tenure_days);
        eprintln!("archetype: {}", prof.archetype.title);
        eprintln!("summary: {}", prof.archetype.summary);
        eprintln!(
            "autonomy: auto={} plan={} default={} accept={} dominant={}",
            prof.autonomy.auto,
            prof.autonomy.plan,
            prof.autonomy.default,
            prof.autonomy.accept_edits,
            prof.autonomy.dominant
        );
        eprintln!("primary stack: {:?}", prof.stack.primary);
        eprintln!(
            "languages: {:?}",
            prof.stack
                .languages
                .iter()
                .map(|l| (l.language.as_str(), l.count))
                .collect::<Vec<_>>()
        );
        eprintln!(
            "comm: prompts={} avg_words={:.1} q={:.2} short={:.2} polite={:.2} vocab={:.3}",
            prof.communication.prompts,
            prof.communication.avg_prompt_words,
            prof.communication.question_ratio,
            prof.communication.short_command_ratio,
            prof.communication.politeness_ratio,
            prof.communication.vocabulary_richness
        );
        eprintln!(
            "rhythm: peak={:?} busiest={} avg_min={:.1}",
            prof.rhythm.peak_hours, prof.rhythm.busiest_weekday, prof.rhythm.avg_session_minutes
        );
        for t in &prof.traits {
            eprintln!("  trait[{}] {} = {} ({})", t.key, t.title, t.value, t.evidence);
        }
        if prof.found {
            assert!(!prof.traits.is_empty());
        }

        // Security: the serialized profile must never carry credential-like data.
        let json = serde_json::to_string(&prof).unwrap();
        for secret_key in [
            "oauthAccount",
            "userID",
            "accessToken",
            "refreshToken",
            "apiKey",
            "customApiKeyResponses",
            "machineId",
        ] {
            assert!(
                !json.contains(secret_key),
                "profile JSON leaked secret field: {secret_key}"
            );
        }
    }
}
