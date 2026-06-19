use crate::config::{BodyConfig, CapConfig, CapShape, EffectConfig, ImageConfig, Preset, RgbaColor, TailConfig};

/// 默认特效：无暗化、无发光
fn default_effect(echo_color: RgbaColor) -> EffectConfig {
    EffectConfig {
        cap_echo_enabled: false,
        echo_color,
        echo_opacity: 76,
        echo_length: 50,
        glow_enabled: false,
        glow_color: RgbaColor { r: 144, g: 238, b: 144, a: 255 },
        glow_opacity: 180,
        glow_size: 8,
        glow_spread: 4,
        glow_match_body: false,
        glow_opacity_independent: false,
    }
}

/// 默认身体：无边框，颜色跟随全局
fn default_body() -> BodyConfig {
    BodyConfig {
        independent_settings: false,
        color: RgbaColor::GREY,
        opacity: 255,
        border_enabled: false,
        border_color: RgbaColor::WHITE,
        border_opacity: 255,
        border_opacity_independent: false,
        border_match_body: false,
        border_width: 1,
    }
}

pub fn builtin_presets() -> Vec<Preset> {
    vec![
        // ── 球皮-标准 ──
        Preset {
            name: "球皮-标准".to_string(),
            builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 100, height: 32800, filename: "mania-noteL".to_string() },
                margin: 8,
                throw_length: 50,
                global_color: RgbaColor::GREY,
                cap: CapConfig {
                    shape: CapShape::Ball,
                    scale: 100,
                    independent_settings: false,
                    color: RgbaColor::GREY,
                    opacity: 255,
                },
                body: default_body(),
                effect: default_effect(RgbaColor::GREY),
                global_opacity: 255,
            },
        },
        // ── 菱形-标准 ──
        Preset {
            name: "菱形-标准".to_string(),
            builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 100, height: 32800, filename: "mania-noteL".to_string() },
                margin: 8,
                throw_length: 50,
                global_color: RgbaColor { r: 255, g: 226, b: 116, a: 255 },
                cap: CapConfig {
                    shape: CapShape::Diamond,
                    scale: 100,
                    independent_settings: false,
                    color: RgbaColor::GREY,
                    opacity: 255,
                },
                body: default_body(),
                effect: default_effect(RgbaColor::GREY),
                global_opacity: 179,
            },
        },
        // ── 渐变-标准 ──
        Preset {
            name: "渐变-标准".to_string(),
            builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 100, height: 32800, filename: "mania-noteL".to_string() },
                margin: 8,
                throw_length: 50,
                global_color: RgbaColor { r: 204, g: 252, b: 178, a: 255 },
                cap: CapConfig {
                    shape: CapShape::Gradient,
                    scale: 150,
                    independent_settings: false,
                    color: RgbaColor::GREY,
                    opacity: 255,
                },
                body: default_body(),
                effect: default_effect(RgbaColor::GREY),
                global_opacity: 178,
            },
        },
        // ── 渐变-边框 ──
        Preset {
            name: "渐变-边框".to_string(),
            builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 100, height: 32800, filename: "mania-noteL".to_string() },
                margin: 8,
                throw_length: 50,
                global_color: RgbaColor { r: 188, g: 219, b: 241, a: 255 },
                cap: CapConfig {
                    shape: CapShape::Gradient,
                    scale: 150,
                    independent_settings: false,
                    color: RgbaColor::GREY,
                    opacity: 255,
                },
                body: BodyConfig {
                    border_enabled: true,
                    border_color: RgbaColor::WHITE,
                    border_opacity: 255,
                    border_width: 5,
                    ..default_body()
                },
                effect: default_effect(RgbaColor::GREY),
                global_opacity: 178,
            },
        },
        // ── 球皮-暗化 ──
        Preset {
            name: "球皮-暗化".to_string(),
            builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 100, height: 32800, filename: "mania-noteL".to_string() },
                margin: 8,
                throw_length: 2,
                global_color: RgbaColor { r: 193, g: 114, b: 255, a: 255 },
                cap: CapConfig {
                    shape: CapShape::Ball,
                    scale: 100,
                    independent_settings: false,
                    color: RgbaColor::GREY,
                    opacity: 255,
                },
                body: BodyConfig {
                    border_enabled: true,
                    border_color: RgbaColor::WHITE,
                    border_opacity: 255,
                    border_width: 2,
                    ..default_body()
                },
                effect: EffectConfig {
                    cap_echo_enabled: true,
                    echo_color: RgbaColor { r: 193, g: 114, b: 255, a: 255 },
                    echo_opacity: 76,
                    echo_length: 50,
                    glow_enabled: false,
                    glow_color: RgbaColor { r: 144, g: 238, b: 144, a: 255 },
                    glow_opacity: 180,
                    glow_size: 8,
                    glow_spread: 4,
                    glow_match_body: false,
                    glow_opacity_independent: false,
                },
                global_opacity: 205,
            },
        },
    ]
}
