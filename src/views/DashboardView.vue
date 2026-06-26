<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  getOverview,
  formatNumber,
  formatDate,
  type Overview,
} from "../lib/api";
import StatCard from "../components/StatCard.vue";
import Section from "../components/Section.vue";
import Bars from "../components/Bars.vue";
import ProjectsTable from "../components/ProjectsTable.vue";
import RankedList from "../components/RankedList.vue";

const data = ref<Overview | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);

const COLOR_IN = "#4f8cff";
const COLOR_OUT = "#42b883";
const COLOR_EMBER = "#ff7a45";

async function load() {
  loading.value = true;
  error.value = null;
  try {
    data.value = await getOverview();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);

const dailyBars = computed(() => {
  const d = data.value?.daily ?? [];
  return d.map((p) => ({
    label: p.date.slice(5), // MM-DD
    tooltip: `${p.date}\ninput ${formatNumber(p.input_tokens)} · output ${formatNumber(
      p.output_tokens,
    )} · ${p.messages} msgs`,
    segments: [
      { value: p.input_tokens, color: COLOR_IN },
      { value: p.output_tokens, color: COLOR_OUT },
    ],
  }));
});

const dailyLabelEvery = computed(() =>
  Math.max(1, Math.ceil((data.value?.daily.length ?? 1) / 12)),
);

const hourlyBars = computed(() => {
  const h = data.value?.hourly ?? [];
  return h.map((v, hour) => ({
    label: String(hour),
    tooltip: `${hour}:00 — ${v} messages`,
    segments: [{ value: v, color: COLOR_EMBER }],
  }));
});

const modelRows = computed(() =>
  (data.value?.models ?? []).map((m) => {
    const total = m.input_tokens + m.output_tokens;
    return {
      label: m.model,
      value: total,
      display: `${formatNumber(total)} tok · ${m.messages} msgs`,
    };
  }),
);

const toolRows = computed(() =>
  (data.value?.tools ?? []).slice(0, 12).map((t) => ({
    label: t.name,
    value: t.count,
    display: String(t.count),
  })),
);

const dateRangeLabel = computed(() => {
  const r = data.value?.date_range;
  if (!r) return "";
  return `${formatDate(r[0])} – ${formatDate(r[1])}`;
});

const cacheHitRate = computed(() => {
  const t = data.value?.totals;
  if (!t) return "—";
  const denom = t.input_tokens + t.cache_read_tokens;
  if (denom === 0) return "—";
  return ((t.cache_read_tokens / denom) * 100).toFixed(0) + "%";
});
</script>

<template>
  <div>
    <div class="toolbar">
      <span v-if="dateRangeLabel" class="range">{{ dateRangeLabel }}</span>
      <button class="refresh" :disabled="loading" @click="load">
        {{ loading ? "Loading…" : "Refresh" }}
      </button>
    </div>

    <div v-if="loading" class="state">Reading your Claude data…</div>

    <div v-else-if="error" class="state error">
      <p>Couldn't read usage data.</p>
      <pre>{{ error }}</pre>
      <button class="refresh" @click="load">Retry</button>
    </div>

    <div v-else-if="data && !data.found" class="state">
      <p>No Claude Code data found.</p>
      <p class="muted">Looked in <code>{{ data.claude_dir }}</code>.</p>
    </div>

    <div v-else-if="data" class="dashboard">
      <div class="cards">
        <StatCard
          label="Sessions"
          :value="formatNumber(data.totals.sessions)"
          :sub="`${data.totals.projects} projects`"
        />
        <StatCard
          label="Messages"
          :value="
            formatNumber(
              data.totals.user_messages + data.totals.assistant_messages,
            )
          "
          :sub="`${formatNumber(data.totals.user_messages)} you · ${formatNumber(
            data.totals.assistant_messages,
          )} Claude`"
        />
        <StatCard
          label="Input tokens"
          :value="formatNumber(data.totals.input_tokens)"
          :sub="`${formatNumber(data.totals.cache_read_tokens)} from cache`"
        />
        <StatCard
          label="Output tokens"
          :value="formatNumber(data.totals.output_tokens)"
        />
        <StatCard
          label="Tool calls"
          :value="formatNumber(data.totals.tool_calls)"
          :sub="`${data.totals.web_searches} searches · ${data.totals.web_fetches} fetches`"
        />
        <StatCard
          label="Cache hit rate"
          :value="cacheHitRate"
          sub="of input read from cache"
        />
      </div>

      <Section title="Token usage over time" hint="input + output per day">
        <Bars :bars="dailyBars" :height="180" :label-every="dailyLabelEvery" />
        <div class="legend">
          <span><i :style="{ background: COLOR_IN }" /> input</span>
          <span><i :style="{ background: COLOR_OUT }" /> output</span>
        </div>
      </Section>

      <Section title="Activity by hour of day" hint="messages, UTC">
        <Bars :bars="hourlyBars" :height="140" :label-every="3" />
      </Section>

      <div class="two-col">
        <Section title="By project" hint="sorted by tokens">
          <ProjectsTable :projects="data.projects" />
        </Section>

        <div class="stack-col">
          <Section title="Models" hint="by total tokens">
            <RankedList :rows="modelRows" :color="COLOR_IN" />
          </Section>
          <Section title="Top tools" hint="by call count">
            <RankedList :rows="toolRows" :color="COLOR_EMBER" />
          </Section>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 14px;
  margin-bottom: 18px;
}
.range {
  font-size: 0.82rem;
  color: var(--muted);
}
.refresh {
  background: var(--accent);
  color: #1a1206;
  border: none;
  border-radius: 7px;
  padding: 8px 16px;
  font-weight: 600;
  font-size: 0.85rem;
  cursor: pointer;
}
.refresh:disabled {
  opacity: 0.6;
  cursor: default;
}
.state {
  padding: 80px 0;
  text-align: center;
  color: var(--muted);
}
.state.error pre {
  text-align: left;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 12px;
  overflow: auto;
  color: #ff9a9a;
  font-size: 0.8rem;
}
.state .muted {
  color: var(--muted-2);
}
.dashboard {
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 14px;
}
.legend {
  display: flex;
  gap: 18px;
  margin-top: 12px;
  font-size: 0.78rem;
  color: var(--muted);
}
.legend i {
  display: inline-block;
  width: 11px;
  height: 11px;
  border-radius: 3px;
  margin-right: 6px;
  vertical-align: middle;
}
.two-col {
  display: grid;
  grid-template-columns: 1.5fr 1fr;
  gap: 18px;
  align-items: start;
}
.stack-col {
  display: flex;
  flex-direction: column;
  gap: 18px;
}
@media (max-width: 860px) {
  .two-col {
    grid-template-columns: 1fr;
  }
}
</style>
