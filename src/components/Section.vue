<script setup lang="ts">
import { ref } from "vue";

const props = withDefaults(
  defineProps<{
    title: string;
    hint?: string;
    collapsible?: boolean;
    defaultOpen?: boolean;
  }>(),
  { collapsible: false, defaultOpen: true },
);

const open = ref(props.defaultOpen);
function toggle() {
  if (props.collapsible) open.value = !open.value;
}
</script>

<template>
  <section class="section">
    <header :class="{ clickable: collapsible }" @click="toggle">
      <div class="left">
        <span v-if="collapsible" class="chev" :class="{ open }" aria-hidden="true">▸</span>
        <h2>{{ title }}</h2>
      </div>
      <span v-if="hint" class="hint">{{ hint }}</span>
    </header>
    <!-- v-show keeps the slot mounted so its state survives collapse/expand -->
    <div v-show="!collapsible || open" class="body">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.section {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 18px 20px 20px;
}
header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 14px;
}
header.clickable {
  cursor: pointer;
  user-select: none;
  margin-bottom: 0;
}
header.clickable + .body {
  margin-top: 14px;
}
.left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
.chev {
  color: var(--accent);
  font-size: 1.25rem;
  line-height: 1;
  transition: transform 0.15s ease;
}
.chev.open {
  transform: rotate(90deg);
}
h2 {
  font-size: 1rem;
  font-weight: 600;
  margin: 0;
  color: var(--text);
}
.hint {
  font-size: 0.78rem;
  color: var(--muted-2);
}
.body {
  min-width: 0;
}
</style>
