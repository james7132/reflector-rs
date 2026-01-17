use anyhow::Result;
use arch_mirrors_rs::{Mirror, Status};
use std::collections::HashMap;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    time::SystemTime,
};
use xdg::BaseDirectories;

pub(crate) fn get_cache_file(name: Option<&str>) -> PathBuf {
    let name = name.unwrap_or("mirrorstatus.json");
    let base_dirs = BaseDirectories::new();
    let cache_dir = base_dirs
        .get_cache_home()
        .unwrap_or_else(|| PathBuf::from("~/.cache"));
    fs::create_dir_all(&cache_dir).expect("creating directory should not fail");
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
    let is_invalid = mtime.is_none_or(|time| {
        let now = SystemTime::now();
        let elapsed = now.duration_since(time).expect("Time went backwards");
        elapsed.as_secs() > u64::from(cache_timeout)
    });
    let loaded = if is_invalid {
        let loaded = reqwest::get(url).await?.json().await?;
        let to_write = serde_json::to_string_pretty(&loaded)?;
        fs::write(cache_file_path, to_write)?;
        loaded
    } else {
        serde_json::from_reader(File::open(cache_file_path)?)?
    };
    Ok(loaded)
}

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct Country<'a> {
    pub country: &'a str,
    pub code: &'a str,
}

pub(crate) fn count_countries<'a>(
    mirrors: impl IntoIterator<Item = &'a Mirror>,
) -> HashMap<Country<'a>, usize> {
    let mut counts = HashMap::new();
    for mirror in mirrors {
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
