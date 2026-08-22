<template>
  <article class="bookmarks-window">
    <AppHeader />
    <section class="bookmarks-body">
      <aside class="bm-sidebar">
        <NotebookSidebar />
      </aside>
      <main class="bm-content">
        <header class="bm-content-head">
          <section class="bm-title">
            <h2>{{ currentNotebookName }}</h2>
            <span class="bm-count">{{ store.currentBookmarks.length }} 词</span>
          </section>
          <nav class="bm-actions">
            <NInput v-model:value="filter" placeholder="搜索生词…" size="small">
              <template #suffix>
                <i style="font-size:14px" class="icon-search"></i>
              </template>
            </NInput>
            <button
              type="button"
              class="btn btn-default bm-export"
              :disabled="exporting || store.currentBookmarks.length === 0"
              @click="onExportAnki"
            >
              <NIcon><Download /></NIcon>
              {{ exporting ? '导出中…' : '导出 Anki' }}
            </button>
          </nav>
        </header>

        <section class="bm-list" v-if="filtered.length > 0">
          <table class="bm-table">
            <colgroup>
              <col class="col-word" />
              <col />
              <col class="col-time" />
              <col class="col-action" />
            </colgroup>
            <thead>
              <tr>
                <th>单词</th>
                <th>来源词典</th>
                <th>收藏日期</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in filtered"
                :key="item.word + '@' + item.dict_id"
                class="bm-item"
                @click="lookupWord(item)"
              >
                <td class="bm-word">
                  <button type="button" class="bm-lookup" @click.stop="lookupWord(item)">
                    {{ item.word }}
                  </button>
                </td>
                <td class="bm-dict">{{ item.dict_name || item.dict_id }}</td>
                <td class="bm-time">{{ formatDate(item.create_time) }}</td>
                <td class="bm-remove-cell">
                  <button type="button" class="bm-remove" title="移除" @click.stop="remove(item)">
                    <i class="icon-minus"></i>
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </section>

        <section v-else class="bm-empty">
          <i class="bm-empty-icon icon-star"></i>
          <strong>{{ filter ? '无匹配的生词' : '该生词本暂无内容' }}</strong>
          <span>{{ filter ? '尝试其他关键词。' : '在阅读词典时点击星标图标即可加入生词本。' }}</span>
        </section>
      </main>
    </section>
  </article>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { NIcon, NInput, useMessage } from 'naive-ui';
import { Download } from '@vicons/fa';
import AppHeader from '@/components/layout/AppHeader.vue';
import NotebookSidebar from '@/components/bookmarks/NotebookSidebar.vue';
import { useBookmarkStore } from '@/store/bookmark';
import { useDictQueryStore } from '@/store/dict';
import { useRouter } from 'vue-router';
import { exportAnkiToApkg } from '@/apis/bookmark-api';
import { useUIStore } from '@/store/ui';

const store = useBookmarkStore();
const dictQueryStore = useDictQueryStore();
const router = useRouter();
const message = useMessage();
useUIStore().updateCurrentTab('bookmarks');
store.ensureLoaded();

const filter = ref('');
const exporting = ref(false);

const currentNotebookName = computed(() => {
  const nb = store.notebooks.find((n) => n.id === store.selectedNotebookId);
  return nb?.name || '生词本';
});

const filtered = computed(() => {
  const q = filter.value.trim().toLowerCase();
  if (!q) return store.currentBookmarks;
  return store.currentBookmarks.filter((b) => b.word.toLowerCase().includes(q));
});

function formatDate(ts: number | string) {
  if (!ts) return '';
  return new Date(Number(ts) * 1000).toLocaleDateString('zh-CN');
}

function lookupWord(item: { word: string; dict_id: string }) {
  dictQueryStore.updateInputSearchWord(item.word);
  dictQueryStore.searchWord(item.word);
  router.replace('/');
}

async function remove(item: { word: string; dict_id: string; notebook_id: string }) {
  try {
    await store.removeBookmark(item.word, item.dict_id, item.notebook_id);
  } catch (e) {
    message.error((e as Error)?.message || '移除失败');
  }
}

async function onExportAnki() {
  const notebookId = store.selectedNotebookId;
  if (!notebookId) { message.warning('请先选择一个生词本'); return; }
  exporting.value = true;
  try {
    const path = await exportAnkiToApkg(notebookId);
    message.success(`已导出：${path}`);
  } catch (e) {
    message.error((e as Error)?.message || '导出失败');
  } finally {
    exporting.value = false;
  }
}
</script>

<style lang="scss" scoped>
@use '@/style/variables.scss' as *;

.bookmarks-window {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.bookmarks-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.bm-sidebar {
  width: $layout-left-sidebar-width;
  flex-shrink: 0;
  height: 100%;
}

.bm-content {
  flex: 1;
  min-width: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.bm-content-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 14px;
  height: 46px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--c-gray-200);
  background: var(--c-gray-50);
  gap: 12px;
}

.bm-title {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex-shrink: 0;

  h2 { font-size: 13px; font-weight: 600; color: var(--c-gray-900); margin: 0; }
  .bm-count { font-size: 11px; color: var(--c-gray-500); }
}

.bm-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  justify-content: flex-end;

  .n-input { max-width: 200px; }
}

.bm-list {
  flex: 1;
  overflow: auto;

  &::-webkit-scrollbar { width: 6px; height: 6px; }
  &::-webkit-scrollbar-track { background: transparent; }
  &::-webkit-scrollbar-thumb {
    background: var(--c-gray-300);
    border-radius: 3px;
    &:hover { background: var(--c-gray-400); }
  }
  scrollbar-width: thin;
  scrollbar-color: var(--c-gray-300) transparent;
}

.bm-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;

  th {
    position: sticky;
    top: 0;
    padding: 7px 12px;
    text-align: left;
    font-weight: 600;
    font-size: 11px;
    color: var(--c-gray-600);
    background: var(--c-gray-50);
    border-bottom: 1px solid var(--c-gray-200);
    white-space: nowrap;
  }

  td {
    padding: 6px 12px;
    border-bottom: 1px solid var(--c-gray-100);
    vertical-align: middle;
  }

  .bm-item {
    cursor: pointer;
    &:hover td { background: var(--c-gray-50); }
    &:last-child td { border-bottom: none; }
  }

  .col-time { width: 100px; }
  .col-action { width: 36px; }
  .col-word { width: 140px; }
}

.bm-lookup {
  background: none;
  border: none;
  padding: 0;
  color: var(--c-primary);
  font: inherit;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  text-align: left;
  &:hover { text-decoration: underline; }
}

.bm-dict { color: var(--c-gray-600); }
.bm-time { color: var(--c-gray-500); font-size: 11px; }

.bm-remove-cell { text-align: center; }
.bm-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 5px;
  background: transparent;
  cursor: pointer;
  color: var(--c-gray-400);
  font-size: 14px;
  transition: background 0.1s, color 0.1s;
  &:hover { background: var(--c-gray-200); color: #c0392b; }
}

.bm-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 8px;
  color: var(--c-gray-500);
  font-size: 13px;

  .bm-empty-icon { font-size: 32px; color: var(--c-gray-300); margin-bottom: 8px; }
  strong { color: var(--c-gray-700); font-size: 14px; }
}
</style>
