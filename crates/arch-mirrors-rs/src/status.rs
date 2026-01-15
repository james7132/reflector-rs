//! This is where the [`Status`] struct and all of its direct dependencies go.
use serde::{Deserialize, Serialize};

/// The status of all the Arch Linux mirrors.
#[derive(Debug, Clone, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Status {
    /// The cut off.
    pub cutoff: u32,

    /// The last time every listed Arch Linux mirror polled the [`lastsync`] file.
    pub last_check: chrono::DateTime<chrono::Utc>,

    /// The number of checks that have been run in the last 24 hours.
    pub num_checks: u32,

    /// The frequency of each check.
    pub check_frequency: u32,

    /// Every known Arch Linux mirror.
    pub urls: Vec<crate::Mirror>,

    /// The version of the status.
    pub version: u32,
}

impl Status {
    /// The URL where the JSON is found from.
    pub const DEFAULT_URL: &'static str = "https://archlinux.org/mirrors/status/json";

    /// Get the status from [`Status::URL`](Self::URL).
    pub async fn get_from_default_url() -> reqwest::Result<Self> {
        Self::get_from_url(Self::DEFAULT_URL).await
    }

    /// Get the status from a given url.
    pub async fn get_from_url(url: &str) -> reqwest::Result<Self> {
        let response = reqwest::get(url).await?;
        let value = response.json().await;
        Ok(value?)
    }
}
