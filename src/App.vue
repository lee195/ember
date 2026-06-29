<script setup lang="ts">
import { ref } from "vue";
import DashboardView from "./views/DashboardView.vue";
import ProfileView from "./views/ProfileView.vue";
import PortableView from "./views/PortableView.vue";
import ProfilesView from "./views/ProfilesView.vue";

type Tab = "dashboard" | "profile" | "portable" | "profiles";
const tab = ref<Tab>("dashboard");
</script>

<template>
  <main>
    <header class="topbar">
      <div class="brand">
        <span class="logo">🔥</span>
        <div>
          <h1>ember</h1>
          <p class="tagline">Your Claude Code usage, at a glance</p>
        </div>
      </div>
      <nav class="tabs">
        <button :class="{ active: tab === 'dashboard' }" @click="tab = 'dashboard'">
          Dashboard
        </button>
        <button :class="{ active: tab === 'profile' }" @click="tab = 'profile'">
          Profile
        </button>
        <button :class="{ active: tab === 'portable' }" @click="tab = 'portable'">
          Portable
        </button>
        <button :class="{ active: tab === 'profiles' }" @click="tab = 'profiles'">
          Profiles
        </button>
      </nav>
    </header>

    <DashboardView v-if="tab === 'dashboard'" />
    <ProfileView v-else-if="tab === 'profile'" />
    <PortableView v-else-if="tab === 'portable'" />
    <ProfilesView v-else />
  </main>
</template>

<style>
:root {
  --bg: #15161a;
  --panel: #1d1f25;
  --border: #2b2e36;
  --text: #e8e9ec;
  --muted: #9aa0ad;
  --muted-2: #6b7280;
  --accent: #ff7a45;
  --track: #2b2e36;
  --hover: #23262e;
}

* {
  box-sizing: border-box;
}

html,
body {
  margin: 0;
  background: var(--bg);
  color: var(--text);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica,
    Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
}
</style>

<style scoped>
main {
  max-width: 1100px;
  margin: 0 auto;
  padding: 24px 28px 48px;
}
.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 12px;
}
.logo {
  font-size: 1.8rem;
}
h1 {
  margin: 0;
  font-size: 1.4rem;
  font-weight: 700;
  letter-spacing: -0.02em;
}
.tagline {
  margin: 2px 0 0;
  font-size: 0.82rem;
  color: var(--muted);
}
.tabs {
  display: flex;
  gap: 4px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 9px;
  padding: 4px;
}
.tabs button {
  background: transparent;
  border: none;
  color: var(--muted);
  padding: 7px 16px;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
}
.tabs button.active {
  background: var(--accent);
  color: #1a1206;
}
</style>
