<template>
  <article class="dict-window">
    <AppHeader />
    <section class="dict-body">
      <aside class="dict-sidebar">
        <AppSidebar>
          <DictionaryGroupSidebar
            :groups="sidebarGroups"
            :selected-group-id="selectedGroupId"
            :favorite-group-id="favoriteGroupId"
            :default-group-id="DEFAULT_DICTIONARY_GROUP_ID"
            @select="selectedGroupId = $event"
            @create="openCreateGroup"
            @delete="confirmDeleteGroup"
            @toggle-favorite="toggleFavoriteGroup"
          />
        </AppSidebar>
      </aside>

      <main class="dict-content">
        <header class="dict-toolbar">
          <section class="dict-toolbar-title">
            <strong>{{ selectedGroupName }}</strong>
            <span>{{ visibleDicts.length }} 个词典</span>
          </section>
          <NButton v-if="selectedGroupId !== DEFAULT_DICTIONARY_GROUP_ID" size="small" secondary @click="openMembers">
            管理成员
          </NButton>
        </header>

        <section class="dict-table-area">
          <table v-if="visibleDicts.length" class="dict-table">
            <thead>
              <tr>
                <th>词典名称</th>
                <th>内置标题</th>
                <th>类型</th>
                <th>引擎版本</th>
                <th>创建时间</th>
                <th>目录</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in visibleDicts" :key="item.id">
                <td>{{ item.name }}</td>
                <td>{{ item.title }}</td>
                <td>{{ item.dictType }}</td>
                <td>{{ item.generateEngineVersion }}</td>
                <td>{{ item.createDate }}</td>
                <td class="path-cell">{{ item.baseDir }}</td>
              </tr>
            </tbody>
          </table>

          <section v-else class="dict-empty">
            <i class="dict-empty-icon icon-book"></i>
            <strong>{{ selectedGroupId === DEFAULT_DICTIONARY_GROUP_ID ? '尚未加载到词典' : '当前词典组为空' }}</strong>
            <span>{{ selectedGroupId === DEFAULT_DICTIONARY_GROUP_ID ? '请检查词典目录或重新加载应用。' : '点击"管理成员"向该组添加词典。' }}</span>
            <NButton v-if="selectedGroupId !== DEFAULT_DICTIONARY_GROUP_ID" size="small" secondary @click="openMembers">
              管理成员
            </NButton>
          </section>
        </section>
      </main>
    </section>

    <NModal v-model:show="createGroupVisible" preset="dialog" title="新建词典组" :show-icon="false">
      <NInput
        ref="createGroupInput"
        v-model:value="newGroupName"
        placeholder="词典组名称"
        @keydown.enter="saveNewGroup"
      />
      <template #action>
        <NButton @click="createGroupVisible = false">取消</NButton>
        <NButton type="primary" :loading="saving" @click="saveNewGroup">创建</NButton>
      </template>
    </NModal>

    <NModal v-model:show="membersVisible" preset="dialog" title="管理词典组成员" :show-icon="false" style="width: 480px">
      <p class="member-dialog-hint">勾选要加入当前组的词典：</p>
      <section class="member-checkbox-list">
        <NCheckbox
          v-for="dict in dictsList"
          :key="dict.id"
          :checked="memberDraft.includes(dict.id)"
          @update:checked="toggleMember(dict.id, $event)"
        >{{ dict.name }}</NCheckbox>
      </section>
      <template #action>
        <NButton @click="membersVisible = false">取消</NButton>
        <NButton type="primary" :loading="saving" @click="saveMembers">保存</NButton>
      </template>
    </NModal>
  </article>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';
import { NButton, NModal, NInput, NCheckbox, useMessage } from 'naive-ui';
import AppHeader from '@/components/layout/AppHeader.vue';
import AppSidebar from '@/components/layout/AppSidebar.vue';
import DictionaryGroupSidebar from './DictionaryGroupSidebar.vue';
import { GetAllDicts } from '@/apis/dicts-api';
import { getPreferences, savePreferences } from '@/apis/config';
import {
  DEFAULT_DICTIONARY_GROUP_ID,
  parseDictionaryGroups,
  updateDictionaryGroupMembers,
} from './dictionary-groups';

const GROUPS_PREFERENCE_KEY = 'dictionarygroups';
const FAVORITE_GROUP_PREFERENCE_KEY = 'dictionaryfavoritegroup';

interface DictItem {
  id: string;
  name: string;
  dictType: string;
  title: string;
  generateEngineVersion: string;
  createDate: string;
  baseDir: string;
}

const message = useMessage();
const dictsList = ref<DictItem[]>([]);
const customGroups = ref<ReturnType<typeof parseDictionaryGroups>>([]);
const selectedGroupId = ref(DEFAULT_DICTIONARY_GROUP_ID);
const favoriteGroupId = ref('');
const saving = ref(false);
const createGroupVisible = ref(false);
const membersVisible = ref(false);
const newGroupName = ref('');
const createGroupInput = ref(null);
const memberDraft = ref<string[]>([]);

const sidebarGroups = computed(() => customGroups.value);

const selectedGroupName = computed(() => {
  if (selectedGroupId.value === DEFAULT_DICTIONARY_GROUP_ID) return '全部词典';
  return customGroups.value.find((g) => g.id === selectedGroupId.value)?.name ?? '词典组';
});

const selectedCustomGroup = computed(() =>
  customGroups.value.find((g) => g.id === selectedGroupId.value),
);

const visibleDicts = computed(() => {
  if (selectedGroupId.value === DEFAULT_DICTIONARY_GROUP_ID) return dictsList.value;
  const members = selectedCustomGroup.value?.members ?? [];
  return dictsList.value.filter((d) => members.includes(d.id));
});

function openCreateGroup() {
  newGroupName.value = '';
  createGroupVisible.value = true;
}

async function saveNewGroup() {
  const name = newGroupName.value.trim();
  if (!name) return;
  const id = `group-${Date.now()}`;
  const next = [...customGroups.value, { id, name, members: [] }];
  customGroups.value = next;
  saving.value = true;
  try {
    await persistGroups();
    createGroupVisible.value = false;
    selectedGroupId.value = id;
  } catch (cause) {
    customGroups.value = customGroups.value.filter((g) => g.id !== id);
    message.error(cause instanceof Error ? cause.message : '创建词典组失败');
  } finally {
    saving.value = false;
  }
}

async function confirmDeleteGroup(groupId: string) {
  const previous = customGroups.value;
  customGroups.value = customGroups.value.filter((g) => g.id !== groupId);
  if (selectedGroupId.value === groupId) selectedGroupId.value = DEFAULT_DICTIONARY_GROUP_ID;
  if (favoriteGroupId.value === groupId) favoriteGroupId.value = '';
  saving.value = true;
  try {
    await persistGroups();
  } catch (cause) {
    customGroups.value = previous;
    message.error(cause instanceof Error ? cause.message : '删除词典组失败');
  } finally {
    saving.value = false;
  }
}

async function toggleFavoriteGroup(groupId: string) {
  favoriteGroupId.value = favoriteGroupId.value === groupId ? '' : groupId;
  try {
    await savePreferences({ [FAVORITE_GROUP_PREFERENCE_KEY]: favoriteGroupId.value });
  } catch {
    message.error('收藏词典组失败');
  }
}

function openMembers() {
  memberDraft.value = [...(selectedCustomGroup.value?.members ?? [])];
  membersVisible.value = true;
}

function toggleMember(id: string, checked: boolean) {
  if (checked) {
    memberDraft.value = [...memberDraft.value, id];
  } else {
    memberDraft.value = memberDraft.value.filter((m) => m !== id);
  }
}

async function persistGroups() {
  await savePreferences({ [GROUPS_PREFERENCE_KEY]: JSON.stringify(customGroups.value) });
}

async function saveMembers() {
  if (!selectedCustomGroup.value) return;
  const previous = customGroups.value;
  customGroups.value = updateDictionaryGroupMembers(
    customGroups.value,
    selectedCustomGroup.value.id,
    memberDraft.value,
    dictsList.value.map((dict) => dict.id),
  );
  saving.value = true;
  try {
    await persistGroups();
    membersVisible.value = false;
    message.success('词典组成员已更新');
  } catch (cause) {
    customGroups.value = previous;
    message.error(cause instanceof Error ? cause.message : '保存词典组成员失败');
  } finally {
    saving.value = false;
  }
}

onMounted(async () => {
  try {
    const [dictionaries, preferences] = await Promise.all([GetAllDicts(), getPreferences()]);
    dictsList.value = dictionaries.map((dict: any) => ({
      id: dict.id,
      name: dict.name,
      dictType: dict.type || '',
      title: dict.description?.title || '',
      generateEngineVersion: dict.description?.generateEngineVersion || '',
      createDate: dict.description?.createDate || '',
      baseDir: dict.base_dir || '',
    }));
    customGroups.value = parseDictionaryGroups(
      preferences[GROUPS_PREFERENCE_KEY],
      dictsList.value.map((dict) => dict.id),
    );
    const storedFavorite = String(preferences[FAVORITE_GROUP_PREFERENCE_KEY] || '');
    favoriteGroupId.value =
      storedFavorite === DEFAULT_DICTIONARY_GROUP_ID ||
      customGroups.value.some((group) => group.id === storedFavorite)
        ? storedFavorite
        : '';
    selectedGroupId.value = favoriteGroupId.value || DEFAULT_DICTIONARY_GROUP_ID;
  } catch (cause) {
    message.error(cause instanceof Error ? cause.message : '加载词典列表失败');
  }
});
</script>

<style lang="scss" scoped>
@use '@/style/variables.scss' as *;

.dict-window {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.dict-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.dict-sidebar {
  width: $layout-left-sidebar-width;
  flex-shrink: 0;
  height: 100%;
}

.dict-content {
  flex: 1;
  min-width: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.dict-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 14px;
  height: 42px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--c-gray-200);
  background: var(--c-gray-50);
}

.dict-toolbar-title {
  display: flex;
  align-items: baseline;
  gap: 8px;

  strong { color: var(--c-gray-900); font-size: 13px; }
  span { color: var(--c-gray-500); font-size: 11px; }
}

.dict-table-area {
  flex: 1;
  overflow: auto;

  &::-webkit-scrollbar { width: 6px; height: 6px; }
  &::-webkit-scrollbar-track { background: transparent; }
  &::-webkit-scrollbar-thumb {
    background: var(--c-gray-300);
    border-radius: 3px;
    &:hover { background: var(--c-gray-400); }
  }
  scrollbar-width: thin;
  scrollbar-color: var(--c-gray-300) transparent;
}

.dict-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;

  th {
    position: sticky;
    top: 0;
    padding: 8px 12px;
    text-align: left;
    font-weight: 600;
    font-size: 11px;
    color: var(--c-gray-600);
    background: var(--c-gray-50);
    border-bottom: 1px solid var(--c-gray-200);
    white-space: nowrap;
  }

  td {
    padding: 7px 12px;
    color: var(--c-gray-800);
    border-bottom: 1px solid var(--c-gray-100);
  }

  tr:hover td { background: var(--c-gray-50); }

  .path-cell {
    color: var(--c-gray-500);
    font-family: ui-monospace, 'SF Mono', monospace;
    font-size: 11px;
    max-width: 240px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.dict-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 260px;
  gap: 8px;
  color: var(--c-gray-500);
  font-size: 13px;
  padding: 40px;

  .dict-empty-icon {
    font-size: 32px;
    color: var(--c-gray-300);
    margin-bottom: 8px;
  }

  strong {
    color: var(--c-gray-700);
    font-size: 14px;
  }
}

.member-dialog-hint {
  margin: 0 0 12px;
  color: var(--c-gray-600);
  font-size: 13px;
}

.member-checkbox-list {
  display: grid;
  gap: 8px;
  max-height: 320px;
  overflow-y: auto;
}
</style>
