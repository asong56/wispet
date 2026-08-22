<template>
  <AppSidebar>
    <ul id="word-pending-list" class="word-list">
      <li
        v-for="(item, entryIndex) in dictQueryStore.queryPendingList"
        :data-id="entryIndex"
        :key="`${item.keyword}-${entryIndex}`"
        @click="selectItem(entryIndex)"
        :class="selectedIndex === entryIndex ? 'active' : ''"
      >
        <span class="word-text">{{ item.keyword }}</span>
      </li>
    </ul>
  </AppSidebar>
</template>

<script setup>
import AppSidebar from '@/components/layout/AppSidebar.vue';
import { useDictQueryStore } from '@/store/dict';
import { ref, onMounted, onUnmounted, watch } from 'vue';

const dictQueryStore = useDictQueryStore();
const selectedIndex = ref(0);

watch(
  () => dictQueryStore.queryPendingList,
  () => { selectedIndex.value = 0; }
);

function locateEntry(entryIndex) {
  if (dictQueryStore.multiMode) {
    dictQueryStore.locateInMultiPrimary(entryIndex);
  } else {
    dictQueryStore.locateWord(entryIndex);
  }
}

function selectItem(entryIndex) {
  selectedIndex.value = entryIndex;
  locateEntry(entryIndex);
}

function isTextInputActive() {
  const ae = document.activeElement;
  if (!ae) return false;
  const tag = ae.tagName;
  return tag === 'INPUT' || tag === 'TEXTAREA' || ae.isContentEditable;
}

function onKeyDown(e) {
  if (e.key !== 'ArrowUp' && e.key !== 'ArrowDown') return;
  if (isTextInputActive()) return;
  const len = dictQueryStore.queryPendingList.length;
  if (!len) return;
  if (e.key === 'ArrowUp') {
    selectedIndex.value = selectedIndex.value === 0 ? len - 1 : selectedIndex.value - 1;
  } else {
    selectedIndex.value = selectedIndex.value === len - 1 ? 0 : selectedIndex.value + 1;
  }
  locateEntry(selectedIndex.value);
}

onMounted(() => document.addEventListener('keydown', onKeyDown));
onUnmounted(() => document.removeEventListener('keydown', onKeyDown));
</script>

<style lang="scss" scoped>
.word-list {
  list-style: none;
  margin: 0;
  padding: 4px 6px;

  li {
    padding: 5px 8px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    color: var(--c-gray-800);
    transition: background 0.1s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;

    &:hover { background: var(--c-gray-200); }
    &.active {
      background: rgba(50, 108, 184, 0.1);
      color: var(--c-primary);
      font-weight: 500;
    }
  }
}

.word-text {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
