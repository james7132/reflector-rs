//! This is where the [`Url`] struct and all of its dependencies go.
use serde::{Deserialize, Serialize};

/// An Arch Linux mirror and its statistics.
#[derive(Debug, Clone, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Mirror {
    /// The url of the mirror.
    pub url: url::Url,

    /// The protocol that this mirror uses.
    pub protocol: crate::Protocol,

    /// The last time it synced from Arch Linux server.
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,

    /// Completion PCT. Unknown what this means.
    pub completion_pct: Option<f64>,

    /// The average duration. Unknown what this means.
    pub duration_average: Option<f64>,

    /// Duration StdDev. Unknown what this means.
    pub duration_stddev: Option<f64>,

    /// The score of the mirror. This is currently calculated as `(hours delay + average duration + standard deviation) / completion percentage`.
    /// Lower is better.
    pub score: Option<f64>,

    /// Whether or not the mirror is active.
    pub active: bool,

    /// The country where the mirror resides in.
    pub country: String,

    /// The ISO-3166-1 country code where the mirror resides in.
    pub country_code: String,

    /// Whether or not this mirror has Arch Linux ISOs(?)
    pub isos: bool,

    /// Whether or not this mirror supports IPv4.
    pub ipv4: bool,

    /// Whether or not this mirror supports IPv6.
    pub ipv6: bool,

    /// The details of the mirror.
    pub details: String,
}
