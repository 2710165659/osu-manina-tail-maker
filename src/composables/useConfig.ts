import { ref, reactive, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { TailConfig, Preset, CapConfig, BodyConfig, ImageConfig, EffectConfig } from '../types/config'
import { createDefaultConfig, hexToRgba, rgbaToHex } from '../types/config'

// 全局单例状态
const config = reactive<TailConfig>(createDefaultConfig())
const presets = ref<Preset[]>([])
const previewBase64 = ref<string>('')
const previewLoading = ref(false)
const validationErrors = ref<string[]>([])
const dirty = ref(false)
const currentPreset = ref<Preset | null>(null)

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
      presets.value = presetList
      // 默认加载第一个内置预设
      const firstBuiltin = presetList.find(p => p.builtin)
      if (firstBuiltin) {
        currentPreset.value = JSON.parse(JSON.stringify(firstBuiltin))
        Object.assign(config, JSON.parse(JSON.stringify(firstBuiltin.config)))
      } else {
        Object.assign(config, defaultCfg)
      }
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
    currentPreset.value = JSON.parse(JSON.stringify(preset))
    Object.assign(config, JSON.parse(JSON.stringify(preset.config)))
    await updatePreview()
  }

  async function savePreset(name: string, presetConfig?: TailConfig) {
    const newConfig = JSON.parse(JSON.stringify(presetConfig || config))
    const idx = presets.value.findIndex(p => p.name === name)

    if (idx >= 0) {
      // 覆盖已有预设（UI 层已做二次确认）
      presets.value[idx].config = newConfig
      // 更新当前预设引用
      if (currentPreset.value?.name === name) {
        currentPreset.value = JSON.parse(JSON.stringify(presets.value[idx]))
      }
    } else {
      // 新增预设
      const newPreset: Preset = {
        name,
        config: newConfig,
        builtin: false,
      }
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
    // 如果删除的是当前预设，清除引用
    if (currentPreset.value?.name === name) {
      currentPreset.value = null
    }
    await persistUserPresets()
  }

  function resetConfig() {
    if (currentPreset.value) {
      // 有当前预设时，重置到该预设的值
      Object.assign(config, JSON.parse(JSON.stringify(currentPreset.value.config)))
    } else {
      // 没有当前预设时，重置到全局默认值
      Object.assign(config, createDefaultConfig())
    }
    updatePreview()
  }

  /** 重置到全局默认值（忽略当前预设） */
  function resetToGlobalDefault() {
    currentPreset.value = null
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

  function setEffectProp<K extends keyof TailConfig['effect']>(key: K, value: TailConfig['effect'][K]) {
    ;(config.effect as any)[key] = value
  }

  function setCapColorFromHex(hex: string) {
    config.cap.color = hexToRgba(hex, config.cap.color.a)
  }
  function setBodyColorFromHex(hex: string) {
    config.body.color = hexToRgba(hex, config.body.color.a)
  }
  function setBodyBorderColorFromHex(hex: string) {
    config.body.borderColor = hexToRgba(hex, config.body.borderColor.a)
  }

  /** 获取参考配置（当前预设或全局默认） */
  function getRefConfig(): TailConfig {
    return currentPreset.value
      ? JSON.parse(JSON.stringify(currentPreset.value.config))
      : createDefaultConfig()
  }

  /** 判断顶层字段是否与预设一致 */
  function isFieldDefault<K extends keyof TailConfig>(field: K): boolean {
    const ref = getRefConfig()
    return JSON.stringify(config[field]) === JSON.stringify(ref[field])
  }

  /** 判断 cap 子字段是否与预设一致 */
  function isCapFieldDefault<K extends keyof CapConfig>(field: K): boolean {
    const ref = getRefConfig()
    return JSON.stringify(config.cap[field]) === JSON.stringify(ref.cap[field])
  }

  /** 判断 body 子字段是否与预设一致 */
  function isBodyFieldDefault<K extends keyof BodyConfig>(field: K): boolean {
    const ref = getRefConfig()
    return JSON.stringify(config.body[field]) === JSON.stringify(ref.body[field])
  }

  /** 判断 effect 子字段是否与预设一致 */
  function isEffectFieldDefault<K extends keyof EffectConfig>(field: K): boolean {
    const ref = getRefConfig()
    return JSON.stringify(config.effect[field]) === JSON.stringify(ref.effect[field])
  }

  /** 判断 image 子字段是否与预设一致 */
  function isImageFieldDefault<K extends keyof ImageConfig>(field: K): boolean {
    const ref = getRefConfig()
    return JSON.stringify(config.image[field]) === JSON.stringify(ref.image[field])
  }

  /** 恢复顶层字段为预设值 */
  function resetField<K extends keyof TailConfig>(field: K) {
    const ref = getRefConfig()
    ;(config as any)[field] = JSON.parse(JSON.stringify(ref[field]))
  }

  /** 恢复 cap 子字段为预设值 */
  function resetCapField<K extends keyof CapConfig>(field: K) {
    const ref = getRefConfig()
    ;(config.cap as any)[field] = JSON.parse(JSON.stringify(ref.cap[field]))
  }

  /** 恢复 body 子字段为预设值 */
  function resetBodyField<K extends keyof BodyConfig>(field: K) {
    const ref = getRefConfig()
    ;(config.body as any)[field] = JSON.parse(JSON.stringify(ref.body[field]))
  }

  /** 恢复 effect 子字段为预设值 */
  function resetEffectField<K extends keyof EffectConfig>(field: K) {
    const ref = getRefConfig()
    ;(config.effect as any)[field] = JSON.parse(JSON.stringify(ref.effect[field]))
  }

  /** 恢复 image 子字段为预设值 */
  function resetImageField<K extends keyof ImageConfig>(field: K) {
    const ref = getRefConfig()
    ;(config.image as any)[field] = JSON.parse(JSON.stringify(ref.image[field]))
  }

  return {
    config,
    presets,
    previewBase64,
    previewLoading,
    validationErrors,
    dirty,
    currentPreset,
    // methods
    init,
    updatePreview,
    exportImage,
    loadPreset,
    savePreset,
    deletePreset,
    resetConfig,
    resetToGlobalDefault,
    loadUserPresets,
    setImageProp,
    setCapProp,
    setBodyProp,
    setEffectProp,
    setCapColorFromHex,
    setBodyColorFromHex,
    setBodyBorderColorFromHex,
    resetField,
    resetCapField,
    resetBodyField,
    resetImageField,
    resetEffectField,
    // field default checks
    isFieldDefault,
    isCapFieldDefault,
    isBodyFieldDefault,
    isEffectFieldDefault,
    isImageFieldDefault,
    // helpers
    rgbaToHex,
    hexToRgba,
  }
}
