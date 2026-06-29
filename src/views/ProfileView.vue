<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { getProfile, formatPercent, type StyleProfile } from "../lib/api";
import Section from "../components/Section.vue";
import StatCard from "../components/StatCard.vue";
import RankedList from "../components/RankedList.vue";
import ArchetypeHeader from "../components/ArchetypeHeader.vue";
import TraitCard from "../components/TraitCard.vue";
import NarrativePanel from "../components/NarrativePanel.vue";

const data = ref<StyleProfile | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);

const COLOR_EMBER = "#ff7a45";
const COLOR_IN = "#4f8cff";

async function load() {
  loading.value = true;
  error.value = null;
  try {
    data.value = await getProfile();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);

const langRows = computed(() =>
  (data.value?.stack.languages ?? []).map((l) => ({
    label: l.language,
    value: l.count,
    display: String(l.count),
  })),
);

const skillRows = computed(() =>
  (data.value?.skills ?? [])
    .filter((s) => s.usage_count > 0)
    .map((s) => ({
      label: s.name,
      value: s.usage_count,
      display: `${s.usage_count}×`,
    })),
);

const toolRows = computed(() =>
  (data.value?.tools_top ?? []).map((t) => ({
    label: t.name,
    value: t.count,
    display: String(t.count),
  })),
);
</script>

<template>
  <div>
    <div class="toolbar">
      <button class="refresh" :disabled="loading" @click="load">
        {{ loading ? "Loading…" : "Refresh" }}
      </button>
    </div>

    <div v-if="loading" class="state">Analyzing your style…</div>

    <div v-else-if="error" class="state error">
      <p>Couldn't build your profile.</p>
      <pre>{{ error }}</pre>
      <button class="refresh" @click="load">Retry</button>
    </div>

    <div v-else-if="data && !data.found" class="state">
      <p>No Claude Code data found.</p>
      <p class="muted">Looked in <code>{{ data.claude_dir }}</code>.</p>
    </div>

    <div v-else-if="data" class="profile">
      <ArchetypeHeader
        :title="data.archetype.title"
        :summary="data.archetype.summary"
        :tenure-days="data.tenure_days"
        :active-days="data.total_active_days"
        :startups="data.num_startups"
      />

      <div class="traits">
        <TraitCard v-for="t in data.traits" :key="t.key" :trait="t" />
      </div>

      <Section
        title="Narrative"
        hint="LLM-written, your model of choice"
        collapsible
        :default-open="false"
      >
        <NarrativePanel />
      </Section>

      <Section title="Communication style" hint="local heuristics over your prompts">
        <div class="comm-grid">
          <StatCard
            label="Avg prompt length"
            :value="data.communication.avg_prompt_words.toFixed(0)"
            sub="words"
          />
          <StatCard
            label="Quick commands"
            :value="formatPercent(data.communication.short_command_ratio)"
            sub="≤ 5 words"
          />
          <StatCard
            label="Questions"
            :value="formatPercent(data.communication.question_ratio)"
            sub="contain '?'"
          />
          <StatCard
            label="Code-specific"
            :value="formatPercent(data.communication.specificity_ratio)"
            sub="paths / code refs"
          />
          <StatCard
            label="Politeness"
            :value="formatPercent(data.communication.politeness_ratio)"
            sub="please / thanks"
          />
          <StatCard
            label="Vocabulary"
            :value="formatPercent(data.communication.vocabulary_richness)"
            sub="unique-word ratio"
          />
        </div>
        <p class="note">
          Computed entirely on-device from {{ data.communication.prompts }} prompts —
          no prompt text is stored or sent anywhere.
        </p>
      </Section>

      <div class="two-col">
        <Section title="Tech stack" hint="files you touch most">
          <RankedList :rows="langRows" :color="COLOR_EMBER" />
        </Section>
        <Section title="Top tools" hint="by call count">
          <RankedList :rows="toolRows" :color="COLOR_IN" />
        </Section>
      </div>

      <Section
        v-if="skillRows.length"
        title="Skills you rely on"
        hint="from skill usage"
      >
        <RankedList :rows="skillRows" :color="COLOR_EMBER" />
      </Section>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 18px;
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
.profile {
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.traits {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 14px;
}
.comm-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 14px;
}
.note {
  margin: 14px 0 0;
  font-size: 0.78rem;
  color: var(--muted-2);
}
.two-col {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 18px;
  align-items: start;
}
@media (max-width: 860px) {
  .two-col {
    grid-template-columns: 1fr;
  }
}
</style>
