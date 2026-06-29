<script setup lang="ts">
import { nextTick, onMounted, ref } from "vue";

const props = defineProps<{
  title: string;
  initial?: string;
  confirmLabel?: string;
}>();
const emit = defineEmits<{ confirm: [value: string]; cancel: [] }>();

const value = ref(props.initial ?? "");
const input = ref<HTMLInputElement | null>(null);

onMounted(async () => {
  await nextTick();
  input.value?.focus();
  input.value?.select();
});

function ok() {
  const v = value.value.trim();
  if (v) emit("confirm", v);
}
</script>

<template>
  <div class="overlay" @click.self="emit('cancel')">
    <div class="modal">
      <h3>{{ title }}</h3>
      <input
        ref="input"
        v-model="value"
        spellcheck="false"
        @keyup.enter="ok"
        @keyup.escape="emit('cancel')"
      />
      <div class="actions">
        <button class="btn ghost" @click="emit('cancel')">Cancel</button>
        <button class="btn" :disabled="!value.trim()" @click="ok">
          {{ confirmLabel ?? "OK" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 60;
  padding: 24px;
}
.modal {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 12px;
  width: min(440px, 100%);
  padding: 20px 22px;
}
h3 {
  margin: 0 0 14px;
  font-size: 1rem;
}
input {
  width: 100%;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 7px;
  padding: 9px 11px;
  color: var(--text);
  font-size: 0.9rem;
  font-family: inherit;
}
input:focus {
  outline: none;
  border-color: var(--accent);
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 18px;
}
.btn {
  background: var(--accent);
  color: #1a1206;
  border: none;
  border-radius: 7px;
  padding: 9px 18px;
  font-weight: 600;
  font-size: 0.85rem;
  cursor: pointer;
}
.btn.ghost {
  background: transparent;
  color: var(--muted);
  border: 1px solid var(--border);
}
.btn:disabled {
  opacity: 0.55;
  cursor: default;
}
</style>
