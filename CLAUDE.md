# CLAUDE.md — working in the ember repo

ember is a local-first macOS desktop app (Tauri 2 + Vue 3) that turns a person's
Claude Code usage (`~/.claude`) into insights + a portable profile, and lets them
run multiple Claude Code configs side-by-side.

## First: who are you helping?

Before giving detailed guidance, figure out whether the person is a **User** or a
**Developer**. If it isn't already clear from their request, ask:

> Are you here to **use ember** (run the app and its features), or to **work on
> it** (understand or change the code)?

Then follow the matching section below. If their intent is obvious (e.g. they ask
about a Rust module, or about "how do I export my profile"), skip the question and
go straight to the right section.

---

## 👤 For users — using the app

**What it does:** reads your own Claude Code data locally and shows it back to you.
Nothing is uploaded; your real `~/.claude` is only ever read, never modified by the
insights features.

**Run it (needs macOS; to build, Node 20+ and Rust):**
```bash
npm install
npm run tauri dev        # dev window
# or a distributable app:
npm run tauri build      # → src-tauri/target/release/bundle/macos/ember.app
```

**The four tabs:**
- **Dashboard** — usage insights: tokens (in/out/cache), models, activity by hour,
  per-project breakdown, tool usage.
- **Profile** — a "how you work" archetype + trait cards derived from your data,
  with local communication heuristics. Optional **Narrative**: an LLM-written prose
  version. In the Narrative settings, pick a provider — a **local** model (Ollama,
  oMLX, LM Studio, llama.cpp) or any OpenAI-compatible endpoint — by base URL +
  model. Local endpoints keep everything on your machine; a cloud key (if used) is
  stored in the macOS Keychain, never on disk.
- **Portable** — export a re-importable bundle of your setup (style profile,
  `skills/`, `settings.json`, `CLAUDE.md`) with **credentials excluded**, or apply
  a bundle to this machine (dry-run first).
- **Profiles** — manage multiple isolated Claude Code configs (each its own
  `CLAUDE_CONFIG_DIR`) and **launch one — or several side-by-side** — in a Terminal,
  without touching your real `~/.claude`. Create profiles blank / by cloning / from a
  bundle, and edit model, permission mode, enabled plugins, and `CLAUDE.md` in-app.

**Privacy:** credentials are never read or exported (`.claude.json`,
`.credentials.json` are excluded everywhere); profile management never writes to
`~/.claude`.

**Just want to see it with example data?** Generate synthetic personas and launch
against one (your real data is untouched):
```bash
python3 scripts/gen_demo_data.py
HOME="$PWD/demo-data/data-analyst" \
  ./src-tauri/target/release/bundle/macos/ember.app/Contents/MacOS/ember
```

---

## 🛠️ For developers — how it's built

**Stack:** [Tauri 2](https://tauri.app) (Rust backend) + Vue 3 + Vite + TypeScript.
Charts are hand-rolled SVG (no chart dependency). **macOS-only** (the backend uses
`keyring`'s `apple-native` feature and an `osascript` launcher).

**Shape:** the Vue frontend calls Rust `#[tauri::command]`s through `invoke()`
(wrapped in `src/lib/api.ts`); the backend reads local data and returns typed DTOs.
DTOs use **snake_case** on both sides so the TS interfaces mirror the Rust structs.

**Backend — `src-tauri/src/`:**
| File | Responsibility | Commands |
|---|---|---|
| `claude_data.rs` | one `scan_all()` walk of `~/.claude/projects/*/*.jsonl`; usage aggregation | `get_overview` |
| `profile.rs` | deterministic style profile from the `Scan` + whitelisted `~/.claude.json`; ext→language map, archetype/traits | `get_profile` |
| `narrative.rs` | OpenAI-compatible LLM narrative (`reqwest`); settings in app-config dir; API key in Keychain | `get/set_llm_settings`, `generate_narrative`, `get_cached_narrative` |
| `portable.rs` | bundle export/import (`ASSETS`, `copy_tree`, secret exclusion) | `export_profile`, `import_profile` |
| `profiles.rs` | multiple `CLAUDE_CONFIG_DIR` profiles + Terminal launch via `osascript` | `list/create/rename/delete_profile`, `get/set_profile_config`, `launch_profiles`, … |
| `lib.rs` | Tauri builder, plugins (shell, dialog), command registration | — |

**Frontend — `src/`:** `App.vue` (tab shell) → `views/{Dashboard,Profile,Portable,Profiles}View.vue`
→ reusable `components/` → `lib/api.ts` (invoke wrappers + types).

**Data location:** everything is found via `dirs::home_dir()` → `~/.claude`. Overriding
`$HOME` when launching the built binary redirects all reads — that's how the demo
personas (`scripts/gen_demo_data.py`) work without touching real data.

**Common commands:**
```bash
npm run tauri dev                 # run the app (frontend + Rust)
npm run build                     # vue-tsc type-check + vite build (frontend only)
cd src-tauri && cargo test        # Rust unit tests (per-module; some read ~/.claude or temp dirs)
cd src-tauri && cargo check       # fast compile check
```

**Adding a backend command:** write it in the relevant module → register it in the
`generate_handler!` list in `lib.rs` → add a wrapper + types in `src/lib/api.ts` →
call it from a view/component.

**Security invariants (keep these true):** never read, surface, or export
credential-like fields (`oauthAccount`, `userID`, API keys, `.credentials.json`);
`~/.claude` is read-only for the insights/profile features; profile management writes
only under the app-data dir.

**Also in the repo:** `design/icons/ICON.md` (app icon + how it was made),
`scripts/gen_demo_data.py` (synthetic demo personas), `docs/screenshots/` (README
shots + capture guide).
