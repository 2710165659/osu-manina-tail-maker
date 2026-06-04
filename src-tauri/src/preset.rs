use crate::config::{BodyConfig, CapConfig, CapShape, EffectConfig, ImageConfig, Preset, RgbaColor, TailConfig};

pub fn builtin_presets() -> Vec<Preset> {
    vec![
        Preset {
            name: "球皮-标准".to_string(), builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 40, height: 32800, filename: "mania-noteL".to_string() },
                margin: 0, throw_length: 100, global_color: RgbaColor::WHITE,
                cap: CapConfig { shape: CapShape::Ball, scale: 100, independent_settings: false, color: RgbaColor::WHITE, opacity: 255 },
                body: BodyConfig { independent_settings: true, color: RgbaColor::WHITE, opacity: 200, border_enabled: true, border_color: RgbaColor::WHITE, border_opacity: 180, border_match_body: false, border_width: 1 },
                effect: EffectConfig { cap_echo_enabled: false, echo_color: RgbaColor::WHITE, echo_opacity: 76, echo_length: 50 },
                global_opacity: 255,
            },
        },
        Preset {
            name: "菱形-无边框".to_string(), builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 40, height: 32800, filename: "mania-noteL".to_string() },
                margin: 0, throw_length: 100, global_color: RgbaColor::WHITE,
                cap: CapConfig { shape: CapShape::Diamond, scale: 120, independent_settings: false, color: RgbaColor { r: 255, g: 107, b: 157, a: 255 }, opacity: 255 },
                body: BodyConfig { independent_settings: true, color: RgbaColor { r: 255, g: 107, b: 157, a: 255 }, opacity: 220, border_enabled: false, border_color: RgbaColor::BLACK, border_opacity: 255, border_match_body: false, border_width: 1 },
                effect: EffectConfig { cap_echo_enabled: false, echo_color: RgbaColor { r: 255, g: 107, b: 157, a: 255 }, echo_opacity: 76, echo_length: 50 },
                global_opacity: 255,
            },
        },
        Preset {
            name: "渐变-无边框".to_string(), builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 40, height: 32800, filename: "mania-noteL".to_string() },
                margin: 0, throw_length: 100, global_color: RgbaColor::WHITE,
                cap: CapConfig { shape: CapShape::Gradient, scale: 150, independent_settings: false, color: RgbaColor { r: 100, g: 200, b: 255, a: 255 }, opacity: 255 },
                body: BodyConfig { independent_settings: true, color: RgbaColor { r: 100, g: 200, b: 255, a: 255 }, opacity: 200, border_enabled: false, border_color: RgbaColor::BLACK, border_opacity: 255, border_match_body: false, border_width: 1 },
                effect: EffectConfig { cap_echo_enabled: false, echo_color: RgbaColor { r: 100, g: 200, b: 255, a: 255 }, echo_opacity: 76, echo_length: 50 },
                global_opacity: 255,
            },
        },
        Preset {
            name: "纯色-无头".to_string(), builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 40, height: 32800, filename: "mania-noteL".to_string() },
                margin: 0, throw_length: 100, global_color: RgbaColor::WHITE,
                cap: CapConfig { shape: CapShape::Rect, scale: 0, independent_settings: false, color: RgbaColor::WHITE, opacity: 255 },
                body: BodyConfig { independent_settings: false, color: RgbaColor::WHITE, opacity: 255, border_enabled: false, border_color: RgbaColor::BLACK, border_opacity: 255, border_match_body: false, border_width: 1 },
                effect: EffectConfig { cap_echo_enabled: false, echo_color: RgbaColor::WHITE, echo_opacity: 76, echo_length: 50 },
                global_opacity: 255,
            },
        },
        Preset {
            name: "纯色-带边框".to_string(), builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 40, height: 32800, filename: "mania-noteL".to_string() },
                margin: 0, throw_length: 100, global_color: RgbaColor::WHITE,
                cap: CapConfig { shape: CapShape::Rect, scale: 0, independent_settings: false, color: RgbaColor::WHITE, opacity: 255 },
                body: BodyConfig { independent_settings: true, color: RgbaColor::WHITE, opacity: 180, border_enabled: true, border_color: RgbaColor::WHITE, border_opacity: 255, border_match_body: false, border_width: 2 },
                effect: EffectConfig { cap_echo_enabled: false, echo_color: RgbaColor::WHITE, echo_opacity: 76, echo_length: 50 },
                global_opacity: 255,
            },
        },
    ]
}
