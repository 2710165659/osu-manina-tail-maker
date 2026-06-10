/// 皮肤文件校验
///
/// 检查 skin.ini 引用的所有图片文件是否存在。
/// 包括 NoteImage#L（面尾）、KeyImage#D（KeyD）、KeyImage#（按键）。

use std::path::PathBuf;

use crate::skin_ini::{find_image_file, SkinIni};

/// 缺失文件信息
#[derive(Debug, Clone)]
pub struct MissingFile {
    /// 缺失的文件名（不含扩展名）
    pub stem: String,
    /// 查找目录
    pub directory: PathBuf,
    /// 引用该文件的 keys 列表
    pub keys: Vec<u32>,
    /// 图片类型标签
    pub image_type: String,
}

/// 校验皮肤文件完整性。
pub fn validate_skin_files(skin_ini: &SkinIni) -> Vec<MissingFile> {
    let mut missing: Vec<MissingFile> = Vec::new();

    for section in &skin_ini.mania_sections {
        for r in &section.note_image_ls {
            check(&mut missing, &r.name, &skin_ini.directory, section.keys, "NoteImage#L");
        }
        for r in &section.key_image_ds {
            check(&mut missing, &r.name, &skin_ini.directory, section.keys, "KeyImage#D");
        }
        for r in &section.key_images {
            check(&mut missing, &r.name, &skin_ini.directory, section.keys, "KeyImage#");
        }
    }

    missing
}

fn check(
    missing: &mut Vec<MissingFile>,
    stem: &str,
    dir: &PathBuf,
    keys: u32,
    img_type: &str,
) {
    if find_image_file(dir, stem).is_some() {
        return;
    }
    if let Some(existing) = missing
        .iter_mut()
        .find(|m| m.stem == stem && m.image_type == img_type)
    {
        if !existing.keys.contains(&keys) {
            existing.keys.push(keys);
        }
        return;
    }
    missing.push(MissingFile {
        stem: stem.to_string(),
        directory: dir.clone(),
        keys: vec![keys],
        image_type: img_type.to_string(),
    });
}
