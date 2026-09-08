# How ember was built

A chronological account of building ember, including what didn't work the first
time and why it was changed. Commit hashes/dates refer to this repo's `master`
history (`git log`) where applicable; some early work (Phases 1–3) predates
`git init` and landed in one initial commit.

Every phase below did also include normal, uneventful work — this document
deliberately foregrounds the mistakes and fixes, since that's the part that
doesn't show up in the code itself.

---

## 0. Starting point: a broken scaffold

ember started from the default `npm create tauri-app` Tauri 2 + Vue 3 template.

**What went wrong:** the scaffold's `src-tauri/src/lib.rs` had a leftover
`setup()` hook calling `_app.handle().open_url(url, tauri::WindowUrl::App)` —
neither `open_url` (not an `AppHandle` method) nor `WindowUrl` (renamed
`WebviewUrl` in v2) exist in Tauri 2. It wouldn't compile. It was also
redundant: `tauri.conf.json`'s `devUrl` already loads the dev server.
**Fix:** deleted the block outright.

---

## 1. Phase 1 — usage dashboard (`447f145`, 2026-06-26)

Built `src-tauri/src/claude_data.rs` (one `scan_all()` walk of
`~/.claude/projects/*/*.jsonl` → token/model/tool/project aggregates) and the
Dashboard view, verified against an independent Python recount of the same
transcripts.

**What went wrong:** after the backend and frontend were both done, `npm run
tauri dev` hung indefinitely, repeating *"Waiting for your frontend dev server
to start on http://localhost:1420/"*. The scaffold's `tauri.conf.json` had no
`beforeDevCommand`, so Tauri never actually launched Vite itself — it just sat
waiting for something else to bind that port. `vite.config.ts` also didn't pin
a port, so a manually-started Vite could easily land on a different one anyway.
**Fix:** added `"beforeDevCommand": "npm run dev"` / `"beforeBuildCommand": "npm
run build"` to `tauri.conf.json`, and `server: { port: 1420, strictPort: true
}` to `vite.config.ts`.

---

## 2. Phase 2 — personal style profile (`2c666b8` region, 2026-06-29)

Refactored `claude_data.rs` around one shared `Scan` accumulator so both the
dashboard and the new deterministic style profile (`profile.rs`) derive from a
single transcript pass. Added an archetype + trait-card system (autonomy,
rhythm, communication heuristics, primary stack).

**What went wrong:** the first version of the "session length" trait computed
average session duration as `last_timestamp − first_timestamp` per
`sessionId`. Claude Code session IDs persist across resumes that can span days
of wall-clock time (close the laptop, come back tomorrow, same session
continues) — so the computed average came out as **4,514 minutes (≈75 hours)**
per session. This was caught only by actually reading the smoke test's printed
output rather than trusting a green test run.
**Fix:** switched to an active-time approximation — sum only the gaps *between
consecutive messages* that are ≤30 minutes, treating longer gaps as "away, not
in session." The average dropped to a plausible ~52 minutes.
**Lesson:** a passing test only proves the code ran; sanity-check the actual
numbers it produced against intuition, not just "no panic."

---

## 3. The Deno detour — `ember-deno` (sibling directory, not this repo)

As an experiment, the same app was rebuilt on Deno 2.9's `deno desktop`
command instead of Tauri, reusing the Vue frontend and porting the Rust data
layer to TypeScript. It lives in `../ember-deno` with its own
[`COMPARISON.md`](../../ember-deno/COMPARISON.md) — not part of this repo's
history — but the mistakes made along the way are worth recording here since
they shaped how ember's own Tauri code is written and reviewed.

**What went wrong (repeatedly):**

- **A confidently wrong claim.** When first asked to use `deno desktop`, the
  assumption was that Deno couldn't build native GUI apps at all (based on
  `deno compile --help` alone), and three workaround architectures were
  proposed instead of the thing actually asked for. The user pushed back:
  *"no, deno now has a desktop command. check the release notes, not just the
  cli manual."* They were right — `deno desktop` is a real (if
  experimental) subcommand in Deno 2.9, just hidden from the top-level
  `--help` summary. **Lesson:** a knowledge-cutoff gap plus a shallow check
  (`--help` alone) produced a confident negative claim that was simply wrong.
  For anything version/release-dependent, check the actual command, not just
  the summary help.
- **`dist/` silently missing from the bundle.** The first compiled `.app`
  showed nothing but "Not Found." `deno desktop` doesn't auto-embed arbitrary
  directories — it needed an explicit `--include dist` flag, and the server
  code had to resolve files via `new URL("./dist/...", import.meta.url)`
  rather than a plain filesystem path, since embedded files live inside the
  compiled binary's virtual filesystem, not on disk.
- **Wrong port.** The app's own HTTP server picked a random free port instead
  of reading the runtime-provided `DENO_SERVE_ADDRESS` (`tcp:127.0.0.1:<port>`)
  that the window was actually wired to navigate to.
- **Invalid app name.** `"name": "Ember (Deno)"` in `deno.json` failed the
  build with `invalid app name: must match [A-Za-z0-9 ._-]+` — parentheses
  aren't allowed. Renamed to `"Ember Deno"`.
- **A binding race condition.** Once the UI loaded, every backend call failed
  with `No callback bound for: getOverview`. The code called `Deno.serve(...)`
  *before* registering `win.bind(...)` handlers — the webview (a separate
  process) started fetching and calling into bindings before they existed.
  Fixed by moving all `win.bind(...)` calls before `Deno.serve(...)` starts.
- **Closing the window didn't quit the app.** The process lingered after the
  red traffic-light was clicked, because the running HTTP server counted as a
  "live async task" and `deno desktop` only auto-exits when there are none.
  Fixed with an explicit `win.addEventListener("close", () => Deno.exit(0))`.

None of this reflects badly on `deno desktop` itself — it worked, and the
comparison doc it produced (Tauri vs. `deno desktop`, when to use which) is a
genuinely useful artifact. But it's a good example of how much of "building
with a new/experimental tool" is discovering undocumented sharp edges one
build-and-fail cycle at a time.

---

## 4. Phase 3 — portable profile (`447f145`, part of the initial commit)

`portable.rs` (`export_profile`/`import_profile`) — a secret-excluding bundle
of `skills/`, `settings.json`, `CLAUDE.md`, and the style profile, with a
dry-run-first import. Logic was actually designed and verified once already
(in `ember-deno`'s `backend/portable.ts`) before being ported back to Rust,
which meant this phase shipped with no notable bugs — a rare case of the
Deno detour directly paying for itself.

---

## 5. LLM narrative feature (`2c666b8`, 2026-06-29)

**A design correction before any code was written:** the initial plan asked
how to store an *Anthropic* API key for a Claude-powered narrative. The user
redirected this: *"to keep data local, a flexible setup that allows ex ollama
or omlx or other local llm providers instead of only claude would be better."*
The feature was redesigned around one OpenAI-compatible HTTP adapter (base URL
+ model + optional key) that works identically with Ollama, oMLX, LM Studio,
llama.cpp, or any cloud OpenAI-compatible endpoint — no Claude-specific code
path at all. This preference was significant enough to record in persistent
memory so future work on the app doesn't default back to a hardcoded cloud API.

**What went wrong after that:**

- **Unusable first output.** The first real generation (against local Ollama)
  returned the model's raw chain-of-thought — *"Here's a thinking process: 1.
  Analyze User Input..."* — and cut off mid-draft. Two compounding causes:
  the system prompt never told the model to suppress reasoning/preamble, and
  `max_tokens: 1024` truncated the response before it ever reached a finished
  answer. **Fix:** rewrote the prompt to forbid preamble/analysis and require
  the answer wrapped in `<profile>…</profile>` tags, added a `clean_narrative`
  post-processor that strips `<think>`/`<thinking>` blocks and extracts just
  the tagged body (tolerating a missing closing tag from truncation), and
  raised `max_tokens` to 4096.
- **Repeated Keychain prompts, even for local-only use.** `get_llm_settings` —
  called every time the Profile tab opened — computed `has_api_key` by hitting
  the macOS Keychain live on every call, regardless of whether a key was ever
  set. **Fix:** `has_api_key` became a plain boolean in the non-secret config
  file, set only when a key is actually saved. Now the Keychain is touched
  only on an explicit "save key" action or when generating with a saved key —
  never for local providers.

---

## 6. Profile management — `CLAUDE_CONFIG_DIR` profiles (`17ea06e`, 2026-06-29)

Before building, feasibility was verified against actual Claude Code docs
(rather than assumed): `CLAUDE_CONFIG_DIR` relocates the entire user config,
multiple `claude` processes can run fully isolated side-by-side, and on macOS
auth lives in the Keychain (not the config dir), so profiles share login with
no re-onboarding. `profiles.rs` + a Terminal-per-profile launcher (`osascript`)
followed from there.

**What went wrong:** "Import bundle" (and Clone, and Rename) silently did
nothing after picking a folder — no error, no dialog. The cause: Tauri's macOS
webview (WKWebView) doesn't implement `window.prompt()` — it just returns
`null` instantly. `window.confirm()` *is* implemented, which is why Delete's
confirmation dialog worked fine and made the broken one less obvious at first.
**Fix:** built an in-app `NamePrompt.vue` modal and replaced every `prompt()`
call with it.

---

## 7. A CI workflow that was built, then removed

A CI workflow was written assuming a macOS runner, since producing ember's
actual bundle requires one (the backend needs `keyring`'s `apple-native`
feature and an `osascript` launcher — neither works on Linux). Once it turned
out only Linux runners were available, it was rewritten to a Linux-friendly
frontend-only check (`npm ci && npm run build`) instead. The user then judged
that a workflow which can't produce the real build artifact wasn't worth
having yet, so it was deleted entirely, workflow directory included. Not a
bug so much as scoping a feature down to zero once its actual value was
reconsidered — worth recording so a future "let's add CI" doesn't repeat the
same macOS-runner assumption unprompted.

---

## 8. App icon (`72b20b9`, 2026-06-30)

Iterative design, not a technical build — static flame marks → a coal mascot →
several orbit-mascot riffs → the final `ember-mascot-orbit-astronaut.svg`, with
every intermediate draft kept in `design/icons/` and documented in
[`ICON.md`](../design/icons/ICON.md). No wrong turns here worth flagging;
included for completeness of the timeline.

---

## 9. README, screenshots, and synthetic demo data (`e1f66e3` → `75e5511` → `bb947b1`, 2026-06-30 to 07-01)

**What went wrong (three separate issues, same root theme — demo-data
hygiene wasn't thought through up front):**

- **Screenshots initially used real personal usage data.** The first README
  screenshots came straight from the developer's live `~/.claude` — real
  project names, real session counts, a real inferred working-style
  archetype. Functionally fine, but not something that should sit in a repo
  pushed to a shared remote. This was only reconsidered after the fact (see
  §10) rather than avoided from the start.
- **The demo-data generator was accidentally software-development-only, and
  it broke ember's own "vocabulary" metric.** The first synthetic personas
  were all coding personas drawing from a tiny fixed pool of ~10–25 prompt
  phrases repeated hundreds of times, which made the Profile tab's
  "vocabulary richness" stat read as ≈0% — a *correct* reflection of how
  repetitive the fake data was, but a bad look for a demo, and it also meant
  the generated personas couldn't show that ember works for non-coding users
  at all. The `ext_to_language` map in `profile.rs` only recognized code file
  extensions, so every synthetic persona showed up as a "Builder" with a
  programming-language stack no matter what. **Fix:** rewrote the persona set
  to span five real domains (frontend dev, data analysis, technical writing,
  DevOps/SRE, academic research) with much larger domain-flavored vocabulary,
  and — more importantly — extended `ext_to_language` to recognize Jupyter
  notebooks, Terraform, LaTeX, R, CSV/data files, GraphQL, Protobuf, XML, and
  plain text. That second part was a genuine, permanent improvement to
  shipped code: real non-developer users' file activity was previously
  invisible to the stack-detection logic entirely.
- **The generator wasn't idempotent, which caused a real mislabeling bug.**
  `write_persona()` appended new synthetic sessions into each persona's
  directory on every run without clearing old ones, so re-running the
  generator silently accumulated sessions across runs. This directly caused
  screenshots labeled `researcher-*.png` to actually show the `data-analyst`
  persona's data ("The Methodical Jupyter Builder" instead of "The Methodical
  LaTeX Explorer") — caught only by re-opening the images and cross-checking
  the archetype against what was expected, then renaming the files to match
  their true content. **Fix:** `write_persona()` now deletes the persona's
  directory before regenerating it (`shutil.rmtree(home, ignore_errors=True)`),
  making runs reproducible.

---

## 10. Purging real-usage screenshots from git history (2026-07-01/02)

Once the real-data screenshots (§9) had been replaced with synthetic ones in
the working tree, the old image blobs still existed in already-pushed commit
history. Removing them required an actual history rewrite — git commit hashes
are content-addressed, so "delete a file from the past" means recreating every
commit from that point forward with new hashes, not just deleting it in a new
commit. `git filter-branch --index-filter` was used (the interactive `git
rebase -i` route that was discussed first can't run without a terminal in this
environment) to drop `dashboard.png`/`profile.png` from all history, after
creating a `backup-pre-purge` branch as a safety net. The force-push was
deliberately left for the user to review and run themselves, since rewriting
already-pushed shared history is exactly the kind of action that shouldn't
happen without an explicit go-ahead.
**Lesson:** decide what's safe to commit *before* the first commit, not after
it's already on the remote — the fix is the same either way, but doing it
after requires a history rewrite instead of just not doing the thing.

---

## Smaller adjustments (preference, not bugs)

A few changes were pure iteration/preference rather than something being
wrong: the narrative provider preset list was trimmed from four entries down
to Ollama / oMLX / OpenAI-compatible cloud once the user clarified oMLX was
what they actually used; the collapsible-section chevron icon was enlarged and
recolored after it read as too small at a glance; the Narrative section on the
Profile tab was made collapsible (and defaults to collapsed) to reduce visual
weight for people not using it.

## Patterns worth remembering

A few root causes recurred across otherwise-unrelated features:

- **Verify webview capabilities before relying on them.** Both `window.prompt`
  (Tauri/WKWebView) and, earlier, the very existence of `deno desktop` were
  wrong assumptions caught only by hitting the failure in practice. When a
  browser/webview API is pivotal to a flow, check it's actually implemented in
  the target runtime before building the flow around it.
- **Local-first has a default-secure gravity of its own.** Both the Keychain
  over-prompting and the original Anthropic-only narrative plan trended toward
  "touch the sensitive resource by default" until corrected toward "touch it
  only on an explicit, minimal action." Worth designing for from the start
  next time, not fixing after the fact.
- **Generators need idempotency as much as production code does.** The demo
  data generator's accumulation bug was a "throwaway script" problem that
  nonetheless produced a real, published mistake (mislabeled screenshots).
