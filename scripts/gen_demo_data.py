#!/usr/bin/env python3
"""Generate synthetic ~/.claude data for ember screenshots.

Writes one self-contained fake HOME per persona under an output dir, each with
`.claude/projects/<enc>/<uuid>.jsonl` transcripts + `.claude.json`. Launch ember
against one with a HOME override to screenshot different usage / profiles:

    python3 scripts/gen_demo_data.py
    HOME="$PWD/demo-data/data-analyst" \
      ./src-tauri/target/release/bundle/macos/ember.app/Contents/MacOS/ember

Personas span several domains (not just coding). Nothing here touches your real
~/.claude — fully synthetic, no credentials.
"""

import json
import os
import random
import shutil
import sys
import uuid
from datetime import datetime, timedelta, timezone

BASE = datetime(2026, 7, 1, tzinfo=timezone.utc)  # fixed "now" for determinism

MODELS = {
    "opus": "claude-opus-4-8", "opus7": "claude-opus-4-7",
    "sonnet": "claude-sonnet-5", "haiku": "claude-haiku-4-5-20251001",
}

PROMPTS = {
    "frontend": [
        "extract this into a reusable component and thread the props through",
        "the layout breaks on narrow viewports — make the grid collapse gracefully",
        "wire the form state to the store and validate on blur",
        "add an empty state and a loading skeleton to this list",
        "why does the modal re-render every keystroke? memoize what you can",
        "tidy up the styles and pull the shared tokens into variables",
    ],
    "data": [
        "join the orders table to customers and compute monthly retention cohorts",
        "the regression residuals look skewed — try a log transform and re-fit",
        "aggregate revenue by segment and plot the trend with a rolling average",
        "clean the nulls in the survey export and dedupe on respondent id",
        "run an ab-test power analysis for a two percent lift at ninety-five percent",
        "profile this query, it scans the whole table — add the right index",
        "summarize the churn drivers from the model coefficients in plain english",
    ],
    "writing": [
        "restructure this section so the key takeaway comes first, then the detail",
        "tighten the prose here and cut the passive voice without losing nuance",
        "draft a getting-started guide that assumes no prior context",
        "add a worked example and a short glossary for the unfamiliar terms",
        "make the tone warmer and more direct, but keep it precise",
        "reconcile the terminology across these three pages so it's consistent",
        "write release notes from this changelog grouped by theme",
    ],
    "devops": [
        "the rollout is stuck — check the readiness probe and the resource limits",
        "parameterize the terraform module and move the secrets to the vault backend",
        "add autoscaling to the service and a sane pod disruption budget",
        "the pipeline is flaky on the cache step, make it resilient and idempotent",
        "trace why the healthcheck flaps under load and add a backoff",
        "split the monolithic manifest into per-environment overlays",
    ],
    "research": [
        "summarize the last five papers on this method and where they disagree",
        "critique the methodology in this study — what confounds the result?",
        "outline a literature review comparing the three dominant approaches",
        "restate the hypothesis precisely and list the assumptions it depends on",
        "find recent citations that challenge the baseline and note their evidence",
        "draft the related-work section and position our contribution against it",
    ],
}
TOKENS = {
    "frontend": ["viewport", "reducer", "memoize", "props", "hydration", "accessibility",
                 "flexbox", "debounce", "suspense", "portal", "skeleton", "tokens"],
    "data": ["cohort", "retention", "residuals", "quantile", "dataframe", "funnel",
             "p-value", "seasonality", "outliers", "regression", "segment", "index"],
    "writing": ["voice", "cadence", "glossary", "callout", "heading", "changelog",
                "terminology", "onboarding", "narrative", "tone", "example", "clarity"],
    "devops": ["rollout", "probe", "autoscale", "terraform", "manifest", "backoff",
               "idempotent", "vault", "canary", "throughput", "overlay", "quota"],
    "research": ["hypothesis", "confound", "citation", "baseline", "methodology",
                 "corpus", "ablation", "priors", "replication", "significance", "abstract"],
}
TERSE = ["ship it", "re-run", "revert that", "looks good", "try again",
         "smaller diff", "add a test", "explain this", "keep going", "undo"]


def weighted(pairs):
    vals, w = zip(*pairs)
    return random.choices(vals, weights=w, k=1)[0]


def prompt_for(p):
    style = random.choice(p["styles"])
    if style == "terse":
        return random.choice(TERSE)
    base = random.choice(PROMPTS[p["domain"]])
    if style == "polite":
        base = random.choice(["please ", "could you ", "thanks — could you "]) + base
    toks = random.sample(TOKENS[p["domain"]], k=random.randint(0, 2))
    return base + "".join(f" `{t}`" for t in toks)


PERSONAS = [
    {
        "slug": "frontend-dev", "domain": "frontend",
        "tenure_days": 95, "startups": 120, "sessions": 34,
        "projects": ["/Users/dev/web-app", "/Users/dev/design-system", "/Users/dev/marketing-site"],
        "hours": [9, 10, 10, 11, 13, 14, 15, 16], "weekend": False,
        "modes": [("auto", 7), ("acceptEdits", 2), ("plan", 2), ("default", 1)],
        "exts": [("tsx", 5), ("ts", 3), ("vue", 2), ("css", 3), ("md", 1)],
        "tool_edit_bias": 0.66, "models": [("opus", 6), ("sonnet", 4)],
        "styles": ["convo", "convo", "terse"], "turns": (8, 15), "web": 0,
        "skills": {"simplify": 8, "code-review": 4, "update-config": 2},
    },
    {
        "slug": "data-analyst", "domain": "data",
        "tenure_days": 140, "startups": 90, "sessions": 30,
        "projects": ["/Users/ana/revenue-analysis", "/Users/ana/churn-model", "/Users/ana/ab-tests"],
        "hours": [9, 10, 11, 11, 13, 14, 15], "weekend": False,
        "modes": [("default", 4), ("plan", 4), ("auto", 2)],
        "exts": [("py", 4), ("ipynb", 5), ("sql", 4), ("csv", 3), ("md", 2)],
        "tool_edit_bias": 0.5, "models": [("opus", 5), ("sonnet", 5)],
        "styles": ["convo", "convo"], "turns": (7, 14), "web": 0,
        "skills": {"deep-research": 4, "code-review": 3},
    },
    {
        "slug": "technical-writer", "domain": "writing",
        "tenure_days": 120, "startups": 70, "sessions": 26,
        "projects": ["/Users/pat/product-docs", "/Users/pat/api-reference", "/Users/pat/handbook"],
        "hours": [8, 9, 10, 11, 13, 14, 16], "weekend": False,
        "modes": [("plan", 5), ("default", 4), ("acceptEdits", 1)],
        "exts": [("md", 7), ("mdx", 3), ("txt", 2), ("json", 1)],
        "tool_edit_bias": 0.33, "models": [("opus", 4), ("sonnet", 6)],
        "styles": ["detailed", "polite", "polite"], "turns": (6, 12), "web": 0,
        "skills": {"simplify": 6, "update-config": 2},
    },
    {
        "slug": "devops-sre", "domain": "devops",
        "tenure_days": 200, "startups": 160, "sessions": 36,
        "projects": ["/Users/sre/infra", "/Users/sre/k8s-manifests", "/Users/sre/ci-pipelines"],
        "hours": [8, 9, 12, 17, 20, 22, 2, 3], "weekend": "any",  # on-call
        "modes": [("acceptEdits", 4), ("auto", 4), ("plan", 2), ("default", 2)],
        "exts": [("tf", 5), ("yaml", 4), ("sh", 3), ("toml", 1), ("md", 1)],
        "tool_edit_bias": 0.55, "models": [("opus", 5), ("sonnet", 3), ("haiku", 2)],
        "styles": ["terse", "convo", "terse"], "turns": (5, 11), "web": 0,
        "skills": {"loop": 5, "update-config": 4, "code-review": 2},
    },
    {
        "slug": "researcher", "domain": "research",
        "tenure_days": 170, "startups": 80, "sessions": 24,
        "projects": ["/Users/lee/literature-review", "/Users/lee/grant-proposal", "/Users/lee/experiments"],
        "hours": [11, 14, 16, 20, 21, 22], "weekend": "any",
        "modes": [("plan", 6), ("default", 3), ("auto", 1)],
        "exts": [("tex", 5), ("md", 4), ("bib", 2), ("py", 2), ("csv", 1)],
        "tool_edit_bias": 0.3, "models": [("opus", 5), ("sonnet", 5)],
        "styles": ["detailed", "detailed", "polite"], "turns": (7, 13), "web": 3,
        "skills": {"deep-research": 9, "stride-analysis": 1},
    },
]


def enc(cwd):
    return cwd.replace("/", "-")


def rand_day(p):
    for _ in range(50):
        d = BASE - timedelta(days=random.randint(0, p["tenure_days"] - 1))
        we = d.weekday() >= 5
        if p["weekend"] == "any" or p["weekend"] == we:
            return d
    return BASE


def gen_session(p):
    cwd = random.choice(p["projects"])
    day = rand_day(p)
    t = day.replace(hour=random.choice(p["hours"]), minute=random.randint(0, 55),
                    second=0, microsecond=0)
    sid = str(uuid.uuid4())
    lines = []
    for _ in range(random.randint(*p["turns"])):
        lines.append({
            "type": "user", "timestamp": t.isoformat().replace("+00:00", "Z"),
            "cwd": cwd, "sessionId": sid, "permissionMode": weighted(p["modes"]),
            "isMeta": False, "message": {"role": "user", "content": prompt_for(p)},
        })
        t += timedelta(minutes=random.randint(1, 6))
        content = [{"type": "thinking", "thinking": "..."}]
        for _ in range(random.randint(1, 3)):
            name = random.choice(["Edit", "Write"]) if random.random() < p["tool_edit_bias"] \
                else random.choices(["Read", "Grep", "Bash"], weights=[5, 3, 2])[0]
            item = {"type": "tool_use", "name": name}
            if name in ("Edit", "Write", "Read"):
                ext = weighted(p["exts"])
                item["input"] = {"file_path": f"{cwd}/file_{random.randint(1,40)}.{ext}"}
            content.append(item)
        lines.append({
            "type": "assistant", "timestamp": t.isoformat().replace("+00:00", "Z"),
            "cwd": cwd, "sessionId": sid,
            "message": {"role": "assistant", "model": MODELS[weighted(p["models"])],
                "usage": {"input_tokens": random.randint(400, 2600),
                          "output_tokens": random.randint(800, 7000),
                          "cache_creation_input_tokens": random.randint(2000, 40000),
                          "cache_read_input_tokens": random.randint(40000, 600000),
                          "server_tool_use": {"web_search_requests": random.randint(0, p["web"]) if p["web"] else 0,
                                              "web_fetch_requests": 0}},
                "content": content}})
        t += timedelta(minutes=random.randint(1, 7))
    return enc(cwd), sid, lines


def write_persona(out_root, p):
    random.seed(p["slug"])
    home = os.path.join(out_root, p["slug"])
    shutil.rmtree(home, ignore_errors=True)  # idempotent: clear before regenerating
    projects = os.path.join(home, ".claude", "projects")
    for _ in range(p["sessions"]):
        d, sid, lines = gen_session(p)
        pdir = os.path.join(projects, d)
        os.makedirs(pdir, exist_ok=True)
        with open(os.path.join(pdir, f"{sid}.jsonl"), "w") as f:
            for ln in lines:
                f.write(json.dumps(ln) + "\n")
    first = BASE - timedelta(days=p["tenure_days"])
    with open(os.path.join(home, ".claude.json"), "w") as f:
        json.dump({
            "numStartups": p["startups"],
            "firstStartTime": first.isoformat().replace("+00:00", "Z"),
            "claudeCodeFirstTokenDate": first.isoformat().replace("+00:00", "Z"),
            "skillUsage": {k: {"usageCount": v,
                               "lastUsedAt": int((BASE - timedelta(days=random.randint(1, 20))).timestamp() * 1000)}
                           for k, v in p["skills"].items()},
            "pluginUsage": {},
        }, f, indent=2)
    return home


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "demo-data"
    os.makedirs(out, exist_ok=True)
    for p in PERSONAS:
        print(f"  {p['slug']:18} ({p['domain']:9}) -> {write_persona(out, p)}")
    print("\nLaunch one (after `npm run tauri build`):")
    print(f'  HOME="$PWD/{out}/data-analyst" \\')
    print("    ./src-tauri/target/release/bundle/macos/ember.app/Contents/MacOS/ember")


if __name__ == "__main__":
    main()
