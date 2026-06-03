<script setup lang="ts">
import { computed, ref } from 'vue'
import { useConfig } from '../../composables/useConfig'
import { isImageFieldDefault } from '../../types/config'
import RevertButton from './RevertButton.vue'

const { config, setImageProp, resetImageField } = useConfig()
const widthWarn = ref('')

const widthModel = computed({
  get: () => config.image.width,
  set: (v: number) => {
    if (v > 800) {
      setImageProp('width', 40)
      widthWarn.value = '宽度不能超过800，已重置为40'
      setTimeout(() => { widthWarn.value = '' }, 3000)
      return
    }
    widthWarn.value = ''
    setImageProp('width', Math.max(1, v))
  },
})
const heightModel = computed({
  get: () => config.image.height,
  set: (v: number) => setImageProp('height', Math.max(1, Math.min(65535, v))),
})
const filenameModel = computed({
  get: () => config.image.filename,
  set: (v: string) => setImageProp('filename', v),
})

const tipKey = ref('')
const tipText = ref('')
function showTip(key: string, text: string) { tipKey.value = key; tipText.value = text }
function hideTip() { tipKey.value = ''; tipText.value = '' }
</script>

<template>
  <section class="config-section">
    <h3 class="section-title">
      <svg width="14" height="14" viewBox="0 0 14 14" class="section-icon-svg">
        <rect x="1" y="1" width="12" height="12" rx="2" fill="none" stroke="currentColor" stroke-width="1.2"/>
        <line x1="1" y1="5" x2="13" y2="5" stroke="currentColor" stroke-width="0.6" opacity="0.5"/>
        <line x1="1" y1="9" x2="13" y2="9" stroke="currentColor" stroke-width="0.6" opacity="0.5"/>
      </svg>
      图片尺寸
    </h3>
    <div class="field-grid">
      <div class="field">
        <div class="label-wrap">
          <div class="label-row">
            <label class="field-label">
              宽度 <span class="unit">px</span>
              <span class="help-icon" @mouseenter="showTip('width', 'lazer必须设置为skin.ini里ColumnWidth值的1.6倍才不会变形。')" @mouseleave="hideTip">
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><circle cx="6" cy="6" r="5" stroke="currentColor" stroke-width="1"/><path d="M5.4 5.4a.6.6 0 0 1 1.2 0c0 .66-.6.9-.6 1.5v.3h.6v-.3c0-.66.6-.9.6-1.5a1.2 1.2 0 1 0-2.4 0h.6ZM5.4 9h1.2v-1.2h-1.2z" fill="currentColor"/></svg>
              </span>
            </label>
            <RevertButton :visible="!isImageFieldDefault(config, 'width')" @revert="resetImageField('width')" />
          </div>
          <div v-if="tipKey === 'width'" class="help-tip-banner">{{ tipText }}</div>
        </div>
        <div class="input-wrap">
          <input
            v-model.number="widthModel"
            type="number"
            min="1"
            max="800"
            class="num-input"
          />
        </div>
        <p v-if="widthWarn" class="warn">{{ widthWarn }}</p>
      </div>
      <div class="field">
        <div class="label-wrap">
          <div class="label-row">
            <label class="field-label">
              高度 <span class="unit">px</span>
              <span class="help-icon" @mouseenter="showTip('height', 'lazer推荐32800，否则可能导致尾部变形。stable推荐32767。')" @mouseleave="hideTip">
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><circle cx="6" cy="6" r="5" stroke="currentColor" stroke-width="1"/><path d="M5.4 5.4a.6.6 0 0 1 1.2 0c0 .66-.6.9-.6 1.5v.3h.6v-.3c0-.66.6-.9.6-1.5a1.2 1.2 0 1 0-2.4 0h.6ZM5.4 9h1.2v-1.2h-1.2z" fill="currentColor"/></svg>
              </span>
            </label>
            <RevertButton :visible="!isImageFieldDefault(config, 'height')" @revert="resetImageField('height')" />
          </div>
          <div v-if="tipKey === 'height'" class="help-tip-banner">{{ tipText }}</div>
        </div>
        <div class="input-wrap">
          <input
            v-model.number="heightModel"
            type="number"
            min="1"
            max="65535"
            class="num-input"
          />
        </div>
      </div>
    </div>
    <div class="field">
      <div class="label-wrap">
        <div class="label-row">
          <label class="field-label">
            图片名称
            <span class="help-icon" @mouseenter="showTip('filename', '因lazer加载问题对于此图片来说2x反而不如1x清晰。')" @mouseleave="hideTip">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><circle cx="6" cy="6" r="5" stroke="currentColor" stroke-width="1"/><path d="M5.4 5.4a.6.6 0 0 1 1.2 0c0 .66-.6.9-.6 1.5v.3h.6v-.3c0-.66.6-.9.6-1.5a1.2 1.2 0 1 0-2.4 0h.6ZM5.4 9h1.2v-1.2h-1.2z" fill="currentColor"/></svg>
            </span>
          </label>
          <RevertButton :visible="!isImageFieldDefault(config, 'filename')" @revert="resetImageField('filename')" />
        </div>
        <div v-if="tipKey === 'filename'" class="help-tip-banner">{{ tipText }}</div>
      </div>
      <div class="input-wrap input-with-suffix">
        <input
          v-model="filenameModel"
          type="text"
          class="text-input"
          placeholder="mania-note1H"
          spellcheck="false"
        />
        <span class="input-suffix">.png</span>
      </div>
    </div>
  </section>
</template>

<style scoped>
.section-icon-svg {
  color: var(--accent-purple);
  flex-shrink: 0;
}
.field-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}
.label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}
.input-with-suffix {
  position: relative;
}
.input-with-suffix .text-input {
  padding-right: 42px;
}
.warn { color: var(--error); font-size: 11px; margin-top: 4px }
.input-suffix {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  pointer-events: none;
  background: var(--bg-input);
  padding-left: 4px;
}
.help-icon { display: inline-flex; align-items: center; justify-content: center; width: 14px; height: 14px; border-radius: 50%; color: var(--text-muted); cursor: help; vertical-align: middle; margin-left: 3px; transition: color .15s }
.help-icon:hover { color: var(--accent-purple) }
.label-wrap { position: relative; }
.help-tip-banner {
  position: absolute;
  bottom: 100%;
  left: -30px;
  right: -30px;
  margin-bottom: 2px;
  padding: 4px 14px;
  background: rgba(15,17,29,0.97);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 11px;
  line-height: 1.55;
  color: var(--text-primary);
  text-align: left;
  white-space: normal;
  word-break: break-all;
  z-index: 100;
  box-shadow: 0 4px 16px rgba(0,0,0,0.5);
  pointer-events: none;
}
</style>
