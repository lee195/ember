<script setup lang="ts">
defineProps<{
  title: string;
  hint?: string;
}>();
</script>

<template>
  <section class="section">
    <header>
      <h2>{{ title }}</h2>
      <span v-if="hint" class="hint">{{ hint }}</span>
    </header>
    <div class="body">
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
