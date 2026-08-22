<template>
  <section class="content-frame">
    <header class="content-toolbar">
      <section class="toolbar-dicts">
        <MainDictsToolbar />
      </section>
      <nav class="toolbar-actions">
        <button class="toolbar-btn" type="button" title="导出当前词条 HTML" @click="onExportEntry">
          <NIcon><Bug16Regular /></NIcon>
        </button>
        <button class="toolbar-btn" type="button" title="编辑词典 CSS" @click="onEditCSS">
          <NIcon><DocumentCss20Regular /></NIcon>
        </button>
        <button class="toolbar-btn" type="button" title="刷新" @click="refresh">
          <NIcon><ArrowClockwise20Filled /></NIcon>
        </button>
        <button
          class="toolbar-btn"
          type="button"
          :title="`缩小（当前 ${contentZoom}%）`"
          :disabled="contentZoom <= MIN_CONTENT_ZOOM"
          @click="zoomOut"
        ><NIcon><ZoomOut16Regular /></NIcon></button>
        <button
          class="toolbar-btn"
          type="button"
          :title="`放大（当前 ${contentZoom}%）`"
          :disabled="contentZoom >= MAX_CONTENT_ZOOM"
          @click="zoomIn"
        ><NIcon><ZoomIn16Regular /></NIcon></button>
      </nav>
    </header>

    <section id="app-content-main-iframe-wrapper" class="iframe-wrapper" ref="wrapperRef">
      <section v-if="showMulti" class="multi-stack">
        <MainDictSection
          v-for="(r, i) in dictQueryStore.multiResults"
          :ref="el => { if (el) sectionRefs[i] = el as any; }"
          :key="r.dictId"
          :name="r.dictName"
          :url="r.url"
          :empty="r.empty"
          :zoom="contentZoom"
        />
      </section>
      <iframe
        v-else
        ref="iframeRef"
        class="content-iframe"
        :src="iframeSrc"
        frameborder="0"
        style="border: 0;"
        @load="onIframeLoad"
      ></iframe>
    </section>
  </section>
</template>

<script lang="ts" setup>
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useDictQueryStore } from '@/store/dict';
import { ZoomIn16Regular, ZoomOut16Regular, ArrowClockwise20Filled, Bug16Regular, DocumentCss20Regular } from '@vicons/fluent';
import { NIcon } from 'naive-ui';
import { useMessage } from 'naive-ui';
import { EventsOn } from '../../../wailsjs/runtime/runtime';
import { exportCurrentEntry, openDictCSSWindow } from '@/apis/dicts-api';
import { getPreferences, savePreferences } from '@/apis/config';
import { DEFAULT_CONTENT_ZOOM, MAX_CONTENT_ZOOM, MIN_CONTENT_ZOOM, adjustContentZoom, normalizeContentZoom } from './content-zoom';
import MainDictsToolbar from './MainDictsToolbar.vue';
import MainDictSection from './MainDictSection.vue';

const dictQueryStore = useDictQueryStore();
const message = useMessage();

const TOP_WIN_MSG_SET_ZOOM = '__Medict_TOP_WIN_MSG_EVTP_SET_ZOOM';
const TOP_WIN_MSG_REFRESH = '__Medict_TOP_WIN_MSG_EVTP_REFRESH';
const TOP_WIN_MSG_SETUP = '__Medict_TOP_WIN_MSG__EVTY_SETUP__';
const INNER_FRAME_MSG_ENTRY_JUMP = '__Medict_INNER_FRAME_MSG_EVTP_ENTRY_JUMP';
const INNER_FRAME_MSG_CLICK_LOOKUP = '__Medict_INNER_FRAME_MSG_EVTP_CLICK_LOOKUP';
const TOP_WIN_MSG_APPLY_USER_CSS = '__Medict_TOP_WIN_MSG_EVTP_APPLY_USER_CSS';
const CSS_EDITOR_CHANGED_EVENT = 'medict:css-editor-changed';

const iframeRef = ref<HTMLIFrameElement | null>(null);
const wrapperRef = ref<HTMLElement | null>(null);
const sectionRefs = ref<any[]>([]);
const contentZoom = ref(DEFAULT_CONTENT_ZOOM);
let zoomSaveTimer: number | null = null;
let stopCSSChangedListener: (() => void) | null = null;

const showMulti = computed(
  () => dictQueryStore.multiMode && dictQueryStore.multiResults.length > 0,
);

function broadcast(evtype: string, detail: Record<string, unknown> = {}) {
  const payload = { evtype, ts: new Date().getTime(), ...detail };
  if (showMulti.value) {
    for (const s of sectionRefs.value) s?.iframeRef?.contentWindow?.postMessage(payload, '*');
  } else {
    iframeRef.value?.contentWindow?.postMessage(payload, '*');
  }
}

const iframeSrc = computed(() => {
  if (dictQueryStore.mainContentURL) return dictQueryStore.mainContentURL;
  const decoded = b64DecodeUnicode(dictQueryStore.mainContent);
  return 'data:text/html;charset=utf-8;base64,' + btoa(unescape(encodeURIComponent(decoded)));
});

function onIframeLoad() {
  const win = iframeRef.value?.contentWindow;
  if (!win) return;
  win.postMessage({ evtype: TOP_WIN_MSG_SETUP } as any, '*');
  win.postMessage({ evtype: TOP_WIN_MSG_SET_ZOOM, scale: contentZoom.value, ts: Date.now() }, '*');
}

function isKnownIframeSource(source: MessageEventSource | null): boolean {
  if (iframeRef.value?.contentWindow === source) return true;
  return sectionRefs.value.some((section) => section?.iframeRef?.contentWindow === source);
}

function onInnerFrameMessage(e: MessageEvent) {
  if (!e?.data?.evtype) return;
  if (!isKnownIframeSource(e.source)) return;
  switch (e.data.evtype) {
    case INNER_FRAME_MSG_ENTRY_JUMP: {
      let keyWord = e.data.word.split('#')[0];
      dictQueryStore.updateInputSearchWord(keyWord);
      dictQueryStore.searchWord(keyWord);
      dictQueryStore.pushHistoryByEntryIDx(0);
      break;
    }
    case INNER_FRAME_MSG_CLICK_LOOKUP: {
      let keyWord = (e.data.word || '').split('#')[0];
      if (keyWord) {
        dictQueryStore.updateInputSearchWord(keyWord);
        dictQueryStore.searchWord(keyWord);
        dictQueryStore.pushHistoryByEntryIDx(0);
      }
      break;
    }
  }
}

async function onExportEntry() {
  const dict = dictQueryStore.selectDict;
  const word = dictQueryStore.inputSearchWord;
  if (!dict?.id || !word?.trim()) { message.warning('请先查一个词'); return; }
  try {
    const path = await exportCurrentEntry(dict.id, word.trim());
    if (path) message.success(`已导出：${path}`);
  } catch (e) {
    message.error((e as Error)?.message || '导出失败');
  }
}

async function onEditCSS() {
  const dict = dictQueryStore.selectDict;
  if (!dict?.id) { message.warning('请先选择一个词典'); return; }
  try {
    await openDictCSSWindow(dict.id, dict.name || '', dictQueryStore.inputSearchWord || '');
  } catch (e) {
    message.error((e as Error)?.message || '无法打开 CSS 编辑器');
  }
}

function applyUserCSS(css: string) {
  const payload = { evtype: TOP_WIN_MSG_APPLY_USER_CSS, ts: Date.now(), css };
  if (showMulti.value) {
    for (const s of sectionRefs.value) s?.iframeRef?.contentWindow?.postMessage(payload, '*');
  } else {
    iframeRef.value?.contentWindow?.postMessage(payload, '*');
  }
}

function refresh() { broadcast(TOP_WIN_MSG_REFRESH); }
function zoomOut() { setContentZoom(adjustContentZoom(contentZoom.value, -1)); }
function zoomIn() { setContentZoom(adjustContentZoom(contentZoom.value, 1)); }

function setContentZoom(value: number) {
  const next = normalizeContentZoom(value);
  if (next === contentZoom.value) return;
  contentZoom.value = next;
  broadcast(TOP_WIN_MSG_SET_ZOOM, { scale: next });
  if (zoomSaveTimer !== null) window.clearTimeout(zoomSaveTimer);
  zoomSaveTimer = window.setTimeout(() => {
    savePreferences({ dictionarycontentzoom: next }).catch(() => message.warning('缩放比例未能保存'));
    zoomSaveTimer = null;
  }, 250);
}

function onZoomKey(e: KeyboardEvent) {
  if (!(e.ctrlKey || e.metaKey)) return;
  if (e.key === '=' || e.key === '+') { e.preventDefault(); zoomIn(); }
  else if (e.key === '-' || e.key === '_') { e.preventDefault(); zoomOut(); }
}

function onIframeWheel(e: WheelEvent) {
  if (!(e.ctrlKey || e.metaKey)) return;
  e.preventDefault();
  if (e.deltaY < 0) zoomIn(); else zoomOut();
}

function b64DecodeUnicode(str: string) {
  return decodeURIComponent(
    atob(str).split('').map((c) => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2)).join(''),
  );
}

onMounted(() => {
  stopCSSChangedListener = EventsOn(CSS_EDITOR_CHANGED_EVENT, (dictID: string, css: string) => {
    if (dictQueryStore.selectDict?.id === dictID) applyUserCSS(css);
  });
  window.addEventListener('message', onInnerFrameMessage);
  window.addEventListener('keydown', onZoomKey);
  wrapperRef.value?.addEventListener('wheel', onIframeWheel, { passive: false });
  setTimeout(() => dictQueryStore.setUpAPIBaseURL(), 1000);
  getPreferences().then((preferences) => {
    contentZoom.value = normalizeContentZoom(preferences.dictionarycontentzoom);
    broadcast(TOP_WIN_MSG_SET_ZOOM, { scale: contentZoom.value });
  }).catch(() => { contentZoom.value = DEFAULT_CONTENT_ZOOM; });
});

onUnmounted(() => {
  stopCSSChangedListener?.();
  stopCSSChangedListener = null;
  window.removeEventListener('message', onInnerFrameMessage);
  window.removeEventListener('keydown', onZoomKey);
  wrapperRef.value?.removeEventListener('wheel', onIframeWheel);
  if (zoomSaveTimer !== null) window.clearTimeout(zoomSaveTimer);
});
</script>

<style lang="scss" scoped>
@use '@/style/variables.scss' as *;

.content-frame {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.content-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 32px;
  flex-shrink: 0;
  background: var(--c-gray-50);
  border-bottom: 1px solid var(--c-gray-200);
  padding: 0 4px 0 0;
}

.toolbar-dicts {
  flex: 1;
  min-width: 0;
  height: 100%;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 100%;
  flex-shrink: 0;
}

.toolbar-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  padding: 0;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: var(--c-gray-600);
  cursor: pointer;
  transition: background 0.12s, color 0.12s;

  &:hover {
    background: var(--c-gray-200);
    color: var(--c-gray-900);
  }

  &:disabled {
    color: var(--c-gray-300);
    cursor: not-allowed;
    &:hover { background: transparent; }
  }
}

.iframe-wrapper {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  padding: 0;
  position: relative;
}

.multi-stack {
  height: 100%;
  overflow-y: auto;

  &::-webkit-scrollbar { width: 6px; }
  &::-webkit-scrollbar-track { background: transparent; }
  &::-webkit-scrollbar-thumb {
    background: var(--c-gray-300);
    border-radius: 3px;
    &:hover { background: var(--c-gray-400); }
  }
  scrollbar-width: thin;
  scrollbar-color: var(--c-gray-300) transparent;
}

.content-iframe {
  width: 100%;
  height: 100%;
  display: block;
}
</style>
