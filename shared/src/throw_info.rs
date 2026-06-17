/// 投长度信息获取与修改执行
///
/// 基于 `skin_ini`、`throw_length`、`image_utils` 模块，提供面尾投长度检测和批量修改功能。
use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::Serialize;

use crate::{backup, image_utils, skin_ini, throw_cache, throw_length};

/// 投长度信息
#[derive(Debug, Serialize, Clone)]
pub struct SkinThrowInfo {
    pub keys: u32,
    pub stem: String,
    pub column_width: u32,
    /// Stable 模式下的当前投长度（直接从原图计算）
    pub current_throw: u32,
    /// Lazer 模式下的当前投长度（拉伸到 ColumnWidth×1.6, 高度 32800 后计算）
    pub lazer_throw: u32,
    pub height: u32,
    pub valid: bool,
    pub is_2x: bool,
}

/// 图片-键数-轨道关联信息（供预设替换区块按 stem 分组展示）
#[derive(Debug, Serialize, Clone)]
pub struct ImageKeyInfo {
    pub stem: String,
    pub image_path: String,
    /// 该 stem 被哪些键数引用，以及对应的轨道列号
    pub used_by: Vec<KeyColumnEntry>,
}

#[derive(Debug, Serialize, Clone)]
pub struct KeyColumnEntry {
    pub keys: u32,
    pub columns: Vec<u32>,
}

/// Key/KeyD stem 信息（供 Key/KeyD 修复列表）
#[derive(Debug, Serialize, Clone)]
pub struct KeydStemInfo {
    pub stem: String,
    /// 作为 KeyImage# 的键数列表
    pub as_key: Vec<u32>,
    /// 作为 KeyImage#D 的键数列表
    pub as_keyd: Vec<u32>,
}

// ---------------------------------------------------------------------------
// 查询函数（纯数据获取，无副作用）
// ---------------------------------------------------------------------------

/// 获取图片-键数-轨道关联信息。
pub fn get_image_key_info(skin_dir: &Path) -> Result<Vec<ImageKeyInfo>, String> {
    let ini_path = skin_dir.join("skin.ini");
    if !ini_path.exists() {
        return Err("未找到 skin.ini 文件".to_string());
    }
    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;

    let mut stem_map: HashMap<String, Vec<(u32, Vec<u32>)>> = HashMap::new();

    for section in &skin_ini.mania_sections {
        for r in &section.note_image_ls {
            stem_map
                .entry(r.name.clone())
                .or_default()
                .push((section.keys, vec![r.column]));
        }
    }

    let mut results: Vec<ImageKeyInfo> = Vec::new();
    for (stem, entries) in stem_map {
        let mut key_groups: HashMap<u32, Vec<u32>> = HashMap::new();
        for (keys, cols) in &entries {
            key_groups.entry(*keys).or_default().extend(cols);
        }
        let mut used_by: Vec<KeyColumnEntry> = key_groups
            .into_iter()
            .map(|(keys, mut columns)| {
                columns.sort();
                columns.dedup();
                KeyColumnEntry { keys, columns }
            })
            .collect();
        used_by.sort_by_key(|e| e.keys);

        let image_path = skin_ini::find_image_file(skin_dir, &stem)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        results.push(ImageKeyInfo {
            stem,
            image_path,
            used_by,
        });
    }

    results.sort_by(|a, b| a.stem.cmp(&b.stem));
    Ok(results)
}

/// 获取 Key/KeyD 图片信息列表（每个 stem 在哪些 keys 下是 KeyImage# 或 KeyImage#D）。
pub fn get_keyd_list(skin_dir: &Path) -> Result<Vec<KeydStemInfo>, String> {
    let ini_path = skin_dir.join("skin.ini");
    if !ini_path.exists() {
        return Err("未找到 skin.ini 文件".to_string());
    }
    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;

    let mut stem_map: HashMap<String, (HashSet<u32>, HashSet<u32>)> = HashMap::new();

    for section in &skin_ini.mania_sections {
        for r in &section.key_images {
            stem_map
                .entry(r.name.clone())
                .or_default()
                .0
                .insert(section.keys);
        }
        for r in &section.key_image_ds {
            stem_map
                .entry(r.name.clone())
                .or_default()
                .1
                .insert(section.keys);
        }
    }

    let mut results: Vec<KeydStemInfo> = Vec::new();
    for (stem, (key_set, keyd_set)) in stem_map {
        let mut as_key: Vec<u32> = key_set.into_iter().collect();
        as_key.sort();
        let mut as_keyd: Vec<u32> = keyd_set.into_iter().collect();
        as_keyd.sort();
        results.push(KeydStemInfo {
            stem,
            as_key,
            as_keyd,
        });
    }

    results.sort_by(|a, b| a.stem.cmp(&b.stem));
    Ok(results)
}

/// 获取皮肤的投长度信息。
pub fn get_throw_info(skin_dir: &Path) -> Result<Vec<SkinThrowInfo>, String> {
    let ini_path = skin_dir.join("skin.ini");
    if !ini_path.exists() {
        return Err("未找到 skin.ini 文件".to_string());
    }

    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;
    let mut results: Vec<SkinThrowInfo> = Vec::new();

    for section in &skin_ini.mania_sections {
        if section.note_image_ls.is_empty() {
            continue;
        }

        let mut stems: Vec<String> = section
            .note_image_ls
            .iter()
            .map(|r| r.name.clone())
            .collect();
        stems.sort();
        stems.dedup();

        for stem in &stems {
            let info = read_single_throw_info(skin_dir, stem, section.keys, section.column_width);
            results.push(info);
        }
    }

    Ok(results)
}

/// 读取单个 stem 的投长度信息（含缓存逻辑）。
fn read_single_throw_info(
    skin_dir: &Path,
    stem: &str,
    keys: u32,
    column_width: u32,
) -> SkinThrowInfo {
    let image_path = match skin_ini::find_image_file(skin_dir, stem) {
        Some(p) => p,
        None => {
            return SkinThrowInfo {
                keys,
                stem: stem.to_string(),
                column_width: 0,
                current_throw: 0,
                lazer_throw: 0,
                height: 0,
                valid: false,
                is_2x: false,
            };
        }
    };

    let is_2x = image_utils::is_2x(&image_path);

    let cache_key = throw_cache::hash_file(&image_path).ok();
    let cached = cache_key.as_deref().and_then(throw_cache::get);

    match image::open(&image_path) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (valid, height) = throw_length::validate_tail_image(&rgba);
            let current_throw = if let Some(ref c) = cached {
                c.stable_throw
            } else {
                let t = throw_length::compute_throw_stable(&rgba, column_width);
                if let Some(ref k) = cache_key {
                    throw_cache::set(k, &throw_cache::ThrowCacheEntry {
                        stable_throw: t,
                        lazer_throw: 0,
                    });
                }
                t
            };
            let lazer_throw = cached.as_ref().map_or(0, |c| c.lazer_throw);

            SkinThrowInfo {
                keys,
                stem: stem.to_string(),
                column_width,
                current_throw,
                lazer_throw,
                height,
                valid,
                is_2x,
            }
        }
        Err(_) => SkinThrowInfo {
            keys,
            stem: stem.to_string(),
            column_width: 0,
            current_throw: 0,
            lazer_throw: 0,
            height: 0,
            valid: false,
            is_2x,
        },
    }
}

/// 按需计算指定 stems 的 lazer 投长度（优先从缓存读取，未命中则计算并写缓存）。
pub fn compute_lazer_throws(
    skin_dir: &Path,
    stems: &[String],
) -> Result<Vec<(String, u32)>, String> {
    let mut results: Vec<(String, u32)> = Vec::new();
    let mut seen = HashSet::new();

    for stem in stems {
        if !seen.insert(stem.clone()) {
            continue;
        }
        let t = compute_lazer_throw_single(skin_dir, stem, 0);
        results.push((stem.clone(), t));
    }

    Ok(results)
}

/// 计算单个 stem 的 lazer 投长度。
///
/// 优先从缓存读取，未命中则计算并写入缓存。
/// 返回 lazer_throw（px），失败返回 0。
pub fn compute_lazer_throw_single(
    skin_dir: &Path,
    stem: &str,
    _column_width: u32,
) -> u32 {
    let image_path = match skin_ini::find_image_file(skin_dir, stem) {
        Some(p) => p,
        None => return 0,
    };

    if let Ok(cache_key) = throw_cache::hash_file(&image_path) {
        if let Some(cached) = throw_cache::get(&cache_key) {
            if cached.lazer_throw > 0 {
                return cached.lazer_throw;
            }
        }
        if let Ok(img) = image::open(&image_path) {
            let rgba = img.to_rgba8();
            let (valid, _) = throw_length::validate_tail_image(&rgba);
            if valid {
                let t = throw_length::compute_throw_lazer(&rgba, 0);
                let stable = throw_cache::get(&cache_key)
                    .map_or(0, |c| c.stable_throw);
                throw_cache::set(&cache_key, &throw_cache::ThrowCacheEntry {
                    stable_throw: stable,
                    lazer_throw: t,
                });
                return t;
            }
        }
    }

    0
}

// ---------------------------------------------------------------------------
// 修改函数
// ---------------------------------------------------------------------------

/// 对单个 stem 执行投长度修改。
///
/// 返回日志行。
pub fn modify_one_throw_stem(
    skin_dir: &Path,
    stem: &str,
    keys: u32,
    target_throw: u32,
    mode: &str,
    cw: u32,
    backup_dir: &Path,
    ts_dir: &str,
) -> Result<Vec<String>, String> {
    let mut log: Vec<String> = Vec::new();

    let image_path = match skin_ini::find_image_file(skin_dir, stem) {
        Some(p) => p,
        None => {
            log.push(format!(
                "⚠ 找不到面尾图片: {} ({}k)",
                stem, keys
            ));
            return Ok(log);
        }
    };

    let img = image::open(&image_path)
        .map_err(|e| format!("读取图片失败 {}: {}", stem, e))?
        .to_rgba8();

    let current_throw = throw_length::find_throw_length(&img);

    let modified = if mode == "lazer" {
        let h = img.height();
        let current_lazer = if h > 0 {
            ((current_throw as u64 * 32800) / h as u64) as u32
        } else {
            0
        };
        log.push(format!(
            "{} {}k: 投长度 {}px → {}px (Lazer, cw={})",
            image_path.display(),
            keys,
            current_lazer,
            target_throw,
            cw,
        ));
        throw_length::modify_throw_length_lazer(&img, target_throw, cw)
    } else {
        log.push(format!(
            "{} {}k: 投长度 {}px → {}px (Stable)",
            image_path.display(),
            keys,
            current_throw,
            target_throw,
        ));
        throw_length::modify_throw_length(&img, target_throw)
    };

    backup::backup_file(skin_dir, &image_path, backup_dir, ts_dir)?;
    modified
        .save(&image_path)
        .map_err(|e| format!("保存失败 {}: {}", stem, e))?;

    Ok(log)
}

/// 执行投长度修改。
///
/// # 参数
/// - `throw_map`: 键数 → 目标投长度的映射
/// - `mode`: "stable" 或 "lazer"
/// - `column_widths`: 键数 → ColumnWidth 映射（仅 lazer 模式需要）
///
/// # 返回
/// 日志行列表。
pub fn execute_throw_modification(
    skin_dir: &Path,
    throw_map: &HashMap<u32, u32>,
    backup_dir: &Path,
    mode: &str,
    column_widths: &HashMap<u32, u32>,
) -> Result<Vec<String>, String> {
    let ini_path = skin_dir.join("skin.ini");
    if !ini_path.exists() {
        return Err("未找到 skin.ini 文件".to_string());
    }

    let skin_ini = skin_ini::parse_skin_ini(&ini_path)?;
    let mut log: Vec<String> = Vec::new();

    let mut seen = HashSet::new();
    let mut processed = false;
    let ts_dir = backup::backup_timestamp();

    for section in &skin_ini.mania_sections {
        let target_throw = match throw_map.get(&section.keys) {
            Some(&t) => t,
            None => continue,
        };

        let cw = column_widths
            .get(&section.keys)
            .copied()
            .unwrap_or(section.column_width);

        for r in &section.note_image_ls {
            if !seen.insert(r.name.clone()) {
                continue;
            }

            match modify_one_throw_stem(
                skin_dir, &r.name, section.keys, target_throw,
                mode, cw, backup_dir, &ts_dir,
            ) {
                Ok(stem_log) => log.extend(stem_log),
                Err(e) => log.push(format!("✗ {}: {}", r.name, e)),
            }
            processed = true;
        }
    }

    if !processed {
        log.push("未找到匹配的键数小节".to_string());
    } else {
        log.push("修改完成！".to_string());
    }

    Ok(log)
}