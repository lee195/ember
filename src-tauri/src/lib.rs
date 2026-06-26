mod claude_data;
mod portable;
mod profile;

use claude_data::Overview;
use portable::{ExportResult, ImportPlan};
use profile::StyleProfile;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_overview,
            get_profile,
            export_profile,
            import_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri process");
}

/// Parse and aggregate all local Claude Code usage data into a single payload
/// for the insights dashboard. Runs on a blocking thread since it does file I/O.
#[tauri::command]
async fn get_overview() -> Result<Overview, String> {
    tauri::async_runtime::spawn_blocking(claude_data::build_overview)
        .await
        .map_err(|e| e.to_string())
}

/// Derive the deterministic "personal style" profile from local usage data.
#[tauri::command]
async fn get_profile() -> Result<StyleProfile, String> {
    tauri::async_runtime::spawn_blocking(profile::build_profile)
        .await
        .map_err(|e| e.to_string())
}

/// Write a portable, secret-free profile bundle to `dest_dir`.
#[tauri::command]
async fn export_profile(dest_dir: String) -> Result<ExportResult, String> {
    tauri::async_runtime::spawn_blocking(move || portable::export_profile(dest_dir))
        .await
        .map_err(|e| e.to_string())?
}

/// Apply a profile bundle from `src_dir` to ~/.claude (dry run unless `apply`).
#[tauri::command]
async fn import_profile(src_dir: String, apply: bool) -> Result<ImportPlan, String> {
    tauri::async_runtime::spawn_blocking(move || portable::import_profile(src_dir, apply))
        .await
        .map_err(|e| e.to_string())?
}
