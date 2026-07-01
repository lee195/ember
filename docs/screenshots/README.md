# Screenshots

The top-level `README.md` showcases two synthetic **demo personas** (Dashboard +
Profile each), plus the Portable / Profiles tabs. Recapture with these filenames:

| File | Tab | Launch as persona |
| --- | --- | --- |
| `technical-writer-dashboard.png` | Dashboard | `technical-writer` |
| `technical-writer-profile.png`   | Profile   | `technical-writer` |
| `data-analyst-dashboard.png`     | Dashboard | `data-analyst` |
| `data-analyst-profile.png`       | Profile   | `data-analyst` |
| `portable.png`                   | Portable  | any (UI is persona-agnostic) |
| `profiles.png`                   | Profiles  | any |

Swap in any of the five personas (`frontend-dev`, `data-analyst`,
`technical-writer`, `devops-sre`, `researcher`) — just keep the filename/caption
in the README in sync with the archetype shown.

## Capturing them (macOS)

```bash
# 1. generate synthetic demo homes (fully fake; your real ~/.claude is untouched)
python3 scripts/gen_demo_data.py

# 2. build the app once (normal HOME)
npm run tauri build

# 3. launch ember AS a persona — run the binary DIRECTLY so it inherits HOME
HOME="$PWD/demo-data/technical-writer" \
  ./src-tauri/target/release/bundle/macos/ember.app/Contents/MacOS/ember
```

For each shot: switch to the tab, then **Cmd + Shift + 4 → Space → click the
ember window** (or **Cmd + Shift + 5 → Capture Selected Window**). Save with the
filename above. Relaunch with a different `HOME=…/<persona>` for another persona.

Tips:
- Launch the binary **directly** (not via `open`) or it won't pick up `HOME`.
- The `Portable` / `Profiles` tabs are short — crop off the empty space, e.g.
  `magick profiles.png -crop 2032x620+0+0 +repage profiles.png`.
