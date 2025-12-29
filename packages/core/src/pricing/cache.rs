use std::path::PathBuf;
use std::fs;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

const CACHE_TTL_SECS: u64 = 3600;

pub fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("tokscale")
}

pub fn get_cache_path(filename: &str) -> PathBuf {
    get_cache_dir().join(filename)
}

#[derive(Serialize, Deserialize)]
pub struct CachedData<T> {
    pub timestamp: u64,
    pub data: T,
}

pub fn load_cache<T: for<'de> Deserialize<'de>>(filename: &str) -> Option<T> {
    let path = get_cache_path(filename);
    let content = fs::read_to_string(&path).ok()?;
    let cached: CachedData<T> = serde_json::from_str(&content).ok()?;
    
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    if now - cached.timestamp > CACHE_TTL_SECS {
        return None;
    }
    
    Some(cached.data)
}

pub fn save_cache<T: Serialize>(filename: &str, data: &T) -> Result<(), std::io::Error> {
    let dir = get_cache_dir();
    fs::create_dir_all(&dir)?;
    
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let cached = CachedData { timestamp: now, data };
    let content = serde_json::to_string(&cached)?;
    
    // Atomic write: write to temp file first, then rename
    // This prevents corruption from concurrent writes or crashes
    let final_path = get_cache_path(filename);
    let tmp_path = final_path.with_extension("tmp");
    
    use std::io::Write;
    let mut file = fs::File::create(&tmp_path)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;  // Ensure data is flushed to disk
    
    // Atomic rename (POSIX guarantees atomicity for same-filesystem renames)
    fs::rename(&tmp_path, &final_path)
}
