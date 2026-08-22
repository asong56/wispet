<style lang="scss">
@use './style/variables.scss' as *;
@use '@/style/photon/photon.scss';
@use '@/style/settings.scss';
@use '@/style/icons-svg.scss';

#app-root {
  height: 100%;
  width: 100%;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;

  .fake-title-bar {
    width: 100%;
    height: $fake-title-bar-height;
    flex-shrink: 0;
    --wails-draggable: drag;
    background-color: $theme-top-header-background-color;
  }

  .x-space-provider {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
}
</style>

<template>
  <article id="app-root" class="app-container">
    <header class="fake-title-bar" data-wails-drag></header>
    <n-config-provider
      :theme="theme"
      :locale="zhCN"
      :date-locale="dateZhCN"
      class="x-space-provider"
      :theme-overrides="themeOverrides"
    >
      <n-dialog-provider>
        <n-message-provider>
          <CSSWindow v-if="windowMode === 'css-editor'" />
          <router-view v-else-if="windowMode === 'main'"></router-view>
        </n-message-provider>
      </n-dialog-provider>
      <n-global-style />
    </n-config-provider>
  </article>
</template>

<script lang="ts" setup>
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import {
  NConfigProvider,
  NGlobalStyle,
  NDialogProvider,
  NMessageProvider,
} from 'naive-ui';
import { darkTheme as dark, lightTheme as light } from 'naive-ui';
import { zhCN, dateZhCN } from 'naive-ui';
import type { GlobalThemeOverrides } from 'naive-ui';
import { useDictQueryStore } from './store/dict';
import { BRAND, palette } from '@/style/tokens';
import CSSWindow from '@/view/css-editor/index.vue';

const isDark = ref(false);
const windowMode = ref<'loading' | 'main' | 'css-editor'>('loading');
let theme = reactive(light);

if (isDark.value) theme = dark;

const themeOverrides: GlobalThemeOverrides = {
  common: { primaryColor: BRAND },
  Input: {
    borderFocus: `1px solid ${BRAND}`,
    borderHover: `1px solid ${palette.primaryHover}`,
  },
  Button: {
    textColor: palette.gray[900],
    textColorHoverPrimary: BRAND,
    textColorPressedPrimary: BRAND,
    textColorFocusPrimary: BRAND,
    border: 'none',
    borderHover: 'none',
    borderPressed: 'none',
    borderFocus: 'none',
    borderDisabled: 'none',
  },
  Dialog: { iconSize: '0px' },
};

let unscribeDictQueryStore: (() => void) | null = null;
const dictQueryStore = useDictQueryStore();

onMounted(() => {
  const modeCall = (window as any)?.go?.main?.App?.WindowMode;
  if (typeof modeCall === 'function') {
    modeCall()
      .then((mode: string) => {
        windowMode.value = mode === 'css-editor' ? 'css-editor' : 'main';
      })
      .catch(() => {
        windowMode.value = 'main';
      });
  } else {
    windowMode.value = 'main';
  }
});

onUnmounted(() => {
  unscribeDictQueryStore?.();
  unscribeDictQueryStore = null;
});
</script>
