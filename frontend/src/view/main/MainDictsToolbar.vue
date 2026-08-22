<style lang="scss" scoped>
@use '@/style/variables.scss' as *;

.dictionaries {
  display: flex;
  flex-direction: row;
  align-items: center;
  height: 100%;
  padding: 0 4px;
  gap: 3px;
  overflow-x: auto;
  overflow-y: hidden;

  &::-webkit-scrollbar { height: 3px; }
  &::-webkit-scrollbar-track { background: transparent; }
  &::-webkit-scrollbar-thumb {
    background: var(--c-gray-300);
    border-radius: 2px;
  }
  scrollbar-width: thin;
  scrollbar-color: var(--c-gray-300) transparent;
}

.dictionary-item {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  padding: 0;
  border: 1px solid var(--c-gray-200);
  border-radius: 5px;
  cursor: pointer;
  user-select: none;
  -webkit-user-select: none;
  color: var(--c-gray-700);
  background-color: var(--c-gray-50);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: background 0.12s, border-color 0.12s;

  &:hover { background-color: var(--c-gray-200); }

  &:focus-visible {
    outline: 2px solid var(--c-primary);
    outline-offset: 1px;
  }

  &.dictionary-item-active {
    color: var(--c-primary);
    border-color: var(--c-primary);
    background: rgba(50, 108, 184, 0.06);
  }

  .dictionary-icon {
    width: 13px;
    height: 13px;
    font-size: 13px;
  }
}
</style>

<template>
  <nav class="dictionaries">
    <button
      v-for="item in state.dictList"
      type="button"
      class="dictionary-item"
      :class="{ 'dictionary-item-active': isSelected(item) }"
      :key="item.id"
      @click="chooseDict(item)"
      :style="getBackground(item)"
      :title="item.name"
    >
      <i v-if="!item.background" class="dictionary-icon icon-book"></i>
    </button>
  </nav>
</template>

<script setup>
import { useDictQueryStore } from '@/store/dict';
import { useUIStore } from '@/store/ui';
import { reactive, onMounted } from 'vue';
import { BuildIndex } from '@/apis/dicts-api';
import { isDictionarySelected } from './dictionary-toolbar';

const dictQueryStore = useDictQueryStore();
const uiStore = useUIStore();

const state = reactive({ dictList: [] });

function chooseDict(item) { dictQueryStore.updateSelectDict(item); }
function isSelected(item) { return isDictionarySelected(item.id, dictQueryStore.selectDict.id); }

function getBackground(item) {
  if (!item.background) return undefined;
  return {
    backgroundImage: `url(${item.background})`,
    backgroundSize: 'cover',
    backgroundRepeat: 'no-repeat',
    backgroundPosition: 'center',
  };
}

function loadDictionaries() {
  dictQueryStore.queryDictList().then((res) => {
    if (res.length > 0) dictQueryStore.updateSelectDict(res[0]);
    const totalNumber = res.length;
    const updater = updateProgress(totalNumber);

    function sequenceHandle(promiseArr) {
      const pro = promiseArr.shift();
      if (pro && pro.handle) {
        pro.handle().then((resp) => { pro.callback(resp); sequenceHandle(promiseArr); });
      }
    }

    const promiseArray = [];
    for (let i = 0; i < res.length; i++) {
      state.dictList.push(res[i]);
      const { id, name } = res[i];
      promiseArray.push({
        handle: () => BuildIndex(id),
        callback: () => updater(`词典 ${name} 加载完成`),
      });
    }
    sequenceHandle(promiseArray);
  });
}

function updateProgress(totalNumber) {
  let count = 0;
  const queue = [];
  const intv = setInterval(() => {
    if (!queue.length) return;
    const item = queue.shift();
    count++;
    const progress = (count / totalNumber) * 100;
    uiStore.updateProgress(item.hint, progress);
    if (progress >= 100) {
      clearInterval(intv);
      setTimeout(() => uiStore.updateProgress('全部加载完成', 100), 150);
    }
  }, 200);
  return (hint) => queue.push({ hint });
}

onMounted(() => {
  dictQueryStore.initDicts().then(() => loadDictionaries());
});
</script>
