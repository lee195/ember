import { invoke } from "@tauri-apps/api/core";

// Mirrors the Rust DTOs in src-tauri/src/claude_data.rs

export interface Totals {
  sessions: number;
  projects: number;
  user_messages: number;
  assistant_messages: number;
  input_tokens: number;
  output_tokens: number;
  cache_creation_tokens: number;
  cache_read_tokens: number;
  tool_calls: number;
  web_searches: number;
  web_fetches: number;
}

export interface DailyPoint {
  date: string;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  messages: number;
}

export interface ModelUsage {
  model: string;
  messages: number;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
}

export interface ProjectSummary {
  path: string;
  name: string;
  sessions: number;
  messages: number;
  input_tokens: number;
  output_tokens: number;
  last_active: string | null;
}

export interface ToolUsage {
  name: string;
  count: number;
}

export interface Overview {
  claude_dir: string;
  found: boolean;
  totals: Totals;
  daily: DailyPoint[];
  hourly: number[];
  models: ModelUsage[];
  projects: ProjectSummary[];
  tools: ToolUsage[];
  date_range: [string, string] | null;
}

export function getOverview(): Promise<Overview> {
  return invoke<Overview>("get_overview");
}

// ---------- Phase 2: personal style profile ----------

export interface Trait {
  key: string;
  title: string;
  value: string;
  detail: string;
  evidence: string;
}

export interface Autonomy {
  plan: number;
  auto: number;
  default: number;
  accept_edits: number;
  dominant: string;
}

export interface Rhythm {
  peak_hours: number[];
  busiest_weekday: string;
  avg_session_minutes: number;
  sessions_per_active_day: number;
  active_days: number;
}

export interface LangCount {
  language: string;
  count: number;
}

export interface Stack {
  languages: LangCount[];
  primary: string | null;
}

export interface NamedUsage {
  name: string;
  usage_count: number;
  last_used: string | null;
}

export interface Communication {
  prompts: number;
  avg_prompt_words: number;
  question_ratio: number;
  short_command_ratio: number;
  politeness_ratio: number;
  specificity_ratio: number;
  vocabulary_richness: number;
}

export interface StyleProfile {
  found: boolean;
  claude_dir: string;
  tenure_days: number;
  first_active: string | null;
  num_startups: number;
  total_active_days: number;
  archetype: { title: string; summary: string };
  traits: Trait[];
  autonomy: Autonomy;
  rhythm: Rhythm;
  stack: Stack;
  tools_top: ToolUsage[];
  models_top: ModelUsage[];
  skills: NamedUsage[];
  plugins: NamedUsage[];
  communication: Communication;
  cache_efficiency: number;
}

export function getProfile(): Promise<StyleProfile> {
  return invoke<StyleProfile>("get_profile");
}

// ---------- Phase 3: portable profile ----------

export interface ExportResult {
  bundle_dir: string;
  included: string[];
  skipped: string[];
}

export interface ImportPlan {
  source_dir: string;
  applied: boolean;
  actions: string[];
}

export function exportProfile(destDir: string): Promise<ExportResult> {
  return invoke<ExportResult>("export_profile", { destDir });
}

export function importProfile(
  srcDir: string,
  apply: boolean,
): Promise<ImportPlan> {
  return invoke<ImportPlan>("import_profile", { srcDir, apply });
}

/** Open a native folder picker; returns the chosen path or null if cancelled. */
export async function pickFolder(title: string): Promise<string | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const res = await open({ directory: true, multiple: false, title });
  return typeof res === "string" ? res : null;
}

export function formatPercent(ratio: number): string {
  return Math.round(ratio * 100) + "%";
}

// ---------- formatting helpers ----------

export function formatNumber(n: number): string {
  if (n >= 1_000_000_000) return (n / 1_000_000_000).toFixed(2) + "B";
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + "M";
  if (n >= 1_000) return (n / 1_000).toFixed(1) + "K";
  return String(n);
}

export function formatDate(iso: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (isNaN(d.getTime())) return iso;
  return d.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}
