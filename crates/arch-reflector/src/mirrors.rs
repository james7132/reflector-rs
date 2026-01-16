use anyhow::Result;
use arch_mirrors_rs::{Mirror, Status};
use directories::BaseDirs;
use std::collections::HashMap;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    time::SystemTime,
};

pub fn get_cache_file(name: Option<&str>) -> PathBuf {
    let name = name.unwrap_or("mirrorstatus.json");
    let base_dirs = BaseDirs::new().expect("is not expected to fail");
    let cache_dir = base_dirs.cache_dir();
    fs::create_dir_all(cache_dir).expect("creating directory should not fail");
    cache_dir.join(name)
}

/// Retrieve the mirror status JSON object. The downloaded data will be cached locally and
/// re-used within the cache timeout period. Returns the object and the local cache's
/// modification time.
pub async fn get_mirror_status(
    // TODO: Allow using this parameter
    _connection_timeout: u8,
    cache_timeout: u8,
    url: &str,
    cache_file_path: &Path,
) -> Result<Status> {
    let mtime = cache_file_path
        .metadata()
        .ok()
        .and_then(|meta| meta.modified().ok());
    let is_invalid = mtime
        .map(|time| {
            let now = SystemTime::now();
            let elapsed = now.duration_since(time).expect("Time went backwards");
            elapsed.as_secs() > cache_timeout as u64
        })
        .unwrap_or(true);
    let loaded = if !is_invalid {
        serde_json::from_reader(File::open(cache_file_path)?)?
    } else {
        let loaded = Status::get_from_url(url).await?;
        let to_write = serde_json::to_string_pretty(&loaded)?;
        fs::write(cache_file_path, to_write)?;
        loaded
    };
    Ok(loaded)
}

#[derive(PartialEq, Eq, Hash)]
pub struct Country<'a> {
    pub country: &'a str,
    pub code: &'a str,
}

pub async fn count_countries(mirrors: &[Mirror]) -> HashMap<Country<'_>, usize> {
    let mut counts = HashMap::new();
    for mirror in mirrors.iter() {
        if mirror.country_code.is_empty() {
            continue;
        }
        counts
            .entry(Country {
                country: mirror.country.as_ref(),
                code: mirror.country_code.as_ref(),
            })
            .and_modify(|e| *e += 1)
            .or_insert(1);
    }
    counts
}
