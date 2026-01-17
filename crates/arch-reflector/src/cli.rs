#![allow(
    clippy::doc_markdown,
    reason = "This is used to generate the user facing help."
)]

use arch_mirrors_rs::Protocol;
use clap::{ArgAction, Args, Parser, ValueEnum, value_parser};
use clap_verbosity_flag::Verbosity;

pub const URL: &str = "https://archlinux.org/mirrors/status/json/";
pub const DEFAULT_CONNECTION_TIMEOUT: u16 = 5;
pub const DEFAULT_DOWNLOAD_TIMEOUT: u64 = 5;
pub const DEFAULT_CACHE_TIMEOUT: u16 = 300;

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum SortTypes {
    /// last server synchronization
    Age,
    /// download rate Rate,
    Rate,
    /// country name, either alphabetically or in the order given by the --country option
    Country,
    /// MirrorStatus score
    Score,
    /// MirrorStatus delay
    Delay,
}

#[derive(Parser, Debug)]
#[command(
    about,
    author,
    version,
    propagate_version = true,
    next_line_help = false,
    disable_help_subcommand = true,
    after_long_help = "Retrieve and filter a list of the latest Arch Linux mirrors."
)]
pub struct Cli {
    /// The URL from which to retrieve the mirror data in JSON format. If different from
    /// the default, it must follow the same format.
    #[arg(long, default_value = URL)]
    pub url: String,

    /// Display a table of the distribution of servers by country.
    #[arg(long)]
    pub list_countries: bool,

    /// Print extra information to STDERR. Only works with some options.
    #[clap(flatten)]
    pub verbose: Verbosity,

    #[command(flatten)]
    pub run: RunOptions,
}

#[derive(Debug, Args)]
pub struct RunOptions {
    /// The number of seconds to wait before a connection times out.
    #[arg(long, default_value_t = DEFAULT_CONNECTION_TIMEOUT, value_name = "n")]
    pub connection_timeout: u16,

    /// The number of seconds to wait before a download times out.
    #[arg(long, default_value_t = DEFAULT_DOWNLOAD_TIMEOUT, value_name = "n")]
    pub download_timeout: u64,

    /// The cache timeout in seconds for the data retrieved from the Arch Linux Mirror
    /// Status API.
    #[arg(long, default_value_t = DEFAULT_CACHE_TIMEOUT, value_name = "n")]
    pub cache_timeout: u16,

    /// Save the mirrorlist to the given file path.
    #[arg(long, value_name = "filepath")]
    pub save: Option<String>,

    /// Sort the mirrorlist by the given field.
    #[arg(long)]
    pub sort: Option<SortTypes>,

    /// Use n threads for rating mirrors. This option will speed up the rating step but the
    /// results will be inaccurate if the local bandwidth is saturated at any point during
    /// the operation. If rating takes too long without this option then you should
    /// probably apply more filters to reduce the number of rated servers before using this
    /// option.
    #[arg(long, default_value_t = 0)]
    pub threads: usize,

    /// Print mirror information instead of a mirror list. Filter options apply.
    #[arg(long, default_value_t = false)]
    pub info: bool,

    #[command(flatten)]
    pub filters: Filters,
}

#[derive(Parser, Debug)]
#[command(
    next_help_heading = "filters",
    // FIXME: Display this after heading name, currently it is not displayed at all.
    after_long_help = "The following filters are inclusive, i.e. the returned list will only contain mirrors for which all of the given conditions are met."
)]
pub struct Filters {
    /// Only return mirrors that have synchronized in the last n hours. n may be an integer
    /// or a decimal number.
    #[arg(long, short, value_name = "n")]
    pub age: Option<f32>,

    /// Only return mirrors with a reported sync delay of n hours or less, where n is a float. For example. to limit the results to mirrors with a reported delay of 15 minutes or less, pass 0.25.
    #[arg(long, value_name = "n")]
    pub delay: Option<f32>,

    /// Restrict mirrors to selected countries. Countries may be given by name or country
    /// code, or a mix of both. The case is ignored. Multiple countries be selected using
    /// commas (e.g. --country France,Germany) or by passing this option multiple times
    /// (e.g.  -c fr -c de). Use "--list-countries" to display a table of available
    /// countries along with their country codes. When sorting by country, this option may
    /// also be used to sort by a preferred order instead of alphabetically. For example,
    /// to select mirrors from Sweden, Norway, Denmark and Finland, in that order, use the
    /// options "--country se,no,dk,fi --sort country". To set a preferred country sort
    /// order without filtering any countries.  this option also recognizes the glob
    /// pattern "*", which will match any country. For example, to ensure that any mirrors
    /// from Sweden are at the top of the list and any mirrors from Denmark are at the
    /// bottom, with any other countries in between, use "--country 'se,*,dk' --sort
    /// country". It is however important to note that when "*" is given along with other
    /// filter criteria, there is no guarantee that certain countries will be included in
    /// the results. For example, with the options "--country 'se,*,dk' --sort country
    /// --latest 10", the latest 10 mirrors may all be from the United States. When the
    /// glob pattern is present, it only ensures that if certain countries are included in
    /// the results, they will be sorted in the requested order.
    #[arg(long, short, value_name = "country name or code", action = ArgAction::Append)]
    pub country: Vec<String>,

    /// Return the n fastest mirrors that meet the other criteria. Do not use this option
    /// without other filtering options.
    #[arg(long, short, value_name = "n")]
    pub fastest: Option<u16>,

    /// Include servers that match <regex>, where <regex> is a Rust regular express.
    #[arg(long, short, value_name = "regex", action = ArgAction::Append)]
    pub include: Vec<String>,

    /// Exclude servers that match <regex>, where <regex> is a Rust regular expression.
    #[arg(long, short, value_name = "regex", action = ArgAction::Append)]
    pub exclude: Vec<String>,

    /// Limit the list to the n most recently synchronized servers.
    #[arg(long, short, value_name = "n")]
    pub latest: Option<u16>,

    /// Limit the list to the n servers with the highest score.
    #[arg(long, value_name = "n")]
    pub score: Option<u16>,

    /// Return at most n mirrors.
    #[arg(long, short, value_name = "n")]
    pub number: Option<u16>,

    /// Match one of the given protocols, e.g. "https" or "ftp". Multiple protocols may be
    /// selected using commas (e.g. "https,http") or by passing this option multiple times.
    #[arg(long, short, value_delimiter=',', value_name = "protocol", action = ArgAction::Append)]
    pub protocol: Vec<Protocol>,

    /// Set the minimum completion percent for the returned mirrors. Check the mirror
    /// status webpage for the meaning of this parameter.
    #[arg(long, value_name = "[0-100]", default_value_t = 100, value_parser = value_parser!(u8).range(0..=100))]
    pub completion_percent: u8,

    /// Only return mirrors that host ISOs.
    #[arg(long, default_value_t = false)]
    pub isos: bool,

    /// Only return mirrors that support IPv4.
    #[arg(long, default_value_t = false)]
    pub ipv4: bool,

    /// Only return mirrors that support IPv6.
    #[arg(long, default_value_t = false)]
    pub ipv6: bool,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
