<template>
  <section class="dict-group-sidebar">
    <nav class="dict-group-list">
      <h2 class="dict-group-heading">词典组</h2>
      <button
        v-for="group in groups"
        :key="group.id"
        type="button"
        class="dict-group-item"
        :class="{ active: group.id === selectedGroupId }"
        @click="$emit('select', group.id)"
      >
        <i class="group-item-icon icon-archive"></i>
        <span class="group-item-name" :title="group.name">{{ group.name }}</span>
        <i v-if="group.id === favoriteGroupId" class="group-item-fav icon-star"></i>
        <span class="group-item-count">{{ group.count }}</span>
      </button>
    </nav>

    <footer class="dict-group-actions">
      <button type="button" class="action-btn" title="新建词典组" @click="$emit('create')">
        <i class="icon-plus"></i>
      </button>
      <button
        type="button"
        class="action-btn"
        title="删除当前词典组"
        :disabled="selectedGroupId === defaultGroupId"
        @click="$emit('delete')"
      >
        <i class="icon-minus"></i>
      </button>
      <button
        type="button"
        class="action-btn"
        :class="{ active: selectedGroupId === favoriteGroupId }"
        :title="selectedGroupId === favoriteGroupId ? '取消常用词典组' : '设为常用词典组'"
        @click="$emit('toggle-favorite')"
      >
        <i :class="selectedGroupId === favoriteGroupId ? 'icon-star' : 'icon-star-empty'"></i>
      </button>
    </footer>
  </section>
</template>

<script setup lang="ts">
interface GroupListItem { id: string; name: string; count: number; }

defineProps<{
  groups: GroupListItem[];
  selectedGroupId: string;
  favoriteGroupId: string;
  defaultGroupId: string;
}>();

defineEmits<{
  select: [id: string];
  create: [];
  delete: [];
  'toggle-favorite': [];
}>();
</script>

<style lang="scss" scoped>
.dict-group-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--c-gray-50);
}

.dict-group-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 6px 0;

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

.dict-group-heading {
  font-size: 10px;
  font-weight: 700;
  color: var(--c-gray-500);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  padding: 0 10px 6px;
  margin: 0;
}

.dict-group-item {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 5px;
  padding: 5px 8px 5px 10px;
  border: none;
  border-radius: 0;
  background: transparent;
  font: inherit;
  font-size: 12px;
  color: var(--c-gray-700);
  text-align: left;
  cursor: pointer;
  transition: background 0.1s;

  &:hover { background: var(--c-gray-200); }
  &.active {
    background: var(--c-gray-200);
    color: var(--c-gray-900);
    font-weight: 600;
  }
  &:focus-visible {
    outline: 1px solid var(--c-primary);
    outline-offset: -1px;
  }
}

.group-item-icon {
  width: 14px;
  height: 14px;
  font-size: 14px;
  flex-shrink: 0;
  color: var(--c-gray-500);
}

.group-item-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-item-fav {
  width: 11px;
  height: 11px;
  font-size: 11px;
  flex-shrink: 0;
  color: #f0a020;
}

.group-item-count {
  flex-shrink: 0;
  font-size: 10px;
  color: var(--c-gray-400);
  min-width: 14px;
  text-align: right;
}

.dict-group-actions {
  display: flex;
  align-items: center;
  border-top: 1px solid var(--c-gray-200);
  padding: 4px 6px;
  gap: 2px;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 0;
  border: none;
  border-radius: 5px;
  background: transparent;
  cursor: pointer;
  color: var(--c-gray-600);
  font-size: 14px;
  transition: background 0.12s, color 0.12s;

  &:hover:not(:disabled) {
    background: var(--c-gray-200);
    color: var(--c-gray-900);
  }

  &:disabled {
    color: var(--c-gray-300);
    cursor: not-allowed;
  }

  &.active { color: #f0a020; }

  i { width: 14px; height: 14px; font-size: 14px; }
}
</style>
