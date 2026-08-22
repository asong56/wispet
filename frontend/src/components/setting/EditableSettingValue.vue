<template>
  <section class="editable-setting">
    <section v-if="!editing" class="editable-display">
      <span class="editable-value">{{ displayValue }}</span>
      <button class="btn btn-default" data-test="edit-setting" type="button" @click="startEditing">
        编辑
      </button>
    </section>
    <section v-else class="editable-editor">
      <textarea
        v-if="multiline"
        v-model="draft"
        class="form-control editable-textarea"
        :rows="rows"
      />
      <select v-else-if="options.length" v-model="draft" class="form-control">
        <option v-for="option in options" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
      <input
        v-else
        v-model="draft"
        class="form-control"
        :type="inputType"
        :min="min"
      />
      <footer class="editable-actions">
        <button class="btn btn-default" data-test="cancel-setting" type="button" :disabled="saving" @click="cancel">
          取消
        </button>
        <button class="btn btn-primary" data-test="save-setting" type="button" :disabled="saving" @click="save">
          {{ saving ? '保存中…' : '保存' }}
        </button>
      </footer>
      <p v-if="error" class="editable-error" role="alert">{{ error }}</p>
    </section>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';

interface Option { label: string; value: string; }

const props = withDefaults(defineProps<{
  modelValue: string | number | boolean;
  onSave: (value: string) => Promise<void> | void;
  multiline?: boolean;
  rows?: number;
  inputType?: string;
  min?: number;
  options?: Option[];
}>(), {
  multiline: false,
  rows: 5,
  inputType: 'text',
  min: undefined,
  options: () => [],
});

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();
const editing = ref(false);
const saving = ref(false);
const error = ref('');
const draft = ref(String(props.modelValue));

const displayValue = computed(() => {
  const match = props.options.find((o) => o.value === String(props.modelValue));
  return match?.label ?? String(props.modelValue);
});

watch(() => props.modelValue, (v) => {
  if (!editing.value) draft.value = String(v);
});

function startEditing() {
  draft.value = String(props.modelValue);
  error.value = '';
  editing.value = true;
}

function cancel() {
  editing.value = false;
  error.value = '';
}

async function save() {
  saving.value = true;
  error.value = '';
  try {
    await props.onSave(draft.value);
    emit('update:modelValue', draft.value);
    editing.value = false;
  } catch (e) {
    error.value = e instanceof Error ? e.message : '保存失败';
  } finally {
    saving.value = false;
  }
}
</script>

<style lang="scss" scoped>
.editable-setting {
  width: 100%;
}

.editable-display {
  display: flex;
  align-items: center;
  gap: 10px;
}

.editable-value {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  color: var(--c-gray-700);
  overflow-wrap: anywhere;
}

.editable-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.editable-textarea {
  resize: vertical;
  min-height: 80px;
  font-family: ui-monospace, 'SF Mono', monospace;
  font-size: 12px;
}

.editable-actions {
  display: flex;
  gap: 6px;
  justify-content: flex-end;
}

.editable-error {
  margin: 0;
  font-size: 12px;
  color: #c0392b;
}
</style>
