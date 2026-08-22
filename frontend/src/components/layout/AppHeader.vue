<template>
  <header class="app-header" id="app-header">
    <figure class="app-header-logo">
      <h1>Medict</h1>
    </figure>
    <section class="app-header-func-box">
      <section class="app-header-func-slot">
        <MainFunctions />
      </section>
      <nav class="app-header-func-nav">
        <AppFunctions />
      </nav>
    </section>
  </header>
</template>

<script lang="ts" setup>
import AppFunctions from './AppFunctions.vue';
import MainFunctions from '@/view/main/MainFunctions.vue';
</script>

<style lang="scss" scoped>
@use '@/style/variables.scss' as *;

.app-header {
  width: 100%;
  overflow: hidden;
  padding: 0 16px;
  margin: 0;
  display: flex;
  align-items: center;
  user-select: none;
  -webkit-user-select: none;
  background-color: $theme-top-header-background-color;
  border-bottom: 1px solid var(--c-gray-200);
  height: $layout-header-height;
  max-height: $layout-header-height;
  gap: 8px;
}

.app-header-logo {
  display: flex;
  align-items: center;
  height: 100%;
  width: 68px;
  flex-shrink: 0;
  margin: 0;
  padding: 0;

  h1 {
    margin: 0;
    color: $theme-logo-font-color;
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.3px;
  }
}

.app-header-func-box {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
  height: 100%;
  gap: 4px;
}

.app-header-func-slot {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
  height: 100%;
}

.app-header-func-nav {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  height: 100%;
}
</style>
