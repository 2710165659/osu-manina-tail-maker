export interface RgbaColor { r: number; g: number; b: number; a: number }

export type CapShape = 'ball' | 'diamond' | 'rect' | 'gradient'

export const CAP_SHAPE_LABELS: Record<CapShape, string> = {
  ball: '球皮',
  diamond: '菱形',
  rect: '矩形',
  gradient: '矩形渐变',
}
export const CAP_SHAPE_ORDER: CapShape[] = ['ball', 'diamond', 'rect', 'gradient']

export interface ImageConfig { width: number; height: number; filename: string }
export interface CapConfig { shape: CapShape; scale: number; independentSettings: boolean; color: RgbaColor; opacity: number }
export interface BodyConfig { independentSettings: boolean; color: RgbaColor; opacity: number; borderEnabled: boolean; borderColor: RgbaColor; borderOpacity: number; borderOpacityIndependent: boolean; borderMatchBody: boolean; borderWidth: number }
export interface EffectConfig { capEchoEnabled: boolean; echoColor: RgbaColor; echoOpacity: number; echoLength: number; glowEnabled: boolean; glowColor: RgbaColor; glowOpacity: number; glowSize: number; glowSpread: number; glowMatchBody: boolean; glowOpacityIndependent: boolean }
export interface TailConfig { image: ImageConfig; margin: number; throwLength: number; globalColor: RgbaColor; cap: CapConfig; body: BodyConfig; effect: EffectConfig; globalOpacity: number }
export interface Preset { name: string; config: TailConfig; builtin: boolean }
export interface ValidationResult { valid: boolean; errors: string[] }

export function createDefaultConfig(): TailConfig {
  const grey: RgbaColor = { r: 113, g: 113, b: 113, a: 255 }
  return {
    image: { width: 100, height: 32800, filename: 'mania-noteL' },
    margin: 8,
    throwLength: 100,
    globalColor: grey,
    cap: { shape: 'ball', scale: 100, independentSettings: false, color: grey, opacity: 255 },
    body: { independentSettings: false, color: grey, opacity: 255, borderEnabled: false, borderColor: { r: 255, g: 255, b: 255, a: 255 }, borderOpacity: 255, borderOpacityIndependent: false, borderMatchBody: false, borderWidth: 1 },
    effect: { capEchoEnabled: false, echoColor: grey, echoOpacity: 76, echoLength: 50, glowEnabled: false, glowColor: { r: 144, g: 238, b: 144, a: 255 }, glowOpacity: 180, glowSize: 8, glowSpread: 4, glowMatchBody: false, glowOpacityIndependent: false },
    globalOpacity: 255,
  }
}

const DEFAULT_CONFIG = createDefaultConfig()

/** 判断某个顶层字段是否为默认值 */
export function isFieldDefault(config: TailConfig, field: keyof TailConfig): boolean {
  return JSON.stringify(config[field]) === JSON.stringify(DEFAULT_CONFIG[field])
}

/** 获取某个顶层字段的默认值（深拷贝） */
export function getDefaultField(field: keyof TailConfig) {
  return JSON.parse(JSON.stringify(DEFAULT_CONFIG[field]))
}

/** 判断 cap 子字段是否为默认值 */
export function isCapFieldDefault(config: TailConfig, field: keyof CapConfig): boolean {
  return JSON.stringify(config.cap[field]) === JSON.stringify(DEFAULT_CONFIG.cap[field])
}

/** 判断 image 子字段是否为默认值 */
export function isImageFieldDefault(config: TailConfig, field: keyof ImageConfig): boolean {
  return JSON.stringify(config.image[field]) === JSON.stringify(DEFAULT_CONFIG.image[field])
}

export function rgbaToHex(c: RgbaColor): string {
  return `#${c.r.toString(16).padStart(2, '0')}${c.g.toString(16).padStart(2, '0')}${c.b.toString(16).padStart(2, '0')}`
}

export function hexToRgba(hex: string, a = 255): RgbaColor {
  const clean = hex.replace('#', '')
  return { r: parseInt(clean.substring(0, 2), 16), g: parseInt(clean.substring(2, 4), 16), b: parseInt(clean.substring(4, 6), 16), a }
}
