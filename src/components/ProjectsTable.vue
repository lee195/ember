<script setup lang="ts">
import { type ProjectSummary, formatNumber, formatDate } from "../lib/api";

defineProps<{ projects: ProjectSummary[] }>();
</script>

<template>
  <table>
    <thead>
      <tr>
        <th class="name">Project</th>
        <th class="num">Sessions</th>
        <th class="num">Messages</th>
        <th class="num">Tokens (in / out)</th>
        <th class="num">Last active</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="p in projects" :key="p.path">
        <td class="name" :title="p.path">{{ p.name }}</td>
        <td class="num">{{ p.sessions }}</td>
        <td class="num">{{ p.messages }}</td>
        <td class="num">
          {{ formatNumber(p.input_tokens) }} /
          {{ formatNumber(p.output_tokens) }}
        </td>
        <td class="num">{{ formatDate(p.last_active) }}</td>
      </tr>
      <tr v-if="projects.length === 0">
        <td colspan="5" class="empty">No projects found.</td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.85rem;
}
th,
td {
  padding: 8px 10px;
  text-align: left;
  border-bottom: 1px solid var(--border);
}
th {
  color: var(--muted);
  font-weight: 500;
  font-size: 0.74rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.num {
  text-align: right;
  font-variant-numeric: tabular-nums;
}
.name {
  color: var(--text);
  max-width: 240px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
tbody tr:hover {
  background: var(--hover);
}
.empty {
  text-align: center;
  color: var(--muted-2);
  padding: 20px;
}
</style>
