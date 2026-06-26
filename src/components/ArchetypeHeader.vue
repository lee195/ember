<script setup lang="ts">
defineProps<{
  title: string;
  summary: string;
  tenureDays: number;
  activeDays: number;
  startups: number;
}>();
</script>

<template>
  <div class="hero">
    <div class="kicker">Your Claude Code archetype</div>
    <h2 class="title">{{ title }}</h2>
    <p class="summary">{{ summary }}</p>
    <div class="meta">
      <span><b>{{ tenureDays }}</b> days with Claude Code</span>
      <span><b>{{ activeDays }}</b> active days</span>
      <span><b>{{ startups }}</b> launches</span>
    </div>
  </div>
</template>

<style scoped>
.hero {
  background: linear-gradient(135deg, #2a1a12 0%, var(--panel) 70%);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 28px 30px;
}
.kicker {
  font-size: 0.74rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--accent);
  margin-bottom: 8px;
}
.title {
  margin: 0;
  font-size: 2rem;
  font-weight: 750;
  letter-spacing: -0.025em;
  color: var(--text);
}
.summary {
  margin: 12px 0 0;
  max-width: 70ch;
  line-height: 1.55;
  color: var(--muted);
  font-size: 0.95rem;
}
.meta {
  display: flex;
  flex-wrap: wrap;
  gap: 22px;
  margin-top: 18px;
  font-size: 0.82rem;
  color: var(--muted-2);
}
.meta b {
  color: var(--text);
  font-weight: 650;
}
</style>
