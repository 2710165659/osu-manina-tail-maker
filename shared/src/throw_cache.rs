use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};

/// 单张图片的投长度缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrowCacheEntry {
    pub stable_throw: u32,
    pub lazer_throw: u32,
}

/// 缓存目录
fn cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(dir)
            .join("osu-mania-tail-maker")
            .join("throw_cache")
    } else {
        PathBuf::from("throw_cache")
    }
}

/// 计算文件的 SHA-256 hash（返回前 16 个十六进制字符作为缓存 key）
pub fn hash_file(path: &Path) -> Result<String, String> {
    use std::hash::{Hash, Hasher};
    // 使用文件元数据 + 前 8KB 内容做快速 hash，避免全文件 SHA-256
    let mut file = std::fs::File::open(path).map_err(|e| format!("打开文件失败: {}", e))?;
    let metadata = file.metadata().map_err(|e| format!("读取元数据失败: {}", e))?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    metadata.len().hash(&mut hasher);
    metadata
        .modified()
        .map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })
        .unwrap_or(0)
        .hash(&mut hasher);

    let mut buf = [0u8; 8192];
    if let Ok(n) = file.read(&mut buf) {
        buf[..n].hash(&mut hasher);
    }
    let h = hasher.finish();
    Ok(format!("{:016x}", h))
}

/// 从缓存读取投长度
pub fn get(hash: &str) -> Option<ThrowCacheEntry> {
    let path = cache_dir().join(format!("{}.json", hash));
    if !path.exists() {
        return None;
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// 写入缓存
pub fn set(hash: &str, entry: &ThrowCacheEntry) {
    let dir = cache_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("{}.json", hash));
    if let Ok(json) = serde_json::to_string(entry) {
        let _ = std::fs::write(&path, json);
    }
}
