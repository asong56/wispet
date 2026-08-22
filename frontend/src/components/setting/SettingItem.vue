<template>
  <article class="setting-row">
    <header class="setting-row-label">
      <strong class="setting-row-title">{{ title }}</strong>
      <p class="setting-row-desc"><slot name="desc"></slot></p>
    </header>
    <section class="setting-row-value">
      <slot></slot>
    </section>
  </article>
</template>

<script setup lang="ts">
defineProps(['title']);
defineSlots<{ desc(): void; default(): void; }>();
</script>

<style lang="scss" scoped>
.setting-row {
  display: grid;
  grid-template-columns: minmax(160px, 0.85fr) minmax(200px, 1.15fr);
  gap: 16px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--c-gray-100);
  align-items: start;

  &:last-child { border-bottom: none; }
}

.setting-row-label {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.setting-row-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--c-gray-900);
  line-height: 1.4;
}

.setting-row-desc {
  margin: 0;
  font-size: 12px;
  color: var(--c-gray-500);
  line-height: 1.5;
}

.setting-row-value {
  min-width: 0;
  color: var(--c-gray-800);
  font-size: 13px;
  display: flex;
  align-items: flex-start;
}

@media (max-width: 680px) {
  .setting-row { grid-template-columns: 1fr; gap: 6px; }
}
</style>
