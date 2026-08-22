<template>
  <aside id="app-sidebar" class="app-sidebar">
    <nav class="sidebar-content">
      <slot></slot>
    </nav>
  </aside>
</template>

<script setup></script>

<style lang="scss" scoped>
@use '@/style/variables.scss' as *;

.app-sidebar {
  display: flex;
  flex-direction: column;
  background-color: var(--c-gray-50);
  height: 100%;
  width: 100%;
  overflow: hidden;
  user-select: none;
  -webkit-user-select: none;
  border-right: 1px solid var(--c-gray-200);
}

.sidebar-content {
  height: 100%;
  overflow-y: auto;
  padding: 4px 0;

  // Modern thin scrollbar
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-track { background: transparent; }
  &::-webkit-scrollbar-thumb {
    background: var(--c-gray-300);
    border-radius: 2px;
    &:hover { background: var(--c-gray-400); }
  }
  scrollbar-width: thin;
  scrollbar-color: var(--c-gray-300) transparent;
}
</style>
