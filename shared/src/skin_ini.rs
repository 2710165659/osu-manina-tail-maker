/// skin.ini 解析器
///
/// 解析 osu! skin.ini 中的 [Mania] 小节，提取：
/// - Keys、ColumnWidth
/// - KeyImage#  — 按键背景图
/// - KeyImage#D — 按键按下图（KeyD）
/// - NoteImage#L — 面尾图（长条尾）
///
/// 注意：# 是列号（0-based 轨道索引），不是键数。
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// 图片引用 — 记录某行的列号、文件名、行号
#[derive(Debug, Clone)]
pub struct ImageRef {
    /// 列号（0-based）
    pub column: u32,
    /// skin.ini 中该键对应的值（图片文件名茎，不含路径和扩展名）
    pub name: String,
    /// 该键所在行号（0-based）
    pub line: usize,
}

/// 一个 [Mania] 小节的配置信息
#[derive(Debug, Clone)]
pub struct ManiaSection {
    /// 键数，如 4, 5, ..., 18
    pub keys: u32,
    /// ColumnWidth（取第一个值，因为通常所有列相同）
    pub column_width: u32,
    /// KeyImage# 引用列表
    pub key_images: Vec<ImageRef>,
    /// KeyImage#D 引用列表
    pub key_image_ds: Vec<ImageRef>,
    /// NoteImage#L 引用列表
    pub note_image_ls: Vec<ImageRef>,
    /// 该小节在文件中的起始行号（0-based）
    pub line_start: usize,
    /// 该小节在文件中的结束行号（0-based，排他）
    pub line_end: usize,
}

/// skin.ini 解析结果
#[derive(Debug, Clone)]
pub struct SkinIni {
    /// skin.ini 文件路径
    pub path: PathBuf,
    /// skin.ini 所在目录
    pub directory: PathBuf,
    /// 所有 [Mania] 小节
    pub mania_sections: Vec<ManiaSection>,
}

// ---------------------------------------------------------------------------
// Parse entry points
// ---------------------------------------------------------------------------

/// 解析 skin.ini 文件
pub fn parse_skin_ini(path: &Path) -> Result<SkinIni, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("读取 skin.ini 失败: {}", e))?;

    let directory = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let lines: Vec<&str> = content.lines().collect();
    let mania_sections = parse_mania_sections(&lines);

    Ok(SkinIni {
        path: path.to_path_buf(),
        directory,
        mania_sections,
    })
}

/// 在行集合中扫描所有 [Mania] 小节
fn parse_mania_sections(lines: &[&str]) -> Vec<ManiaSection> {
    let mut sections = Vec::new();
    let mut in_mania = false;
    let mut section_start = 0usize;

    // 当前小节正在收集的数据
    let mut keys: Option<u32> = None;
    let mut column_width: Option<u32> = None;
    let mut key_images: Vec<ImageRef> = Vec::new();
    let mut key_image_ds: Vec<ImageRef> = Vec::new();
    let mut note_image_ls: Vec<ImageRef> = Vec::new();

    let flush = |secs: &mut Vec<ManiaSection>,
                 keys: &mut Option<u32>,
                 cw: &mut Option<u32>,
                 ki: &mut Vec<ImageRef>,
                 kd: &mut Vec<ImageRef>,
                 nl: &mut Vec<ImageRef>,
                 start: usize,
                 end: usize| {
        if let Some(k) = keys.take() {
            secs.push(ManiaSection {
                keys: k,
                column_width: cw.unwrap_or(0),
                key_images: std::mem::take(ki),
                key_image_ds: std::mem::take(kd),
                note_image_ls: std::mem::take(nl),
                line_start: start,
                line_end: end,
            });
        }
        *cw = None;
    };

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // 小节切换
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_mania {
                flush(
                    &mut sections,
                    &mut keys,
                    &mut column_width,
                    &mut key_images,
                    &mut key_image_ds,
                    &mut note_image_ls,
                    section_start,
                    i,
                );
            }

            in_mania = trimmed.eq_ignore_ascii_case("[Mania]");
            if in_mania {
                section_start = i;
            }
            continue;
        }

        if !in_mania {
            continue;
        }

        // Keys:
        if let Some(val) = try_parse_key(trimmed, "Keys:") {
            keys = val.parse::<u32>().ok();
            continue;
        }

        // ColumnWidth:
        if let Some(val) = try_parse_key(trimmed, "ColumnWidth:") {
            // 取第一个值（逗号分隔时）
            if let Some(first) = val.split(',').next() {
                column_width = first.trim().parse::<u32>().ok();
            }
            continue;
        }

        // KeyImage#D:  ← 必须在 KeyImage# 之前检测
        if let Some((col, name)) = try_parse_image_ref(trimmed, "KeyImage", "D") {
            key_image_ds.push(ImageRef {
                column: col,
                name,
                line: i,
            });
            continue;
        }

        // KeyImage#:
        if let Some((col, name)) = try_parse_image_ref(trimmed, "KeyImage", "") {
            key_images.push(ImageRef {
                column: col,
                name,
                line: i,
            });
            continue;
        }

        // NoteImage#L:
        if let Some((col, name)) = try_parse_image_ref(trimmed, "NoteImage", "L") {
            note_image_ls.push(ImageRef {
                column: col,
                name,
                line: i,
            });
            continue;
        }
    }

    // 文件末尾
    if in_mania {
        flush(
            &mut sections,
            &mut keys,
            &mut column_width,
            &mut key_images,
            &mut key_image_ds,
            &mut note_image_ls,
            section_start,
            lines.len(),
        );
    }

    sections
}

// ---------------------------------------------------------------------------
// Low-level helpers
// ---------------------------------------------------------------------------

/// 匹配 "Key: value"，返回 value 部分。
pub fn try_parse_key<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let lower = line.to_lowercase();
    let key_lower = key.to_lowercase();
    if lower.starts_with(&key_lower) {
        Some(line[key.len()..].trim())
    } else {
        None
    }
}

/// 通用解析器：匹配 `<prefix><数字><suffix>: value`。
/// - `prefix`: 键名前缀（如 "KeyImage"、"NoteImage"）
/// - `suffix`: 键名尾缀（如 "D"、"L"、""）
/// 返回 `(列号, value)`。
fn try_parse_image_ref(line: &str, prefix: &str, suffix: &str) -> Option<(u32, String)> {
    let lower = line.to_lowercase();
    let prefix_lower = prefix.to_lowercase();
    if !lower.starts_with(&prefix_lower) {
        return None;
    }

    let after_prefix = &lower[prefix.len()..];
    // 收集数字
    let num_len = count_leading_digits(after_prefix);
    if num_len == 0 {
        return None;
    }
    let col: u32 = after_prefix[..num_len].parse().ok()?;

    let after_num = &after_prefix[num_len..];
    let suffix_lower = suffix.to_lowercase();

    // 检查后缀 + ':'
    if after_num.len() < suffix_lower.len() + 1 {
        return None;
    }

    // 尾缀匹配（忽略大小写）
    if !after_num[..suffix_lower.len()].eq_ignore_ascii_case(suffix) {
        return None;
    }

    let after_suffix = &after_num[suffix_lower.len()..];
    // 必须紧接 ':' 或空格后 ':'
    let after_trimmed = after_suffix.trim_start();
    if !after_trimmed.starts_with(':') {
        return None;
    }

    let colon_pos = line.find(':')?;
    let value = line[colon_pos + 1..].trim().to_string();
    Some((col, value))
}

fn count_leading_digits(s: &str) -> usize {
    let mut n = 0;
    for c in s.chars() {
        if c.is_ascii_digit() {
            n += 1;
        } else {
            break;
        }
    }
    n
}

// ---------------------------------------------------------------------------
// Grouping helpers
// ---------------------------------------------------------------------------

/// 提取所有唯一的 NoteImage#L 图片茎（跨所有小节去重）。
pub fn unique_note_image_l_stems(sections: &[ManiaSection]) -> Vec<String> {
    let mut set: Vec<String> = Vec::new();
    for sec in sections {
        for r in &sec.note_image_ls {
            if !set.contains(&r.name) {
                set.push(r.name.clone());
            }
        }
    }
    set
}

/// 提取所有唯一的 KeyImage#D 图片茎（跨所有小节去重）。
pub fn unique_key_image_d_stems(sections: &[ManiaSection]) -> Vec<String> {
    let mut set: Vec<String> = Vec::new();
    for sec in sections {
        for r in &sec.key_image_ds {
            if !set.contains(&r.name) {
                set.push(r.name.clone());
            }
        }
    }
    set
}

/// 提取所有唯一的 KeyImage# 图片茎（跨所有小节去重）。
pub fn unique_key_image_stems(sections: &[ManiaSection]) -> Vec<String> {
    let mut set: Vec<String> = Vec::new();
    for sec in sections {
        for r in &sec.key_images {
            if !set.contains(&r.name) {
                set.push(r.name.clone());
            }
        }
    }
    set
}

/// 按图片茎将 NoteImage#L 引用分组。
/// 返回 `HashMap<图片茎, Vec<(section_keys, column_width)>>`
pub fn group_note_image_l_by_stem(
    sections: &[ManiaSection],
) -> HashMap<String, Vec<(u32, u32)>> {
    let mut groups: HashMap<String, Vec<(u32, u32)>> = HashMap::new();
    for sec in sections {
        for r in &sec.note_image_ls {
            groups
                .entry(r.name.clone())
                .or_default()
                .push((sec.keys, sec.column_width));
        }
    }
    // 每个组内去重
    for v in groups.values_mut() {
        v.sort();
        v.dedup();
    }
    groups
}

/// 按图片茎将 KeyImage#D 引用分组。
pub fn group_key_image_d_by_stem(
    sections: &[ManiaSection],
) -> HashMap<String, Vec<(u32, u32)>> {
    let mut groups: HashMap<String, Vec<(u32, u32)>> = HashMap::new();
    for sec in sections {
        for r in &sec.key_image_ds {
            groups
                .entry(r.name.clone())
                .or_default()
                .push((sec.keys, sec.column_width));
        }
    }
    for v in groups.values_mut() {
        v.sort();
        v.dedup();
    }
    groups
}

/// 按图片茎将 KeyImage# 和 KeyImage#D 引用合并分组。
/// 同一图片茎在 KeyImage# 和 KeyImage#D 中的引用合并到同一组。
pub fn group_key_images_by_stem(
    sections: &[ManiaSection],
) -> HashMap<String, Vec<(u32, u32)>> {
    let mut groups: HashMap<String, Vec<(u32, u32)>> = HashMap::new();
    for sec in sections {
        for r in sec.key_image_ds.iter().chain(sec.key_images.iter()) {
            groups
                .entry(r.name.clone())
                .or_default()
                .push((sec.keys, sec.column_width));
        }
    }
    for v in groups.values_mut() {
        v.sort();
        v.dedup();
    }
    groups
}

// ---------------------------------------------------------------------------
// File lookup
// ---------------------------------------------------------------------------

/// 在给定目录下查找图片文件的实际路径。
/// 优先查找 @2x 版本（如 `name@2x.png`），找不到再查普通扩展名。
/// 也支持 stem 已含扩展名的情况。
pub fn find_image_file(dir: &Path, stem: &str) -> Option<PathBuf> {
    // 1. stem 直接匹配（已含扩展名）
    let direct = dir.join(stem);
    if direct.exists() {
        return Some(direct);
    }

    // 2. 优先 @2x（osu! 命名规范：stem@2x.ext）
    let exts = ["png", "jpg", "jpeg"];
    for ext in &exts {
        let full = format!("{}@2x.{}", stem, ext);
        let p = dir.join(&full);
        if p.exists() {
            return Some(p);
        }
    }

    // 3. 普通扩展名
    for ext in &exts {
        let full = format!("{}.{}", stem, ext);
        let p = dir.join(&full);
        if p.exists() {
            return Some(p);
        }
    }

    None
}
