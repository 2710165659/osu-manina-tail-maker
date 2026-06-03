use crate::config::{BodyConfig, CapConfig, CapShape, ImageConfig, Preset, RgbaColor, TailConfig};

pub fn builtin_presets() -> Vec<Preset> {
    vec![
        Preset {
            name: "球皮-标准".to_string(), builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 40, height: 32800, filename: "mania-noteL".to_string() },
                margin: 0, throw_length: 100,
                cap: CapConfig { shape: CapShape::Ball, scale: 100, color: RgbaColor::WHITE, independent_opacity: false, opacity: 255 },
                body: BodyConfig { independent_fill: false, fill_color: RgbaColor::WHITE, fill_opacity: 200, border_enabled: true, border_color: RgbaColor::WHITE, border_opacity: 180, border_width: 1 },
                global_opacity: 255,
            },
        },
        Preset {
            name: "菱形-无边框".to_string(), builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 40, height: 32800, filename: "mania-noteL".to_string() },
                margin: 0, throw_length: 100,
                cap: CapConfig { shape: CapShape::Diamond, scale: 120, color: RgbaColor { r: 255, g: 107, b: 157, a: 255 }, independent_opacity: false, opacity: 255 },
                body: BodyConfig { independent_fill: false, fill_color: RgbaColor { r: 255, g: 107, b: 157, a: 255 }, fill_opacity: 220, border_enabled: false, border_color: RgbaColor::BLACK, border_opacity: 255, border_width: 1 },
                global_opacity: 255,
            },
        },
        Preset {
            name: "渐变-无边框".to_string(), builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 40, height: 32800, filename: "mania-noteL".to_string() },
                margin: 0, throw_length: 100,
                cap: CapConfig { shape: CapShape::Gradient, scale: 150, color: RgbaColor { r: 100, g: 200, b: 255, a: 255 }, independent_opacity: false, opacity: 255 },
                body: BodyConfig { independent_fill: false, fill_color: RgbaColor { r: 100, g: 200, b: 255, a: 255 }, fill_opacity: 200, border_enabled: false, border_color: RgbaColor::BLACK, border_opacity: 255, border_width: 1 },
                global_opacity: 255,
            },
        },
        Preset {
            name: "纯色-无头".to_string(), builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 40, height: 32800, filename: "mania-noteL".to_string() },
                margin: 0, throw_length: 100,
                cap: CapConfig { shape: CapShape::Rect, scale: 0, color: RgbaColor::WHITE, independent_opacity: false, opacity: 255 },
                body: BodyConfig { independent_fill: false, fill_color: RgbaColor::WHITE, fill_opacity: 255, border_enabled: false, border_color: RgbaColor::BLACK, border_opacity: 255, border_width: 1 },
                global_opacity: 255,
            },
        },
        Preset {
            name: "纯色-带边框".to_string(), builtin: true,
            config: TailConfig {
                image: ImageConfig { width: 40, height: 32800, filename: "mania-noteL".to_string() },
                margin: 0, throw_length: 100,
                cap: CapConfig { shape: CapShape::Rect, scale: 0, color: RgbaColor::WHITE, independent_opacity: false, opacity: 255 },
                body: BodyConfig { independent_fill: false, fill_color: RgbaColor::WHITE, fill_opacity: 180, border_enabled: true, border_color: RgbaColor::WHITE, border_opacity: 255, border_width: 2 },
                global_opacity: 255,
            },
        },
    ]
}
