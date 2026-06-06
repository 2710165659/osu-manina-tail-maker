use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TailConfig {
    pub image: ImageConfig,
    pub margin: u32,
    pub throw_length: u32,
    pub global_color: RgbaColor,
    pub cap: CapConfig,
    pub body: BodyConfig,
    pub effect: EffectConfig,
    pub global_opacity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageConfig {
    pub width: u32,
    pub height: u32,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapConfig {
    pub shape: CapShape,
    pub scale: u32,
    pub independent_settings: bool,
    pub color: RgbaColor,
    pub opacity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CapShape {
    #[serde(rename = "ball")]
    Ball,
    #[serde(rename = "diamond")]
    Diamond,
    #[serde(rename = "rect")]
    Rect,
    #[serde(rename = "gradient")]
    Gradient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyConfig {
    pub independent_settings: bool,
    pub color: RgbaColor,
    pub opacity: u8,
    pub border_enabled: bool,
    pub border_color: RgbaColor,
    pub border_opacity: u8,
    #[serde(default)]
    pub border_opacity_independent: bool,
    #[serde(default)]
    pub border_match_body: bool,
    pub border_width: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectConfig {
    /// 顶端暗化重复开关
    pub cap_echo_enabled: bool,
    /// 重复颜色
    pub echo_color: RgbaColor,
    /// 重复透明度
    pub echo_opacity: u8,
    /// 重复长度
    pub echo_length: u32,
    /// 外发光开关
    pub glow_enabled: bool,
    /// 发光颜色
    pub glow_color: RgbaColor,
    /// 发光透明度
    pub glow_opacity: u8,
    /// 发光偏移X
    pub glow_dx: i32,
    /// 发光偏移Y
    pub glow_dy: i32,
    /// 发光大小（模糊半径）
    pub glow_size: u32,
    /// 发光扩展（膨胀半径）
    pub glow_spread: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const GREY: Self = Self { r: 113, g: 113, b: 113, a: 255 };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub name: String,
    pub config: TailConfig,
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl TailConfig {
    pub fn default_config() -> Self {
        Self {
            image: ImageConfig {
                width: 100,
                height: 32800,
                filename: "mania-noteL".to_string(),
            },
            margin: 8,
            throw_length: 100,
            global_color: RgbaColor::GREY,
            cap: CapConfig {
                shape: CapShape::Ball,
                scale: 100,
                independent_settings: false,
                color: RgbaColor::GREY,
                opacity: 255,
            },
            body: BodyConfig {
                independent_settings: false,
                color: RgbaColor::GREY,
                opacity: 255,
                border_enabled: false,
                border_color: RgbaColor::WHITE,
                border_opacity: 255,
                border_opacity_independent: false,
                border_match_body: false,
                border_width: 1,
            },
            effect: EffectConfig {
                cap_echo_enabled: false,
                echo_color: RgbaColor::GREY,
                echo_opacity: 76,
                echo_length: 50,
                glow_enabled: false,
                glow_color: RgbaColor { r: 144, g: 238, b: 144, a: 255 },
                glow_opacity: 180,
                glow_dx: 0,
                glow_dy: 0,
                glow_size: 8,
                glow_spread: 4,
            },
            global_opacity: 255,
        }
    }

    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        if self.image.width == 0 {
            errors.push("图片宽度必须大于 0".to_string());
        }
        if self.image.width > 800 {
            errors.push("图片宽度不能超过 800".to_string());
        }
        if self.image.height == 0 {
            errors.push("图片高度必须大于 0".to_string());
        }
        if self.image.height > 65535 {
            errors.push("图片高度不能超过 65535".to_string());
        }
        if self.margin * 2 >= self.image.width {
            errors.push(format!(
                "留白 ({}) × 2 必须小于图片宽度 ({})", self.margin, self.image.width
            ));
        }
        let cap_h = self.cap_height();
        let echo_enabled = self.effect.cap_echo_enabled && self.cap.shape != CapShape::Gradient;
        let echo_total_h = if echo_enabled { cap_h + self.effect.echo_length } else { 0 };
        if self.throw_length + echo_total_h + cap_h >= self.image.height {
            errors.push(format!(
                "投的长度 ({}) + 暗化重复总高度 ({}) + 顶端高度 ({}) 必须小于图片高度 ({})",
                self.throw_length, echo_total_h, cap_h, self.image.height
            ));
        }
        ValidationResult { valid: errors.is_empty(), errors }
    }

    /// 内容区宽度
    pub fn content_width(&self) -> u32 {
        self.image.width.saturating_sub(self.margin * 2)
    }

    /// 顶端渲染高度 = scale × content_width / 200（渐变形状基准翻倍）
    pub fn cap_height(&self) -> u32 {
        let divisor = if self.cap.shape == CapShape::Gradient { 100 } else { 200 };
        (self.cap.scale as u64 * self.content_width() as u64 / divisor) as u32
    }

    /// 身体高度
    pub fn body_height(&self) -> u32 {
        let echo_enabled = self.effect.cap_echo_enabled && self.cap.shape != CapShape::Gradient;
        let echo_total_h = if echo_enabled { self.cap_height() + self.effect.echo_length } else { 0 };
        self.image.height.saturating_sub(self.throw_length).saturating_sub(echo_total_h).saturating_sub(self.cap_height())
    }
}
