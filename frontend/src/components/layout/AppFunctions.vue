<style lang="scss">
@use '@/style/variables.scss' as *;

.app-nav-functions {
  display: flex;
  align-items: center;
  height: $layout-header-height;
  gap: 2px;
  padding: 0 4px;
}

.fn-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 44px;
  gap: 3px;
  border: none;
  background: transparent;
  border-radius: 8px;
  cursor: pointer;
  color: $theme-function-box-font-color;
  transition: background 0.15s, color 0.15s;
  padding: 0;

  &:hover {
    background: rgba(0, 0, 0, 0.06);
    color: var(--c-gray-900);
  }

  &.active {
    background: rgba(50, 108, 184, 0.12);
    color: var(--c-primary);
  }

  .fn-icon {
    width: 18px;
    height: 18px;
    font-size: 18px;
  }

  .fn-label {
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.01em;
  }
}
</style>

<template>
  <nav class="app-nav-functions">
    <button class="fn-btn" :class="{ active: uiStore.currentTab === 'search' }" @click="changeTab('search')">
      <i class="fn-icon icon-search"></i>
      <span class="fn-label">搜索</span>
    </button>
    <button class="fn-btn" :class="{ active: uiStore.currentTab === 'dict' }" @click="changeTab('dict')">
      <i class="fn-icon icon-book"></i>
      <span class="fn-label">词典</span>
    </button>
    <button class="fn-btn" :class="{ active: uiStore.currentTab === 'bookmarks' }" @click="changeTab('bookmarks')">
      <i class="fn-icon icon-star"></i>
      <span class="fn-label">生词</span>
    </button>
    <button class="fn-btn" data-test="function-setting" :class="{ active: uiStore.currentTab === 'setting' }" @click="changeTab('setting')">
      <i class="fn-icon icon-cog"></i>
      <span class="fn-label">设置</span>
    </button>
  </nav>
</template>

<script setup>
import { useUIStore } from '@/store/ui';
import { useRouter } from 'vue-router';

const uiStore = useUIStore();
const router = useRouter();

const tabRouters = {
  search: '/',
  dict: '/dict',
  bookmarks: '/bookmarks',
  setting: '/setting',
};

function changeTab(tabName) {
  if (tabRouters[tabName]) {
    router.replace({ path: tabRouters[tabName] });
  }
  uiStore.updateCurrentTab(tabName);
}
</script>
