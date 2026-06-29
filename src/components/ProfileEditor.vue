<script setup lang="ts">
import { onMounted, ref } from "vue";
import {
  getProfileConfig,
  setProfileConfig,
  type ProfileConfig,
} from "../lib/api";

const props = defineProps<{ id: string; name: string }>();
const emit = defineEmits<{ close: []; saved: [] }>();

const MODES = ["default", "acceptEdits", "plan", "bypassPermissions"];

const loading = ref(true);
const saving = ref(false);
const error = ref<string | null>(null);
const cfg = ref<ProfileConfig | null>(null);

onMounted(async () => {
  try {
    cfg.value = await getProfileConfig(props.id);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
});

function removeSkill(name: string) {
  if (cfg.value) cfg.value.skills = cfg.value.skills.filter((s) => s !== name);
}

async function save() {
  if (!cfg.value) return;
  saving.value = true;
  error.value = null;
  try {
    await setProfileConfig(props.id, {
      ...cfg.value,
      model: cfg.value.model?.trim() ? cfg.value.model.trim() : null,
      default_mode: cfg.value.default_mode || null,
    });
    emit("saved");
    emit("close");
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal">
      <header>
        <h3>Edit “{{ name }}”</h3>
        <button class="x" @click="emit('close')">✕</button>
      </header>

      <div v-if="loading" class="state">Loading…</div>
      <div v-else-if="error" class="banner error">{{ error }}</div>

      <div v-else-if="cfg" class="form">
        <label>
          <span>Model</span>
          <input
            v-model="cfg.model"
            placeholder="e.g. claude-opus-4-8 (blank = Claude Code default)"
            spellcheck="false"
          />
        </label>

        <label>
          <span>Permission default mode</span>
          <select v-model="cfg.default_mode">
            <option :value="null">(unset)</option>
            <option v-for="m in MODES" :key="m" :value="m">
              {{ m }}{{ m === "bypassPermissions" ? " — auto-approves everything ⚠" : "" }}
            </option>
          </select>
        </label>

        <label>
          <span>CLAUDE.md</span>
          <textarea
            v-model="cfg.claude_md"
            rows="6"
            placeholder="Personal instructions for this profile…"
            spellcheck="false"
          />
        </label>

        <div class="block">
          <span class="blabel">Enabled plugins</span>
          <div v-if="cfg.enabled_plugins.length" class="toggles">
            <label v-for="p in cfg.enabled_plugins" :key="p.name" class="toggle">
              <input type="checkbox" v-model="p.enabled" />
              <code>{{ p.name }}</code>
            </label>
          </div>
          <p v-else class="muted">No plugins configured in this profile.</p>
        </div>

        <div class="block">
          <span class="blabel">Skills</span>
          <div v-if="cfg.skills.length" class="chips">
            <span v-for="s in cfg.skills" :key="s" class="chip">
              {{ s }}
              <button class="chip-x" title="Remove on save" @click="removeSkill(s)">✕</button>
            </span>
          </div>
          <p v-else class="muted">No skills in this profile.</p>
        </div>
      </div>

      <footer v-if="!loading">
        <button class="btn ghost" @click="emit('close')">Cancel</button>
        <button class="btn" :disabled="saving || !cfg" @click="save">
          {{ saving ? "Saving…" : "Save" }}
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
  padding: 24px;
}
.modal {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 12px;
  width: min(620px, 100%);
  max-height: 88vh;
  overflow: auto;
  padding: 20px 22px;
}
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}
h3 {
  margin: 0;
  font-size: 1.1rem;
}
.x {
  background: none;
  border: none;
  color: var(--muted);
  font-size: 1rem;
  cursor: pointer;
}
.form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
label {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 0.82rem;
  color: var(--muted);
}
input,
select,
textarea {
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 7px;
  padding: 8px 10px;
  color: var(--text);
  font-size: 0.85rem;
  font-family: inherit;
}
textarea {
  resize: vertical;
  line-height: 1.5;
}
input:focus,
select:focus,
textarea:focus {
  outline: none;
  border-color: var(--accent);
}
.block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.blabel {
  font-size: 0.82rem;
  color: var(--muted);
}
.toggles {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.toggle {
  flex-direction: row;
  align-items: center;
  gap: 8px;
  color: var(--text);
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 3px 6px 3px 12px;
  font-size: 0.8rem;
  color: var(--text);
}
.chip-x {
  background: none;
  border: none;
  color: var(--muted-2);
  cursor: pointer;
  font-size: 0.72rem;
}
.muted {
  margin: 0;
  color: var(--muted-2);
  font-size: 0.82rem;
}
.banner.error {
  background: var(--bg);
  border: 1px solid #5b2b2b;
  color: #ff9a9a;
  border-radius: 8px;
  padding: 12px 14px;
  font-size: 0.85rem;
}
.state {
  padding: 30px 0;
  text-align: center;
  color: var(--muted);
}
footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 20px;
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
.btn.ghost {
  background: transparent;
  color: var(--muted);
  border: 1px solid var(--border);
}
.btn:disabled {
  opacity: 0.55;
  cursor: default;
}
</style>
