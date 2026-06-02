import { ref, reactive, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { TailConfig, Preset } from '../types/config'
import { createDefaultConfig, hexToRgba, rgbaToHex } from '../types/config'

// 全局单例状态
const config = reactive<TailConfig>(createDefaultConfig())
const presets = ref<Preset[]>([])
const previewBase64 = ref<string>('')
const previewLoading = ref(false)
const validationErrors = ref<string[]>([])
const dirty = ref(false)

// 防抖预览定时器
let previewTimer: ReturnType<typeof setTimeout> | null = null

export function useConfig() {
  // --- 初始化 ---
  async function init() {
    try {
      const [defaultCfg, presetList] = await Promise.all([
        invoke<TailConfig>('get_default_config'),
        invoke<Preset[]>('get_presets'),
      ])
      Object.assign(config, defaultCfg)
      presets.value = presetList
      await updatePreview()
    } catch (e) {
      console.error('初始化失败:', e)
    }
  }

  // --- 预览 ---
  async function updatePreview() {
    dirty.value = true
    if (previewTimer) clearTimeout(previewTimer)
    previewTimer = setTimeout(async () => {
      previewLoading.value = true
      try {
        const b64 = await invoke<string>('render_preview', {
          config: JSON.parse(JSON.stringify(config)),
        })
        previewBase64.value = b64
        validationErrors.value = []
      } catch (e) {
        validationErrors.value = [String(e)]
        console.error('预览渲染失败:', e)
      } finally {
        previewLoading.value = false
      }
    }, 150)
  }

  // 参数修改时自动触发预览
  watch(
    () => JSON.parse(JSON.stringify(config)),
    () => { updatePreview() },
    { deep: true }
  )

  // --- 导出 ---
  async function exportImage(outputPath: string): Promise<void> {
    await invoke('export_image', {
      config: JSON.parse(JSON.stringify(config)),
      outputPath,
    })
  }

  // --- 预设 ---
  async function loadPreset(preset: Preset) {
    Object.assign(config, JSON.parse(JSON.stringify(preset.config)))
    await updatePreview()
  }

  async function savePreset(name: string) {
    const newPreset: Preset = {
      name,
      config: JSON.parse(JSON.stringify(config)),
      builtin: false,
    }
    const existing = presets.value.findIndex(p => p.name === name)
    if (existing >= 0) {
      if (presets.value[existing].builtin) {
        throw new Error(`不能覆盖内置预设 "${name}"`)
      }
      presets.value[existing] = newPreset
    } else {
      presets.value.push(newPreset)
    }
    await persistUserPresets()
  }

  async function deletePreset(name: string) {
    const idx = presets.value.findIndex(p => p.name === name)
    if (idx < 0) return
    if (presets.value[idx].builtin) {
      throw new Error(`不能删除内置预设 "${name}"`)
    }
    presets.value.splice(idx, 1)
    await persistUserPresets()
  }

  function resetConfig() {
    Object.assign(config, createDefaultConfig())
    updatePreview()
  }

  // 持久化用户预设到 app data 目录
  async function persistUserPresets() {
    const userPresets = presets.value.filter(p => !p.builtin)
    await invoke('save_user_presets', { presets: userPresets })
  }

  // init 中 get_presets 已返回合并后的全部预设，无需额外加载
  function loadUserPresets() {
    // 预设已在 init() 中通过 get_presets 加载完毕
  }

  // --- 便捷更新方法 ---
  function setImageProp<K extends keyof TailConfig['image']>(key: K, value: TailConfig['image'][K]) {
    ;(config.image as any)[key] = value
  }

  function setCapProp<K extends keyof TailConfig['cap']>(key: K, value: TailConfig['cap'][K]) {
    ;(config.cap as any)[key] = value
  }

  function setBodyProp<K extends keyof TailConfig['body']>(key: K, value: TailConfig['body'][K]) {
    ;(config.body as any)[key] = value
  }

  function setCapColorFromHex(hex: string) {
    config.cap.color = hexToRgba(hex, config.cap.color.a)
  }
  function setBodyFillColorFromHex(hex: string) {
    config.body.fillColor = hexToRgba(hex, config.body.fillColor.a)
  }
  function setBodyBorderColorFromHex(hex: string) {
    config.body.borderColor = hexToRgba(hex, config.body.borderColor.a)
  }

  return {
    config,
    presets,
    previewBase64,
    previewLoading,
    validationErrors,
    dirty,
    // methods
    init,
    updatePreview,
    exportImage,
    loadPreset,
    savePreset,
    deletePreset,
    resetConfig,
    loadUserPresets,
    setImageProp,
    setCapProp,
    setBodyProp,
    setCapColorFromHex,
    setBodyFillColorFromHex,
    setBodyBorderColorFromHex,
    // helpers
    rgbaToHex,
    hexToRgba,
  }
}
