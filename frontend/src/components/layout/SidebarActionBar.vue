<template>
  <footer class="sidebar-action-bar">
    <slot />
  </footer>
</template>

<style lang="scss" scoped>
.sidebar-action-bar {
  display: flex;
  align-items: center;
  flex: 0 0 32px;
  height: 32px;
  padding: 3px 6px;
  border-top: 1px solid var(--c-gray-200);
  background: var(--c-gray-50);

  :deep(button) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 24px;
    padding: 0;
    color: var(--c-gray-600);
    background: transparent;
    border: none;
    border-radius: 5px;
    cursor: pointer;
    font: inherit;
    transition: background 0.12s, color 0.12s;

    &:hover:not(:disabled) { background: var(--c-gray-200); color: var(--c-gray-900); }
    &.active { color: var(--c-primary); }
    &:disabled { color: var(--c-gray-300); cursor: not-allowed; }
    &:focus-visible { outline: 1px solid var(--c-primary); outline-offset: -1px; }
  }
}
</style>
