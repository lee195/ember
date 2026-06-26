<script setup lang="ts">
defineProps<{
  label: string;
  value: string;
  sub?: string;
}>();
</script>

<template>
  <div class="card">
    <div class="value">{{ value }}</div>
    <div class="label">{{ label }}</div>
    <div v-if="sub" class="sub">{{ sub }}</div>
  </div>
</template>

<style scoped>
.card {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 16px 18px;
  min-width: 0;
}
.value {
  font-size: 1.7rem;
  font-weight: 650;
  color: var(--text);
  line-height: 1.1;
  letter-spacing: -0.02em;
}
.label {
  margin-top: 6px;
  font-size: 0.8rem;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.sub {
  margin-top: 4px;
  font-size: 0.78rem;
  color: var(--muted-2);
}
</style>
