<script setup lang="ts">
import { computed } from "vue";

interface Row {
  label: string;
  value: number;
  display: string;
}

const props = defineProps<{
  rows: Row[];
  color?: string;
}>();

const max = computed(() => Math.max(1, ...props.rows.map((r) => r.value)));
</script>

<template>
  <ul class="list">
    <li v-for="(r, i) in rows" :key="i">
      <div class="row-top">
        <span class="rlabel" :title="r.label">{{ r.label }}</span>
        <span class="rval">{{ r.display }}</span>
      </div>
      <div class="track">
        <div
          class="fill"
          :style="{
            width: (r.value / max) * 100 + '%',
            background: color ?? 'var(--accent)',
          }"
        />
      </div>
    </li>
    <li v-if="rows.length === 0" class="empty">No data.</li>
  </ul>
</template>

<style scoped>
.list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.row-top {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  font-size: 0.85rem;
  margin-bottom: 5px;
}
.rlabel {
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.rval {
  color: var(--muted);
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}
.track {
  height: 7px;
  background: var(--track);
  border-radius: 4px;
  overflow: hidden;
}
.fill {
  height: 100%;
  border-radius: 4px;
}
.empty {
  color: var(--muted-2);
  font-size: 0.85rem;
}
</style>
