<style lang="scss">
@use '@/style/variables.scss' as *;

.header-search-box {
  display: flex;
  align-items: center;
  height: 100%;
  width: 100%;
  padding: 0;
  margin: 0;
  gap: 4px;
}

.header-navigate-btns {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
}

.btn-nav {
  width: 28px;
  height: 30px;
  padding: 0;
  border: none;
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
  color: var(--c-gray-700);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.12s;

  &:hover { background: rgba(0,0,0,.06); }
  &:active { background: rgba(0,0,0,.1); }

  &:disabled {
    color: var(--c-gray-400);
    cursor: not-allowed;
    opacity: 0.55;
    &:hover { background: transparent; }
  }

  .nav-icon {
    width: 14px;
    height: 14px;
    font-size: 14px;
  }
}

.header-search-input {
  flex: 1;
  min-width: 0;
  height: 30px;

  .n-input {
    height: 30px;
    border: 1px solid var(--c-gray-300);
    border-radius: 8px;
    box-shadow: none;
    font-size: 13px;
    background: #fff;
    transition: border-color 0.15s;

    &:focus-within { border-color: var(--c-primary); }
  }
}

.is-disabled .header-search-input .n-input {
  background: var(--c-gray-100);
  border-color: var(--c-gray-200);
  cursor: not-allowed;
}

/* Popover rendered in body, must not be scoped */
.nb-picker {
  padding: 4px 0;

  .nb-picker-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--c-gray-500);
    padding: 2px 10px 6px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .nb-picker-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    cursor: pointer;
    font-size: 13px;
    color: var(--c-gray-800);
    transition: background 0.1s;

    &:hover { background: var(--c-gray-100); }

    .nb-picker-icon {
      font-size: 14px;
      flex-shrink: 0;
    }
    .nb-picker-name {
      flex: 1;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .nb-picker-tag {
      font-size: 10px;
      color: var(--c-gray-500);
      border: 1px solid var(--c-gray-300);
      border-radius: 3px;
      padding: 1px 5px;
      flex-shrink: 0;
    }
  }

  .nb-picker-empty {
    text-align: center;
    color: var(--c-gray-500);
    font-size: 12px;
    padding: 14px 0;
  }
}
</style>

<template>
  <section class="header-search-box" :class="{ 'is-disabled': searchDisabled }">
    <nav class="header-navigate-btns">
      <button
        type="button"
        class="btn-nav btn-nav-left"
        :disabled="searchDisabled"
        title="后退"
        @click="backHistory()"
      >
        <i class="nav-icon icon-back"></i>
      </button>

      <button
        type="button"
        class="btn-nav btn-nav-right"
        :disabled="searchDisabled"
        title="前进"
        @click="forwardHistory()"
      >
        <i class="nav-icon icon-forward"></i>
      </button>

      <NPopover
        trigger="click"
        v-model:show="bookmarkPopoverShow"
        placement="bottom-start"
        :width="200"
        @update:show="onBookmarkPopoverShow"
      >
        <template #trigger>
          <button
            type="button"
            class="btn-nav"
            :disabled="searchDisabled"
            title="加入生词本"
          >
            <i class="nav-icon icon-star"></i>
          </button>
        </template>
        <nav class="nb-picker">
          <p class="nb-picker-title">加入生词本</p>
          <button
            v-for="nb in bookmarkStore.notebooks"
            :key="nb.id"
            type="button"
            class="nb-picker-item"
            @click="addToNotebook(nb)"
          >
            <i :class="['nb-picker-icon', nb.is_default ? 'icon-star' : 'icon-book']"></i>
            <span class="nb-picker-name">{{ nb.name }}</span>
            <span v-if="nb.is_default" class="nb-picker-tag">默认</span>
          </button>
          <p v-if="bookmarkStore.notebooks.length === 0" class="nb-picker-empty">
            暂无生词本
          </p>
        </nav>
      </NPopover>
    </nav>

    <section class="header-search-input">
      <NInput
        type="text"
        size="small"
        placeholder="搜索单词…"
        :disabled="searchDisabled"
        v-model:value="inputWord"
        @keydown.enter="handleChange"
      >
        <template #suffix>
          <i style="font-size:14px" class="icon-search"></i>
        </template>
      </NInput>
    </section>
  </section>
</template>

<script setup>
import { ref, computed } from 'vue';
import { NInput, NPopover, useMessage } from 'naive-ui';
import { useDictQueryStore } from '@/store/dict';
import { useBookmarkStore } from '@/store/bookmark';
import { useUIStore } from '@/store/ui';

const dictQueryStore = useDictQueryStore();
const bookmarkStore = useBookmarkStore();
const uiStore = useUIStore();
const message = useMessage();

const inputWord = ref(dictQueryStore.inputSearchWord);

dictQueryStore.$onAction(({ name, args, after }) => {
  if (name !== 'updateInputSearchWord') return;
  const requestedWord = args[0];
  if (typeof requestedWord !== 'string' || requestedWord.trim() === '') return;
  after(() => { inputWord.value = dictQueryStore.inputSearchWord; });
});

const searchDisabled = computed(() => !uiStore.isSearchInputActive());

function backHistory() { dictQueryStore.backHistory(); }
function forwardHistory() { dictQueryStore.forwardHistory(); }

const bookmarkPopoverShow = ref(false);
function onBookmarkPopoverShow(show) {
  if (show) bookmarkStore.ensureLoaded();
}

async function addToNotebook(nb) {
  const word = dictQueryStore.inputSearchWord;
  const dict = dictQueryStore.selectDict;
  if (!word || !dict.id) { message.warning('请先输入单词'); return; }
  try {
    await bookmarkStore.addBookmark(word, dict.id, nb.id);
    message.success(`已加入「${nb.name}」`);
    bookmarkPopoverShow.value = false;
  } catch (e) {
    message.error((e && e.message) || '收藏失败');
  }
}

function handleChange() {
  if (!uiStore.isSearchInputActive()) return;
  const word = inputWord.value.trim();
  dictQueryStore.updateInputSearchWord(word);
  dictQueryStore.searchWord(word);
}
</script>
