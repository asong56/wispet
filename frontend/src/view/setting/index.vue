<template>
  <article class="settings-window">
    <AppHeader />
    <section class="settings-body">
      <aside class="settings-sidebar">
        <nav class="settings-nav">
          <RouterLink
            v-for="item in settingsNavigation"
            :key="item.to"
            :to="item.to"
            class="settings-nav-item"
          >
            <i :class="['settings-nav-icon', item.icon]"></i>
            <span class="settings-nav-label">{{ item.label }}</span>
          </RouterLink>
        </nav>
      </aside>
      <main class="settings-content">
        <RouterView />
      </main>
    </section>
  </article>
</template>

<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router';
import AppHeader from '@/components/layout/AppHeader.vue';
import { useUIStore } from '@/store/ui';
import { settingsNavigation } from './settings-navigation';

useUIStore().updateCurrentTab('setting');
</script>

<style lang="scss" scoped>
@use '@/style/variables.scss' as *;

.settings-window {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.settings-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.settings-sidebar {
  width: $layout-left-sidebar-width;
  flex-shrink: 0;
  height: 100%;
  background: var(--c-gray-50);
  border-right: 1px solid var(--c-gray-200);
  overflow: hidden;
}

.settings-content {
  flex: 1;
  min-width: 0;
  height: 100%;
  overflow: hidden;
}

.settings-nav {
  padding: 8px 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.settings-nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border-radius: 7px;
  color: var(--c-gray-700);
  text-decoration: none;
  font-size: 13px;
  font-weight: 500;
  transition: background 0.12s, color 0.12s;

  &:hover {
    background: var(--c-gray-200);
    color: var(--c-gray-900);
  }

  &.router-link-active {
    background: rgba(50, 108, 184, 0.1);
    color: var(--c-primary);
    font-weight: 600;

    .settings-nav-icon {
      background-color: var(--c-primary);
    }
  }
}

.settings-nav-icon {
  width: 15px;
  height: 15px;
  font-size: 15px;
  flex-shrink: 0;
}
</style>
