<!-- 多词典堆叠模式下一节：词典名标题 + 单个释义 iframe。供 MainContentFrame v-for 使用。 -->
<style lang="scss" scoped>
.dict-section {
  margin: 0 0 10px;
  border: 1px solid var(--c-gray-200);
  border-radius: 8px;
  overflow: hidden;
  background: #fff;
}

.dict-section-head {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 28px;
  padding: 0 12px;
  background: var(--c-gray-50);
  border-bottom: 1px solid var(--c-gray-200);
  font-size: 12px;
  color: var(--c-gray-700);
  user-select: none;
}

.dict-section-name {
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dict-section-tag {
  color: var(--c-gray-400);
  font-size: 11px;
}

.dict-section-body {
  height: 320px;
  background: #fff;
  position: relative;
}

.dict-section-iframe {
  width: 100%;
  height: 100%;
  display: block;
}

.dict-section-empty {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--c-gray-400);
  font-size: 13px;
}
</style>

<template>
  <article class="dict-section">
    <header class="dict-section-head">
      <span class="dict-section-name">{{ name }}</span>
      <span v-if="empty" class="dict-section-tag">无此词</span>
    </header>
    <section class="dict-section-body">
      <p v-if="empty" class="dict-section-empty">该词典无此词</p>
      <iframe
        v-else
        ref="iframeRef"
        class="dict-section-iframe"
        :src="url"
        frameborder="0"
        style="border: 0;"
        @load="onLoad"
      ></iframe>
    </section>
  </article>
</template>

<script lang="ts" setup>
import { ref } from 'vue';

const SETUP_MSG = '__Medict_TOP_WIN_MSG__EVTY_SETUP__';

const props = defineProps<{ name: string; url: string; empty: boolean; zoom: number; }>();
const iframeRef = ref<HTMLIFrameElement | null>(null);

function onLoad() {
  iframeRef.value?.contentWindow?.postMessage({ evtype: SETUP_MSG }, '*');
  iframeRef.value?.contentWindow?.postMessage({
    evtype: '__Medict_TOP_WIN_MSG_EVTP_SET_ZOOM',
    scale: props.zoom,
    ts: Date.now(),
  }, '*');
}

defineExpose({ iframeRef });
</script>
