<template>
  <article class="main-window">
    <AppHeader />
    <section class="main-body">
      <aside class="main-sidebar">
        <MainSidebar />
      </aside>
      <main class="main-content">
        <MainContentFrame />
      </main>
    </section>
  </article>
</template>

<script setup>
import AppHeader from '@/components/layout/AppHeader.vue';
import MainSidebar from './MainSidebar.vue';
import MainContentFrame from './MainContentFrame.vue';
import { useUIStore } from '@/store/ui';

useUIStore().updateCurrentTab('search');
</script>

<style lang="scss" scoped>
@use '@/style/variables.scss' as *;

.main-window {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.main-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.main-sidebar {
  width: $layout-left-sidebar-width;
  flex-shrink: 0;
  height: 100%;
  background-color: var(--c-gray-50);
}

.main-content {
  flex: 1;
  min-width: 0;
  height: 100%;
  overflow: hidden;
}
</style>
