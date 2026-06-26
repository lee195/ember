<script setup lang="ts">
import { computed } from "vue";

interface Segment {
  value: number;
  color: string;
}
interface Bar {
  label: string;
  tooltip?: string;
  segments: Segment[];
}

const props = defineProps<{
  bars: Bar[];
  height?: number;
  /** show every Nth x-axis label to avoid crowding */
  labelEvery?: number;
}>();

const max = computed(() => {
  let m = 0;
  for (const b of props.bars) {
    const total = b.segments.reduce((s, seg) => s + seg.value, 0);
    if (total > m) m = total;
  }
  return m || 1;
});

const everyN = computed(() => props.labelEvery ?? 1);

function pct(v: number): number {
  return (v / max.value) * 100;
}
</script>

<template>
  <div class="chart" :style="{ height: (height ?? 160) + 'px' }">
    <div
      v-for="(bar, i) in bars"
      :key="i"
      class="col"
      :title="bar.tooltip ?? bar.label"
    >
      <div class="stack">
        <div
          v-for="(seg, j) in bar.segments"
          :key="j"
          class="seg"
          :style="{ height: pct(seg.value) + '%', background: seg.color }"
        />
      </div>
      <div class="xlabel" :class="{ hidden: i % everyN !== 0 }">
        {{ bar.label }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.chart {
  display: flex;
  align-items: flex-end;
  gap: 3px;
  width: 100%;
}
.col {
  flex: 1 1 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
  height: 100%;
  justify-content: flex-end;
}
.stack {
  display: flex;
  flex-direction: column-reverse;
  height: 100%;
  border-radius: 3px 3px 0 0;
  overflow: hidden;
  transition: opacity 0.12s;
}
.col:hover .stack {
  opacity: 0.75;
}
.seg {
  width: 100%;
}
.xlabel {
  margin-top: 6px;
  font-size: 0.62rem;
  color: var(--muted-2);
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.xlabel.hidden {
  visibility: hidden;
}
</style>
