<template>
  <aside class="notebook-sidebar">
    <nav class="nb-list">
      <h2 class="nb-heading">生词本</h2>
      <section
        v-for="nb in store.notebooks"
        :key="nb.id"
        class="nb-item"
        :class="{ active: nb.id === store.selectedNotebookId }"
      >
        <button
          type="button"
          class="nb-select"
          @click="store.selectedNotebookId = nb.id"
        >
          <i :class="['nb-icon', nb.is_default ? 'icon-star' : 'icon-book']"></i>
          <span class="nb-name" :title="nb.name">{{ nb.name }}</span>
          <span class="nb-count">{{ store.counts[nb.id] || 0 }}</span>
        </button>
        <NDropdown
          trigger="click"
          placement="bottom-end"
          :options="menuOptions(nb)"
          @select="(key) => onMenuSelect(key, nb)"
        >
          <button type="button" class="nb-more" title="更多操作" @click.stop>
            <i class="icon-cog" style="font-size:13px; width:13px; height:13px;"></i>
          </button>
        </NDropdown>
      </section>
      <p v-if="store.notebooks.length === 0" class="nb-empty">暂无生词本</p>
    </nav>

    <footer class="nb-footer">
      <button type="button" class="action-btn" title="新建生词本" @click="openCreate">
        <i class="icon-plus"></i>
      </button>
    </footer>

    <NModal v-model:show="formShow" preset="dialog" :title="formTitle" :show-icon="false">
      <NInput
        ref="formInputRef"
        v-model:value="formValue"
        placeholder="生词本名称"
        @keydown.enter="confirmForm"
      />
      <template #action>
        <NButton size="small" @click="formShow = false">取消</NButton>
        <NButton size="small" type="primary" :loading="formLoading" @click="confirmForm">确定</NButton>
      </template>
    </NModal>
  </aside>
</template>

<script lang="ts" setup>
import { nextTick, ref } from 'vue';
import { NButton, NDropdown, NInput, NModal, useMessage } from 'naive-ui';
import type { DropdownOption } from 'naive-ui';
import { useBookmarkStore } from '@/store/bookmark';

const store = useBookmarkStore();
const message = useMessage();

type Notebook = { id: string; name: string; is_default: boolean };

const formShow = ref(false);
const formTitle = ref('');
const formValue = ref('');
const formLoading = ref(false);
const formInputRef = ref<InstanceType<typeof NInput> | null>(null);
let editingNbId = '';

function menuOptions(nb: Notebook): DropdownOption[] {
  return nb.is_default
    ? [{ label: '重命名', key: 'rename' }]
    : [
        { label: '重命名', key: 'rename' },
        { label: '删除', key: 'delete' },
      ];
}

function openCreate() {
  editingNbId = '';
  formTitle.value = '新建生词本';
  formValue.value = '';
  formShow.value = true;
  nextTick(() => formInputRef.value?.focus());
}

async function onMenuSelect(key: string, nb: Notebook) {
  if (key === 'rename') {
    editingNbId = nb.id;
    formTitle.value = '重命名生词本';
    formValue.value = nb.name;
    formShow.value = true;
    await nextTick();
    formInputRef.value?.focus();
  } else if (key === 'delete') {
    try {
      await store.deleteNotebook(nb.id);
    } catch (e) {
      message.error((e as Error)?.message || '删除失败');
    }
  }
}

async function confirmForm() {
  const name = formValue.value.trim();
  if (!name) { message.warning('请输入名称'); return; }
  formLoading.value = true;
  try {
    if (editingNbId) {
      await store.renameNotebook(editingNbId, name);
    } else {
      await store.createNotebook(name);
    }
    formShow.value = false;
  } catch (e) {
    message.error((e as Error)?.message || '操作失败');
  } finally {
    formLoading.value = false;
  }
}
</script>

<style lang="scss" scoped>
.notebook-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--c-gray-50);
  border-right: 1px solid var(--c-gray-200);
}

.nb-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 6px 0;

  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-track { background: transparent; }
  &::-webkit-scrollbar-thumb { background: var(--c-gray-300); border-radius: 2px; }
  scrollbar-width: thin;
  scrollbar-color: var(--c-gray-300) transparent;
}

.nb-heading {
  font-size: 10px;
  font-weight: 700;
  color: var(--c-gray-500);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  padding: 0 10px 6px;
  margin: 0;
}

.nb-item {
  display: flex;
  align-items: center;
  width: 100%;

  &.active .nb-select {
    background: var(--c-gray-200);
    color: var(--c-gray-900);
    font-weight: 600;
  }

  &:hover .nb-more { opacity: 1; }
}

.nb-select {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px 5px 10px;
  border: none;
  background: transparent;
  font: inherit;
  font-size: 12px;
  color: var(--c-gray-700);
  text-align: left;
  cursor: pointer;
  transition: background 0.1s;

  &:hover { background: var(--c-gray-100); }
}

.nb-icon {
  width: 13px;
  height: 13px;
  font-size: 13px;
  flex-shrink: 0;
  color: var(--c-gray-500);
}

.nb-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.nb-count {
  flex-shrink: 0;
  font-size: 10px;
  color: var(--c-gray-400);
  min-width: 14px;
  text-align: right;
}

.nb-more {
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 5px;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--c-gray-500);
  opacity: 0;
  transition: background 0.1s, opacity 0.1s;
  margin-right: 4px;

  &:hover { background: var(--c-gray-200); color: var(--c-gray-800); }
}

.nb-empty {
  padding: 16px 10px;
  text-align: center;
  color: var(--c-gray-400);
  font-size: 12px;
  margin: 0;
}

.nb-footer {
  display: flex;
  align-items: center;
  border-top: 1px solid var(--c-gray-200);
  padding: 4px 6px;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 5px;
  background: transparent;
  cursor: pointer;
  color: var(--c-gray-600);
  font-size: 14px;
  transition: background 0.12s, color 0.12s;

  &:hover { background: var(--c-gray-200); color: var(--c-gray-900); }
  i { width: 14px; height: 14px; font-size: 14px; }
}
</style>
