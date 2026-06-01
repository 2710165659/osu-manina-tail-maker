# 系统设计概要 (System Design)

> 版本：v1.0 | 日期：2026-06-01 | 对应需求 v1.0

---

## 1. 架构总览

```
┌──────────────────────────────────────────────────┐
│                  Tauri 桌面壳                      │
│  ┌─────────────────┐  ┌────────────────────────┐ │
│  │   Vue 前端       │  │    Rust 后端            │ │
│  │   (TypeScript)   │  │                        │ │
│  │                 │  │  ┌─ 参数模型 (Config)   │ │
│  │  ┌─ 参数面板    │  │  ├─ 渲染引擎 (Renderer) │ │
│  │  ├─ 预览区      │  │  ├─ PNG 编码器          │ │
│  │  ├─ 预设管理    │  │  ├─ 预设存储            │ │
│  │  └─ 导出触发    │  │  └─ 文件 I/O           │ │
│  └─────────────────┘  └────────────────────────┘ │
│         │                        │               │
│         └── Tauri Commands ──────┘               │
│              (invoke / event)                     │
└──────────────────────────────────────────────────┘
```

**通信方式**：
- 前端 → 后端：Tauri `invoke` 命令
- 后端 → 前端：Tauri Event（预览数据推送）

---

## 2. 技术选型

| 层 | 技术 | 理由 |
|----|------|------|
| 桌面框架 | Tauri 2.x | 轻量（不捆 Chromium），Rust 原生性能 |
| 前端 | Vue 3 + TypeScript | 响应式数据绑定，参数驱动 UI 天然契合 |
| 渲染 | Rust 自研 | 像素级精确控制，抗锯齿，大图（32800px）性能 |
| PNG 编码 | `image` crate (Rust) | 成熟的 PNG RGBA 编码 |
| 前端预览 | HTML5 Canvas | 接收缩略图数据，支持缩放/平移 |
| 样式 | Tailwind CSS 或朴素 CSS | 轻量，可定制 |

---

## 3. 核心数据模型 (Rust)

```rust
/// 整张投皮的完整参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailConfig {
    pub image: ImageConfig,
    pub margin: MarginConfig,
    pub throw_length: u32,       // 投的长度 (px)
    pub cap: CapConfig,
    pub body: BodyConfig,
    pub global_opacity: u8,      // 整体透明度 0-255
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub width: u32,              // 默认 40
    pub height: u32,             // 默认 32800
    pub filename: String,        // 导出文件名 (不含扩展名)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginConfig {
    pub margin: u32,             // 左右对称留白，各侧的 px 值
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapConfig {
    pub shape: CapShape,
    pub height: u32,             // 顶端高度 px
    pub color: RgbaColor,
    pub independent_opacity: bool,
    pub opacity: u8,             // 仅 independent_opacity=true 时生效
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapShape {
    Rect,                         // 矩形
    Ball,                         // 球皮（半圆）
    Diamond,                      // 菱形（半菱形）
    Gradient,                     // 透明度渐变
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyConfig {
    pub fill_color: RgbaColor,
    pub fill_opacity: u8,
    pub border_enabled: bool,
    pub border_color: RgbaColor,
    pub border_opacity: u8,
    pub border_width: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// 预设方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub config: TailConfig,
    pub builtin: bool,           // 内置预设不可删除
}
```

### 3.1 参数校验规则

| 约束 | 规则 |
|------|------|
| `margin * 2` | `< image.width`（至少 1px 内容区） |
| `throw_length + cap.height` | `< image.height`（至少 1px 身体） |
| `border_width` | `≤ content_width / 2` |
| opacity 系列 | 全部 `0..=255` |

---

## 4. 渲染引擎设计

### 4.1 渲染流程

```
TailConfig
    │
    ▼
┌──────────────────────┐
│ 1. 分配像素缓冲区      │  Vec<[u8;4]> 大小 = width × height
│    全部初始化为透明     │  [0, 0, 0, 0]
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│ 2. 计算内容区边界      │  content_left = margin
│                      │  content_right = width - margin
│                      │  cap_start_y = throw_length
│                      │  cap_end_y = throw_length + cap.height
│                      │  body_start_y = cap_end_y
│                      │  body_end_y = height (图片底部)
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│ 3. 绘制顶端 (Cap)     │  根据 CapShape 分支：
│                      │  Rect: 填满矩形
│                      │  Ball: 上半椭圆
│                      │  Diamond: 上半菱形
│                      │  Gradient: 逐行 alpha 插值
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│ 4. 绘制身体 (Body)    │  填充内容区矩形 + 可选边框
│  (body_start_y → 底) │
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│ 5. 应用整体透明度      │  所有非透明像素 × global_opacity / 255
└──────────┬───────────┘
           ▼
      RGBA Buffer
```

### 4.2 Cap 形状渲染算法

#### 球皮 (Ball) — 上半椭圆

```
内容区坐标系（局部）：
  cx = content_width / 2          ← 椭圆圆心 x
  cy = cap_height                  ← 椭圆圆心 y（圆心在 Cap 区域下方边界）
  rx = content_width / 2           ← 椭圆水平半径
  ry = cap_height                  ← 椭圆垂直半径

对 Cap 区域内每个像素 (x, y)：
  if ((x-cx)²/rx² + (y-cy)²/ry²) ≤ 1.0:
      填充 Cap 颜色
  else:
      透明
```

效果：Cap 区域上半部分是圆弧，圆心在 Cap 底部，椭圆刚好在 Cap 起始线处到顶。

#### 菱形 (Diamond) — 上半菱形

```
对 Cap 区域内每个像素 (x, y)，其中 y ∈ [0, cap_height)：
  row_max = (x 到内容区中心的距离对应的菱形边界)
  half_width_at_y = (content_width / 2) * (1 - y / cap_height)
  
  if |x - cx| ≤ half_width_at_y:
      填充 Cap 颜色
  else:
      透明
```

效果：菱形顶点在 Cap 起始线（y=0 处一点），向下逐渐扩宽，在 y=cap_height 处达到 content_width。

#### 渐变 (Gradient) — 透明度渐变

```
对 Cap 区域内每个像素 (x, y)，其中 y ∈ [0, cap_height)：
  t = y / cap_height                      ← 0.0(顶) → 1.0(底)
  alpha = (t × body_fill_opacity) as u8
  填充 (cap_color.r, cap_color.g, cap_color.b, alpha)
  
  所有 x 位置一致（矩形渐变，无水平变化）
```

### 4.3 身体 + 边框绘制

```
对 body 区域内每个像素 (x, y)：
  local_x = x - content_left
  local_y = y - body_start_y
  
  is_border = (
    border_enabled &&
    (
      local_x < border_width ||                           // 左边框
      local_x >= content_width - border_width ||          // 右边框
      local_y < border_width ||                           // 上边框
      local_y >= body_height - border_width               // 下边框
    )
  )
  
  if is_border:
      填充 border_color * border_opacity
  else:
      填充 fill_color * fill_opacity
```

### 4.4 抗锯齿

针对 Cap 形状边缘（球皮椭圆边界、菱形斜边），采用 **超采样 (Supersampling)**：

- 对每个输出像素，在子像素网格（如 4×4 = 16 个采样点）中计算覆盖比例
- alpha = 覆盖数 / 总采样数 × 目标 alpha
- 带来约 16× 的计算开销，仅对 Cap 区域启用（Cap 高度 ≤ 500px，开销可控）

备选/补充方案：对球皮边缘直接计算像素到椭圆边界的距离，用 **距离场 (SDF)** 做平滑：

```
distance = 1.0 - ((x-cx)²/rx² + (y-cy)²/ry²)
alpha = smoothstep(0, 1.5/像素密度, distance) × target_alpha
```

---

## 5. 前端设计

### 5.1 组件树

```
App.vue
├── ToolBar.vue              # 顶部工具栏：新建/撤销/重做
├── MainLayout.vue
│   ├── ConfigPanel.vue      # 左侧参数面板
│   │   ├── ImageSection.vue     # 图片尺寸
│   │   ├── MarginSection.vue    # 留白
│   │   ├── CapSection.vue       # 投的长度 + 顶端形状/颜色
│   │   ├── BodySection.vue      # 身体填充 + 边框
│   │   └── PresetSection.vue    # 预设列表 + 保存/删除
│   └── PreviewPanel.vue     # 右侧预览区
│       ├── PreviewCanvas.vue    # Canvas 预览图
│       ├── ZoomControl.vue      # 缩放控件
│       ├── AnnotationLayer.vue  # 标注线叠加层
│       └── ExportBar.vue        # 文件名 + 导出按钮
└── StatusBar.vue            # 底部状态栏
```

### 5.2 状态管理

使用 Vue 3 的 `reactive` 或 Pinia store：

```typescript
// store/tailConfig.ts
interface TailConfigState {
  config: TailConfig;
  presetName: string | null;
  dirty: boolean;            // 是否有未保存的修改
  undoStack: TailConfig[];
  redoStack: TailConfig[];
  previewUrl: string;        // blob URL，来自后端
}
```

参数修改 → `watch` → debounce 200ms → `invoke("render_preview", config)` → 接收 blob → 更新 Canvas。

### 5.3 Tauri Commands

```rust
#[tauri::command]
fn render_preview(config: TailConfig) -> Result<Vec<u8>, String>;
// 返回缩略图 PNG 字节（高度压缩至 ~800px 便于传输）

#[tauri::command]
fn export_image(config: TailConfig, path: String) -> Result<(), String>;
// 全分辨率渲染 + 写入文件

#[tauri::command]
fn list_presets() -> Vec<Preset>;

#[tauri::command]
fn save_preset(name: String, config: TailConfig) -> Result<(), String>;

#[tauri::command]
fn delete_preset(name: String) -> Result<(), String>;
```

### 5.4 预览 Canvas 实现要点

- 前端拿到缩略图 PNG 字节 → `createImageBitmap` → `CanvasRenderingContext2D.drawImage`
- 缩放模式切换时重算 `drawImage` 参数
- 1:1 模式下支持拖拽平移（mousedown/mousemove 偏移）
- 标注线用独立的 `<canvas>` 叠加层绘制（半透明红色虚线）

---

## 6. 预设存储

预设以 JSON 文件存储：

- **内置预设**：编译到 Rust 二进制中（静态常量）
- **用户预设**：存储在 `%APPDATA%/osu-mania-tail-maker/presets.json`
- 格式：

```json
{
  "version": 1,
  "presets": [
    {
      "name": "球皮-标准",
      "builtin": true,
      "config": { ... }
    }
  ]
}
```

---

## 7. 目录结构（提案）

```
osu-manina-tail-maker/
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs               # Tauri 入口
│       ├── lib.rs
│       ├── config.rs             # TailConfig 数据结构
│       ├── renderer/
│       │   ├── mod.rs
│       │   ├── cap.rs            # Cap 形状渲染
│       │   ├── body.rs           # Body + 边框渲染
│       │   ├── antialias.rs      # 抗锯齿工具
│       │   └── pipeline.rs       # 渲染管线
│       ├── export.rs             # PNG 编码 + 写文件
│       ├── preset.rs             # 预设管理
│       ├── commands.rs           # Tauri commands
│       └── validation.rs         # 参数校验
├── src/                          # Vue 前端
│   ├── App.vue
│   ├── main.ts
│   ├── store/
│   │   └── tailConfig.ts
│   ├── components/
│   │   ├── ConfigPanel/
│   │   │   ├── ImageSection.vue
│   │   │   ├── MarginSection.vue
│   │   │   ├── CapSection.vue
│   │   │   ├── BodySection.vue
│   │   │   └── PresetSection.vue
│   │   └── PreviewPanel/
│   │       ├── PreviewCanvas.vue
│   │       ├── ZoomControl.vue
│   │       └── ExportBar.vue
│   ├── types/
│   │   └── config.ts            # TypeScript 类型定义
│   └── utils/
│       └── debounce.ts
├── docs/
│   ├── glossary.md
│   ├── requirements-analysis.md
│   └── system-design.md
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md
```

---

## 8. 开发阶段划分

| 阶段 | 内容 | 估时 |
|------|------|------|
| **P0 - 骨架** | Tauri + Vue 项目搭建，ConfigPanel 静态 UI，前后端通信跑通 | 1 天 |
| **P1 - 核心渲染** | Rust 渲染引擎（Rect Cap + Body + 边框），预览 + 导出 | 2 天 |
| **P2 - 完整形状** | Ball / Diamond / Gradient Cap + 抗锯齿 | 2 天 |
| **P3 - 预设系统** | 内置预设 + 用户预设 CRUD | 1 天 |
| **P4 - 预览增强** | 缩放模式、标注线、1:1 查看、拖拽平移 | 1 天 |
| **P5 - 打磨** | 撤销/重做、校验提示、样式优化、打包配置 | 1 天 |

---

## 9. 关键风险与对策

| 风险 | 影响 | 对策 |
|------|------|------|
| 32800px 大图渲染性能 | 预览延迟大 | 预览用缩略图（高度压缩至 ~800px）；全分辨率仅导出时渲染 |
| 超采样抗锯齿耗时长 | Cap 渲染慢 | 仅 Cap 区域超采样；用 SDF 方案替代纯超采样 |
| PNG 编码大图内存 | 32800×40×4 ≈ 5MB，可控 | 但需注意高度增大时的倍数关系 |
| Vue ↔ Rust 数据同步 | 前后端类型不一致 | TypeScript 类型从 Rust 序列化结构体手写对齐，编译期无保障，需集成测试覆盖 |
