use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TailConfig {
    pub image: ImageConfig,
    pub margin: u32,
    /// 投的长度：图片顶部到第一个可见像素的透明距离
    pub throw_length: u32,
    pub cap: CapConfig,
    pub body: BodyConfig,
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
    /// 顶端缩放（默认 100 = 半圆；200 = 整圆高度；50 = 扁平）
    pub scale: u32,
    pub color: RgbaColor,
    pub independent_opacity: bool,
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
    pub fill_color: RgbaColor,
    pub fill_opacity: u8,
    pub border_enabled: bool,
    pub border_color: RgbaColor,
    pub border_opacity: u8,
    pub border_width: u32,
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
            cap: CapConfig {
                shape: CapShape::Ball,
                scale: 100,
                color: RgbaColor::GREY,
                independent_opacity: false,
                opacity: 255,
            },
            body: BodyConfig {
                fill_color: RgbaColor::GREY,
                fill_opacity: 255,
                border_enabled: false,
                border_color: RgbaColor::WHITE,
                border_opacity: 255,
                border_width: 1,
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
        if self.throw_length + cap_h >= self.image.height {
            errors.push(format!(
                "投的长度 ({}) + 顶端高度 ({}) 必须小于图片高度 ({})",
                self.throw_length, cap_h, self.image.height
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
        self.image.height.saturating_sub(self.throw_length).saturating_sub(self.cap_height())
    }

    /// Cap 起始 Y
    pub fn cap_start_y(&self) -> u32 { self.throw_length }

    /// Cap 结束 Y
    pub fn cap_end_y(&self) -> u32 { self.throw_length + self.cap_height() }
}
