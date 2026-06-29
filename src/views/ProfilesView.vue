<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  listProfiles,
  createProfile,
  renameProfile,
  deleteProfile,
  launchProfiles,
  openProfileFolder,
  pickFolder,
  type ProfileMeta,
} from "../lib/api";
import Section from "../components/Section.vue";
import ProfileEditor from "../components/ProfileEditor.vue";
import NamePrompt from "../components/NamePrompt.vue";

const profiles = ref<ProfileMeta[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const busy = ref(false);

const newName = ref("");
const selected = ref<Set<string>>(new Set());
const editing = ref<ProfileMeta | null>(null);

// In-app name dialog (window.prompt is unavailable in the macOS webview).
const namePrompt = ref<{
  title: string;
  initial: string;
  resolve: (v: string | null) => void;
} | null>(null);

function askName(title: string, initial = ""): Promise<string | null> {
  return new Promise((resolve) => {
    namePrompt.value = { title, initial, resolve };
  });
}
function onNameConfirm(v: string) {
  namePrompt.value?.resolve(v);
  namePrompt.value = null;
}
function onNameCancel() {
  namePrompt.value?.resolve(null);
  namePrompt.value = null;
}

const selectedCount = computed(() => selected.value.size);

async function refresh() {
  loading.value = true;
  error.value = null;
  try {
    profiles.value = await listProfiles();
    // drop selections that no longer exist
    const ids = new Set(profiles.value.map((p) => p.id));
    selected.value = new Set([...selected.value].filter((id) => ids.has(id)));
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(refresh);

function toggleSelect(id: string) {
  const s = new Set(selected.value);
  s.has(id) ? s.delete(id) : s.add(id);
  selected.value = s;
}

async function run<T>(fn: () => Promise<T>) {
  busy.value = true;
  error.value = null;
  try {
    await fn();
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function newBlank() {
  const name = newName.value.trim();
  if (!name) return;
  await run(async () => {
    await createProfile(name, { kind: "blank" });
    newName.value = "";
    await refresh();
  });
}

async function importBundle() {
  const path = await pickFolder("Select a profile bundle folder…");
  if (!path) return;
  const suggested = path.split("/").filter(Boolean).pop() ?? "Imported profile";
  const name = await askName("Name for the imported profile", suggested);
  if (!name) return;
  await run(async () => {
    await createProfile(name, { kind: "importBundle", path });
    await refresh();
  });
}

async function clone(p: ProfileMeta) {
  const name = await askName(`Name for the clone of “${p.name}”`, `${p.name} copy`);
  if (!name) return;
  await run(async () => {
    await createProfile(name, { kind: "clone", id: p.id });
    await refresh();
  });
}

async function rename(p: ProfileMeta) {
  const name = await askName("Rename profile", p.name);
  if (!name || name === p.name) return;
  await run(async () => {
    await renameProfile(p.id, name);
    await refresh();
  });
}

async function remove(p: ProfileMeta) {
  if (!confirm(`Delete profile “${p.name}”? Its config folder will be removed. (~/.claude is untouched.)`)) return;
  await run(async () => {
    await deleteProfile(p.id);
    await refresh();
  });
}

function launchOne(p: ProfileMeta) {
  run(() => launchProfiles([p.id]));
}

function launchSelected() {
  if (selectedCount.value === 0) return;
  run(() => launchProfiles([...selected.value]));
}
</script>

<template>
  <div class="profiles">
    <Section title="Profiles" hint="isolated Claude Code configs — try one or run side-by-side">
      <p class="lead">
        Each profile is its own <code>CLAUDE_CONFIG_DIR</code>. Launching opens a
        Terminal running <code>claude</code> against that config — your real
        <code>~/.claude</code> is never touched. (Org/managed settings still apply.)
      </p>

      <div class="create">
        <input
          v-model="newName"
          placeholder="New profile name…"
          spellcheck="false"
          @keyup.enter="newBlank"
        />
        <button class="btn" :disabled="busy || !newName.trim()" @click="newBlank">New blank</button>
        <button class="btn ghost" :disabled="busy" @click="importBundle">Import bundle…</button>
        <span class="spacer" />
        <button class="btn" :disabled="busy || selectedCount < 2" @click="launchSelected">
          Launch {{ selectedCount || "" }} side-by-side
        </button>
      </div>

      <div v-if="error" class="banner error">{{ error }}</div>
      <div v-if="loading" class="state">Loading…</div>
      <p v-else-if="profiles.length === 0" class="state muted">
        No profiles yet — create a blank one or import a bundle.
      </p>

      <ul v-else class="list">
        <li v-for="p in profiles" :key="p.id" class="card">
          <label class="pick">
            <input
              type="checkbox"
              :checked="selected.has(p.id)"
              @change="toggleSelect(p.id)"
            />
          </label>
          <div class="meta">
            <div class="name">{{ p.name }}</div>
            <div class="path" :title="p.config_dir">{{ p.config_dir }}</div>
          </div>
          <div class="actions">
            <button class="btn sm" :disabled="busy" @click="launchOne(p)">Launch</button>
            <button class="btn sm ghost" :disabled="busy" @click="editing = p">Edit</button>
            <button class="btn sm ghost" :disabled="busy" @click="clone(p)">Clone</button>
            <button class="btn sm ghost" :disabled="busy" @click="rename(p)">Rename</button>
            <button class="btn sm ghost" :disabled="busy" @click="openProfileFolder(p.id)">Folder</button>
            <button class="btn sm danger" :disabled="busy" @click="remove(p)">Delete</button>
          </div>
        </li>
      </ul>
    </Section>

    <ProfileEditor
      v-if="editing"
      :id="editing.id"
      :name="editing.name"
      @close="editing = null"
      @saved="refresh"
    />

    <NamePrompt
      v-if="namePrompt"
      :title="namePrompt.title"
      :initial="namePrompt.initial"
      @confirm="onNameConfirm"
      @cancel="onNameCancel"
    />
  </div>
</template>

<style scoped>
.profiles {
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.lead {
  margin: 0 0 16px;
  color: var(--muted);
  font-size: 0.88rem;
  line-height: 1.5;
}
.lead code {
  color: var(--text);
}
.create {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}
.create input {
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 7px;
  padding: 8px 10px;
  color: var(--text);
  font-size: 0.85rem;
  min-width: 220px;
}
.create input:focus {
  outline: none;
  border-color: var(--accent);
}
.spacer {
  flex: 1;
}
.list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.card {
  display: flex;
  align-items: center;
  gap: 12px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 12px 14px;
}
.pick input {
  width: 16px;
  height: 16px;
  accent-color: var(--accent);
}
.meta {
  flex: 1;
  min-width: 0;
}
.name {
  color: var(--text);
  font-weight: 600;
  font-size: 0.92rem;
}
.path {
  color: var(--muted-2);
  font-size: 0.72rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  justify-content: flex-end;
}
.btn {
  background: var(--accent);
  color: #1a1206;
  border: none;
  border-radius: 7px;
  padding: 8px 14px;
  font-weight: 600;
  font-size: 0.82rem;
  cursor: pointer;
}
.btn.sm {
  padding: 5px 10px;
  font-size: 0.78rem;
}
.btn.ghost {
  background: transparent;
  color: var(--muted);
  border: 1px solid var(--border);
}
.btn.danger {
  background: transparent;
  color: #d9776b;
  border: 1px solid #5b2b2b;
}
.btn:disabled {
  opacity: 0.5;
  cursor: default;
}
.banner.error {
  background: var(--panel);
  border: 1px solid #5b2b2b;
  color: #ff9a9a;
  border-radius: 8px;
  padding: 12px 14px;
  font-size: 0.85rem;
  margin-bottom: 12px;
}
.state {
  padding: 24px 0;
  text-align: center;
  color: var(--muted);
}
.state.muted {
  color: var(--muted-2);
}
</style>
