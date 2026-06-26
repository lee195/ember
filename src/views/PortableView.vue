<script setup lang="ts">
import { ref } from "vue";
import {
  exportProfile,
  importProfile,
  pickFolder,
  type ExportResult,
  type ImportPlan,
} from "../lib/api";
import Section from "../components/Section.vue";

const busy = ref(false);
const error = ref<string | null>(null);

// Export state
const exportResult = ref<ExportResult | null>(null);

// Import state
const importSrc = ref<string | null>(null);
const importPlan = ref<ImportPlan | null>(null);
const importApplied = ref(false);

async function doExport() {
  error.value = null;
  const dest = await pickFolder("Export ember profile bundle to…");
  if (!dest) return;
  busy.value = true;
  try {
    exportResult.value = await exportProfile(dest);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function chooseImport() {
  error.value = null;
  importPlan.value = null;
  importApplied.value = false;
  const src = await pickFolder("Select an ember profile bundle folder…");
  if (!src) return;
  importSrc.value = src;
  busy.value = true;
  try {
    importPlan.value = await importProfile(src, false); // dry run
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function applyImport() {
  if (!importSrc.value) return;
  if (
    !confirm(
      "Apply this bundle to ~/.claude? Existing skills / settings / CLAUDE.md listed below will be overwritten.",
    )
  ) {
    return;
  }
  busy.value = true;
  error.value = null;
  try {
    importPlan.value = await importProfile(importSrc.value, true);
    importApplied.value = true;
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="portable">
    <p class="intro">
      Package your Claude Code setup into a portable bundle, or apply a bundle to
      this machine. Bundles contain your style profile, <code>skills/</code>,
      <code>settings.json</code>, and <code>CLAUDE.md</code> —
      <strong>never credentials</strong> (<code>.claude.json</code> is excluded).
    </p>

    <div v-if="error" class="banner error">{{ error }}</div>

    <Section title="Export" hint="write a re-importable bundle">
      <button class="btn" :disabled="busy" @click="doExport">
        {{ busy ? "Working…" : "Choose folder & export" }}
      </button>
      <div v-if="exportResult" class="result">
        <p>
          Bundle written to <code>{{ exportResult.bundle_dir }}</code>
        </p>
        <p class="ok">Included: {{ exportResult.included.join(", ") }}</p>
        <p v-if="exportResult.skipped.length" class="muted">
          Not present, skipped: {{ exportResult.skipped.join(", ") }}
        </p>
      </div>
    </Section>

    <Section title="Import / apply to this machine" hint="dry-run first, then apply">
      <button class="btn" :disabled="busy" @click="chooseImport">
        {{ busy ? "Working…" : "Choose bundle folder" }}
      </button>

      <div v-if="importPlan" class="result">
        <p class="muted">
          Source: <code>{{ importPlan.source_dir }}</code>
        </p>
        <template v-if="importPlan.actions.length">
          <p>{{ importApplied ? "Applied:" : "Would change:" }}</p>
          <ul>
            <li v-for="(a, i) in importPlan.actions" :key="i">{{ a }}</li>
          </ul>
          <button
            v-if="!importApplied"
            class="btn danger"
            :disabled="busy"
            @click="applyImport"
          >
            Apply to this machine
          </button>
          <p v-else class="ok">✓ Applied to ~/.claude</p>
        </template>
        <p v-else class="muted">
          Nothing to apply — no skills / settings / CLAUDE.md found in that folder.
        </p>
      </div>
    </Section>
  </div>
</template>

<style scoped>
.portable {
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.intro {
  margin: 0 0 4px;
  color: var(--muted);
  font-size: 0.9rem;
  line-height: 1.55;
  max-width: 75ch;
}
.intro code {
  color: var(--text);
}
.banner.error {
  background: var(--panel);
  border: 1px solid #5b2b2b;
  color: #ff9a9a;
  border-radius: 8px;
  padding: 12px 14px;
  font-size: 0.85rem;
}
.btn {
  background: var(--accent);
  color: #1a1206;
  border: none;
  border-radius: 7px;
  padding: 9px 18px;
  font-weight: 600;
  font-size: 0.85rem;
  cursor: pointer;
}
.btn:disabled {
  opacity: 0.6;
  cursor: default;
}
.btn.danger {
  background: #c0473a;
  color: #fff;
  margin-top: 6px;
}
.result {
  margin-top: 14px;
  font-size: 0.85rem;
  color: var(--muted);
  line-height: 1.5;
}
.result code {
  color: var(--text);
}
.result ul {
  margin: 6px 0 10px;
  padding-left: 20px;
}
.result li {
  font-variant-numeric: tabular-nums;
}
.ok {
  color: #5fd08a;
}
.muted {
  color: var(--muted-2);
}
</style>
