//! This is where the [`Url`] struct and all of its dependencies go.
use serde::{Deserialize, Serialize};

/// An Arch Linux mirror and its statistics.
#[derive(Debug, Clone, PartialOrd, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct Mirror {
    /// The url of the mirror.
    pub url: url::Url,

    /// The protocol that this mirror uses.
    pub protocol: crate::Protocol,

    /// The last time it synced from Arch Linux server.
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,

    /// The number of mirror checks that have successfully connected and disconnected from the
    /// given URL. If this is below 100%, the mirror may be unreliable.
    pub completion_pct: Option<f64>,

    /// The calculated average mirroring delay; e.g. the mean value of `last check − last sync` for
    /// each check of this mirror URL. Due to the timing of mirror checks, any value under one hour
    /// should be viewed as ideal.
    pub delay: Option<u32>,

    /// The average (mean) time it took to connect and retrieve the `lastsync` file from the given
    /// URL. Note that this connection time is from the location of the Arch server; your geography
    /// may product different results.
    pub duration_average: Option<f64>,

    /// The standard deviation of the connect and retrieval time. A high standard deviation can
    /// indicate an unstable or overloaded mirror.
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
