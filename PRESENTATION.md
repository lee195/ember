# Project Ember — Technical Presentation

A comprehensive, slide-by-slide technical presentation for **Project Ember** is available in [`docs/TECHNICAL_PRESENTATION.md`](file:///c:/Users/jisu_/Desktop/workspace/ember/docs/TECHNICAL_PRESENTATION.md).

---

## 📽️ How to View & Present

The presentation is authored in **Marp-compatible Markdown** with GitHub Flavored Markdown (GFM) compatibility, complete with Mermaid architecture diagrams, tables, mathematical formulations, and speaker notes.

### Options to view or present:

1. **GitHub / Local Markdown Preview:**
   - Open [`docs/TECHNICAL_PRESENTATION.md`](./TECHNICAL_PRESENTATION.md) directly in any standard Markdown viewer, IDE preview, or GitHub web interface.
   - Each slide is cleanly partitioned by `---` dividers, with collapsed/annotated speaker notes in `<!-- Speaker Notes: ... -->` blocks.

2. **VS Code / Cursor / Antigravity with Marp Extension:**
   - Install the **Marp for VS Code** extension (`marp-team.marp-vscode`).
   - Open [`docs/TECHNICAL_PRESENTATION.md`](./TECHNICAL_PRESENTATION.md).
   - Click the **Marp Slide Deck Preview** icon in the editor toolbar, or press `Ctrl+K, V` / `Cmd+K, V` to view interactive slides.
   - Export to PDF, HTML, or PPTX with `Marp: Export Slide Deck`.

3. **Marp CLI (Standalone / Terminal):**

   ```bash
   npx @marp-team/marp-cli docs/TECHNICAL_PRESENTATION.md -o presentation.html
   # Or directly launch Marp presenter view:
   npx @marp-team/marp-cli -s docs/TECHNICAL_PRESENTATION.md
   ```

4. **Obsidian / Slidev / Reveal.js:**
   - The document conforms to standard slide delimiter syntax (`---`), making it drop-in compatible with Obsidian Advanced Slides, Slidev, and reveal-md.

---

## 📑 Slide Deck Outline & Agenda

| Slide # | Title                                       | Core Focus                                                                       |
| :-----: | :------------------------------------------ | :------------------------------------------------------------------------------- |
| **01**  | **Title & Executive Overview**              | High-level introduction to Project Ember, stack, and mission                     |
| **02**  | **The Problem Space**                       | Telemetry opacity, lack of portability, and config drift in AI CLI tools         |
| **03**  | **System Architecture Overview**            | Tauri 2 + Rust + Vue 3 process model, IPC routing, OS boundaries                 |
| **04**  | **Fast Data Ingestion Engine**              | Single-pass streaming transcript parsing (`claude_data.rs`)                      |
| **05**  | **Style Profiling & Heuristics**            | Deterministic profiling, the 30-min idle threshold, median vs. mean              |
| **06**  | **Archetype Taxonomy & Domain Detection**   | Autonomy × Domain × Tooling matrix (e.g. _The Methodical Markdown Explorer_)     |
| **07**  | **Security Invariants & Privacy**           | Zero-network core, `.claude.json` whitelist, and credential quarantine           |
| **08**  | **Portable Profile Bundles**                | Cross-machine synchronization (`portable.rs`) & safe dry-run imports             |
| **09**  | **Multi-Config Sandboxing**                 | `CLAUDE_CONFIG_DIR` isolation, config editing, and AppleScript Terminal launcher |
| **10**  | **Local-First LLM Narrative**               | Provider-agnostic OpenAI adapter (Ollama, oMLX) & defensive CoT filtering        |
| **11**  | **macOS System Integration & Keychain**     | Apple-native Keychain security, lazy evaluation, eliminating OS prompt spam      |
| **12**  | **Frontend Architecture & UI Design**       | Vue 3 component tree, hand-rolled SVG micro-charts (zero chart bloat)            |
| **13**  | **Engineering Case Studies & Post-Mortems** | Lessons learned from `HISTORY.md` (Deno detour, WKWebView prompt bug, etc.)      |
| **14**  | **Performance & Resource Utilization**      | Tauri 2 vs. Electron benchmarks (RAM, bundle size, cold boot, safety)            |
| **15**  | **Developer Workflow & Persona Simulation** | Headless testing via `$HOME` override and synthetic persona generation           |
| **16**  | **Future Horizons & Roadmap**               | Cross-platform PTYs, MCP server management, and team rule packs                  |
| **17**  | **Summary & Architectural Principles**      | Local-first, determinism over hallucination, safe sandboxing                     |
| **18**  | **Q&A & Discussion Guide**                  | Wrap-up, repository references, and open discussion                              |

---

_Authored for Project Ember — Local-first macOS app for Claude Code usage insights & configuration sandboxing._
