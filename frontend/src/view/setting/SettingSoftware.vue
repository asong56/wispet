<template>
  <SettingPage>
    <section class="settings-section">
      <h2 class="settings-section-title">搜索</h2>
      <SettingItem title="全文搜索引擎">
        <template #desc>用于全文检索的内置索引引擎。</template>
        <span class="readonly-value">Bleve</span>
      </SettingItem>
      <SettingItem title="索引超时时间">
        <template #desc>建立全文索引允许等待的最长时间，例如 <code>10s</code> 或 <code>1m</code>。</template>
        <EditableSettingValue v-model="indexTimeout" :on-save="saveTimeout" />
      </SettingItem>
    </section>
  </SettingPage>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { getPreferences, savePreferences } from '@/apis/config';
import EditableSettingValue from '@/components/setting/EditableSettingValue.vue';
import SettingItem from '@/components/setting/SettingItem.vue';
import SettingPage from '@/components/setting/SettingPage.vue';

const indexTimeout = ref('10s');
const saveTimeout = (value: string) => savePreferences({ fulltextindextimeout: value });

onMounted(async () => {
  const preferences = await getPreferences();
  indexTimeout.value = String(preferences.fulltextindextimeout ?? '10s');
});
</script>

<style scoped>
.readonly-value { color: var(--c-gray-700); font-size: 13px; }
code {
  font-family: ui-monospace, 'SF Mono', monospace;
  font-size: 11px;
  background: var(--c-gray-100);
  padding: 1px 4px;
  border-radius: 3px;
}
</style>
