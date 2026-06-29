<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  getLlmSettings,
  setLlmSettings,
  generateNarrative,
  getCachedNarrative,
  type LlmSettings,
} from "../lib/api";

const PRESETS: { label: string; url: string }[] = [
  { label: "Ollama", url: "http://localhost:11434/v1" },
  { label: "oMLX", url: "http://localhost:8000/v1" },
  { label: "OpenAI-compatible cloud", url: "https://api.openai.com/v1" },
];

const settings = ref<LlmSettings | null>(null);
const baseUrl = ref("");
const model = ref("");
const apiKeyInput = ref("");
const showSettings = ref(false);

const saving = ref(false);
const generating = ref(false);
const error = ref<string | null>(null);
const narrative = ref<string | null>(null);

const isLocal = computed(() => /\/\/(localhost|127\.0\.0\.1|\[::1\])/.test(baseUrl.value));
const configured = computed(
  () => !!settings.value && settings.value.base_url !== "" && settings.value.model !== "",
);

onMounted(async () => {
  try {
    settings.value = await getLlmSettings();
    baseUrl.value = settings.value.base_url;
    model.value = settings.value.model;
    showSettings.value = !configured.value;
    narrative.value = await getCachedNarrative();
  } catch (e) {
    error.value = String(e);
  }
});

function applyPreset(url: string) {
  baseUrl.value = url;
}

async function save() {
  saving.value = true;
  error.value = null;
  try {
    // Pass the key only if the user typed one; empty input leaves it unchanged.
    const key = apiKeyInput.value.length > 0 ? apiKeyInput.value : undefined;
    await setLlmSettings(baseUrl.value.trim(), model.value.trim(), key);
    apiKeyInput.value = "";
    settings.value = await getLlmSettings();
    if (configured.value) showSettings.value = false;
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

async function clearKey() {
  saving.value = true;
  error.value = null;
  try {
    await setLlmSettings(baseUrl.value.trim(), model.value.trim(), "");
    settings.value = await getLlmSettings();
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

async function generate() {
  generating.value = true;
  error.value = null;
  try {
    narrative.value = await generateNarrative();
  } catch (e) {
    error.value = String(e);
  } finally {
    generating.value = false;
  }
}
</script>

<template>
  <div class="narrative">
    <div class="bar">
      <p class="lead">
        Turn your profile into prose with a local (or OpenAI-compatible) model.
      </p>
      <button class="link" @click="showSettings = !showSettings">
        {{ showSettings ? "Hide settings" : "Settings" }}
      </button>
    </div>

    <div v-if="showSettings" class="settings">
      <div class="presets">
        <span class="plabel">Presets:</span>
        <button v-for="p in PRESETS" :key="p.url" class="chip" @click="applyPreset(p.url)">
          {{ p.label }}
        </button>
      </div>
      <label>
        <span>Base URL</span>
        <input v-model="baseUrl" placeholder="http://localhost:11434/v1" spellcheck="false" />
      </label>
      <label>
        <span>Model</span>
        <input v-model="model" placeholder="e.g. llama3.1, qwen2.5-coder, gpt-4o-mini" spellcheck="false" />
      </label>
      <label>
        <span>API key <em>(only for cloud endpoints)</em></span>
        <input
          v-model="apiKeyInput"
          type="password"
          :placeholder="settings?.has_api_key ? '•••••••• (saved in Keychain)' : 'optional'"
          spellcheck="false"
        />
      </label>
      <div class="actions">
        <button class="btn" :disabled="saving || !baseUrl || !model" @click="save">
          {{ saving ? "Saving…" : "Save settings" }}
        </button>
        <button v-if="settings?.has_api_key" class="link" :disabled="saving" @click="clearKey">
          Clear saved key
        </button>
      </div>
      <p class="note" :class="{ warn: !isLocal && baseUrl }">
        <template v-if="isLocal || !baseUrl">
          Local endpoint — your profile stays on this machine.
        </template>
        <template v-else>
          ⚠ Non-local endpoint — generating will send your aggregated profile to
          <code>{{ baseUrl }}</code>.
        </template>
      </p>
    </div>

    <div class="generate-row">
      <button class="btn" :disabled="generating || !configured" @click="generate">
        {{ generating ? "Generating…" : narrative ? "Regenerate" : "Generate narrative" }}
      </button>
      <span v-if="!configured" class="hint">Configure a model in settings first.</span>
    </div>

    <div v-if="error" class="banner error">{{ error }}</div>

    <blockquote v-if="narrative" class="prose">{{ narrative }}</blockquote>
  </div>
</template>

<style scoped>
.narrative {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.bar {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
}
.lead {
  margin: 0;
  color: var(--muted);
  font-size: 0.88rem;
}
.settings {
  display: flex;
  flex-direction: column;
  gap: 12px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 16px;
}
.presets {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.plabel {
  font-size: 0.78rem;
  color: var(--muted-2);
}
.chip {
  background: var(--panel);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 999px;
  padding: 4px 12px;
  font-size: 0.78rem;
  cursor: pointer;
}
.chip:hover {
  border-color: var(--accent);
}
label {
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-size: 0.8rem;
  color: var(--muted);
}
label em {
  color: var(--muted-2);
  font-style: normal;
}
input {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 7px;
  padding: 8px 10px;
  color: var(--text);
  font-size: 0.85rem;
  font-family: inherit;
}
input:focus {
  outline: none;
  border-color: var(--accent);
}
.actions {
  display: flex;
  align-items: center;
  gap: 14px;
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
  opacity: 0.55;
  cursor: default;
}
.link {
  background: none;
  border: none;
  color: var(--muted);
  font-size: 0.82rem;
  cursor: pointer;
  text-decoration: underline;
  padding: 0;
}
.note {
  margin: 0;
  font-size: 0.78rem;
  color: var(--muted-2);
}
.note.warn {
  color: #e0a23a;
}
.note code {
  color: var(--text);
}
.generate-row {
  display: flex;
  align-items: center;
  gap: 12px;
}
.hint {
  font-size: 0.8rem;
  color: var(--muted-2);
}
.banner.error {
  background: var(--panel);
  border: 1px solid #5b2b2b;
  color: #ff9a9a;
  border-radius: 8px;
  padding: 12px 14px;
  font-size: 0.85rem;
}
.prose {
  margin: 0;
  padding: 16px 18px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-left: 3px solid var(--accent);
  border-radius: 8px;
  color: var(--text);
  font-size: 0.95rem;
  line-height: 1.6;
  white-space: pre-wrap;
}
</style>
