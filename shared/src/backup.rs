/// 备份工具
///
/// 文件备份到 `_backup/{timestamp}/` 目录下，支持时间戳子目录命名。
use std::fs;
use std::path::Path;

/// 生成备份时间戳目录名（UTC+8）。
pub fn backup_timestamp() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + 28800; // UTC+8
    let secs_of_day = ts % 86400;
    let days_since_epoch = ts / 86400;
    let (y, m, d) = epoch_to_date(days_since_epoch);
    let h = secs_of_day / 3600;
    let min = (secs_of_day % 3600) / 60;
    let s = secs_of_day % 60;
    format!("{:04}-{:02}-{:02}_{:02}-{:02}-{:02}", y, m, d, h, min, s)
}

/// 备份单个文件到 `backup_dir/{timestamp}/` 下。
///
/// `skin_dir` 用于计算文件的相对路径（生成扁平化的备份文件名）。
/// 如果备份目标已存在则跳过。
pub fn backup_file(skin_dir: &Path, file_path: &Path, backup_dir: &Path, ts_dir: &str) -> Result<(), String> {
    let backup_root = backup_dir.join(ts_dir);
    fs::create_dir_all(&backup_root).map_err(|e| format!("创建备份目录失败: {}", e))?;

    let relative = file_path
        .strip_prefix(skin_dir)
        .unwrap_or_else(|_| Path::new(file_path.file_name().unwrap_or_default()));
    let flat_name = relative
        .to_string_lossy()
        .replace(['/', '\\'], "-");

    let backup_path = backup_root.join(&flat_name);
    if backup_path.exists() {
        return Ok(());
    }
    fs::copy(file_path, &backup_path).map_err(|e| format!("备份失败: {}", e))?;
    Ok(())
}

/// 粗糙的 Unix epoch days → (year, month, day)，以 1970-01-01 为起点。
fn epoch_to_date(days: u64) -> (u64, u64, u64) {
    let mut y = 1970u64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1u64;
    for md in month_days.iter() {
        if remaining < *md {
            break;
        }
        remaining -= *md;
        m += 1;
    }
    let d = remaining as u64 + 1;
    (y, m, d)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}
