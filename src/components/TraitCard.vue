<script setup lang="ts">
import { type Trait } from "../lib/api";

defineProps<{ trait: Trait }>();
</script>

<template>
  <div class="trait">
    <div class="title">{{ trait.title }}</div>
    <div class="value">{{ trait.value }}</div>
    <div class="detail">{{ trait.detail }}</div>
    <div v-if="trait.evidence" class="evidence">{{ trait.evidence }}</div>
  </div>
</template>

<style scoped>
.trait {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 11px;
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.title {
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--muted-2);
}
.value {
  font-size: 1.15rem;
  font-weight: 650;
  color: var(--accent);
  letter-spacing: -0.01em;
}
.detail {
  font-size: 0.85rem;
  color: var(--muted);
  line-height: 1.4;
}
.evidence {
  margin-top: 4px;
  font-size: 0.74rem;
  color: var(--muted-2);
  font-variant-numeric: tabular-nums;
}
</style>
