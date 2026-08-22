<template>
  <SettingPage>

    <section class="settings-section">
      <h2 class="settings-section-title">词典目录</h2>

      <SettingItem title="词典文件位置">
        <template #desc>
          Medict 在此目录中扫描并加载 .mdx / .mdd / .ifo / .db 等词典文件。
          修改目录后需重启应用才能生效。
        </template>
        <section class="setting-path-row">
          <span class="setting-path-text">{{ dictDir || '读取中…' }}</span>
          <button class="btn btn-default" type="button" :disabled="!dictDir" @click="openDictDir">
            在资源管理器中打开
          </button>
        </section>
      </SettingItem>

      <SettingItem title="允许软链接（符号链接）">
        <template #desc>
          扫描词典目录时，是否跟踪通过符号链接指向的子目录或文件。
          关闭可避免意外加载不在词典目录内的文件。
        </template>
        <EditableSettingValue
          v-model="allowSymlinks"
          :options="booleanOptions"
          :on-save="saveValue('dictionaryallowsymlinks')"
        />
      </SettingItem>

      <SettingItem title="单组词典数量上限">
        <template #desc>
          每个词典分组最多同时加载的词典数量。
          数值越大，启动索引耗时越长；建议设为 5–20。
        </template>
        <EditableSettingValue
          v-model="groupLimit"
          input-type="number"
          :min="1"
          :on-save="saveValue('dictionarygroupmax')"
        />
      </SettingItem>
    </section>

    <section class="settings-section">
      <h2 class="settings-section-title">词典内容注入</h2>

      <SettingItem title="自定义 CSS / JS 模板">
        <template #desc>
          每次渲染词典释义页时，会在 &lt;head&gt; 中插入此模板内容。
          支持占位符：<code>#{DictName}</code> 词典名称、<code>#{DictID}</code> 词典 ID。
          常用于加载词典自带的样式表或脚本。
        </template>
        <EditableSettingValue
          v-model="presetContent"
          multiline
          :rows="6"
          :on-save="saveValue('dictionarypresetcontent')"
        />
      </SettingItem>
    </section>

  </SettingPage>
</template>

<script lang="ts" setup>
import { onMounted, ref } from 'vue';
import { BaseDictDirectory, OpenDirOrFile } from '@/apis/apis';
import { getPreferences, savePreferences } from '@/apis/config';
import EditableSettingValue from '@/components/setting/EditableSettingValue.vue';
import SettingItem from '@/components/setting/SettingItem.vue';
import SettingPage from '@/components/setting/SettingPage.vue';

const defaultPreset = `<link href="#{DictName}.css?dict_id=#{DictID}" rel="stylesheet" />\n<script async src="#{DictName}.js?dict_id=#{DictID}"><\/script>`;
const dictDir = ref('');
const allowSymlinks = ref('true');
const groupLimit = ref('10');
const presetContent = ref(defaultPreset);

const booleanOptions = [
  { label: '允许', value: 'true' },
  { label: '不允许', value: 'false' },
];

function saveValue(key: string) {
  return (value: string) => savePreferences({ [key]: value });
}

async function openDictDir() {
  if (dictDir.value) await OpenDirOrFile(dictDir.value);
}

onMounted(async () => {
  const [dir, preferences] = await Promise.all([BaseDictDirectory(), getPreferences()]);
  dictDir.value = dir;
  allowSymlinks.value = String(preferences.dictionaryallowsymlinks ?? 'true');
  groupLimit.value = String(preferences.dictionarygroupmax ?? '10');
  presetContent.value = String(preferences.dictionarypresetcontent ?? defaultPreset);
});
</script>

<style lang="scss" scoped>
.setting-path-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.setting-path-text {
  flex: 1;
  min-width: 0;
  color: var(--c-gray-700);
  font-size: 12px;
  font-family: ui-monospace, 'SF Mono', monospace;
  overflow-wrap: anywhere;
}

code {
  font-family: ui-monospace, 'SF Mono', monospace;
  font-size: 11px;
  background: var(--c-gray-100);
  padding: 1px 4px;
  border-radius: 3px;
  color: var(--c-gray-800);
}
</style>
