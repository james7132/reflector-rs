use anyhow::Result;
use arch_mirrors_rs::{Mirror, Protocol, Status};
use clap::{ArgAction, Args, Parser, ValueEnum, value_parser};
use clap_verbosity_flag::Verbosity;
use jiff::{Span, Timestamp};
use reqwest::Url;
use std::cmp::{Ordering, Reverse};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use xdg::BaseDirectories;

const URL: &str = "https://archlinux.org/mirrors/status/json/";
const DEFAULT_CONNECTION_TIMEOUT: u64 = 5;
const DEFAULT_DOWNLOAD_TIMEOUT: u64 = 5;
const DEFAULT_CACHE_TIMEOUT: u64 = 300;

#[derive(Debug, ValueEnum, Clone, Copy)]
#[allow(
    clippy::doc_markdown,
    reason = "This is used to generate the user facing help."
)]
enum SortTypes {
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
#[allow(
    clippy::doc_markdown,
    reason = "This is used to generate the user facing help."
)]
#[command(
    about,
    author,
    version,
    propagate_version = true,
    next_line_help = false,
    disable_help_subcommand = true,
    after_long_help = "Retrieve and filter a list of the latest Arch Linux mirrors."
)]
struct Cli {
    /// The URL from which to retrieve the mirror data in JSON format. If different from
    /// the default, it must follow the same format.
    #[arg(long, default_value = URL)]
    url: String,

    /// Display a table of the distribution of servers by country.
    #[arg(long)]
    list_countries: bool,

    /// Print extra information to STDERR. Only works with some options.
    #[clap(flatten)]
    verbose: Verbosity,

    #[command(flatten)]
    run: RunOptions,
}

#[derive(Debug, Args)]
#[allow(
    clippy::doc_markdown,
    reason = "This is used to generate the user facing help."
)]
struct RunOptions {
    /// The number of seconds to wait before a connection times out.
    #[arg(long, default_value_t = DEFAULT_CONNECTION_TIMEOUT, value_name = "n")]
    connection_timeout: u64,

    /// The number of seconds to wait before a download times out.
    #[arg(long, default_value_t = DEFAULT_DOWNLOAD_TIMEOUT, value_name = "n")]
    download_timeout: u64,

    /// The cache timeout in seconds for the data retrieved from the Arch Linux Mirror
    /// Status API.
    #[arg(long, default_value_t = DEFAULT_CACHE_TIMEOUT, value_name = "n")]
    cache_timeout: u64,

    /// Save the mirrorlist to the given file path.
    #[arg(long, value_name = "filepath")]
    save: Option<String>,

    /// Sort the mirrorlist by the given field.
    #[arg(long)]
    sort: Option<SortTypes>,

    /// Use n threads for rating mirrors. This option will speed up the rating step but the
    /// results will be inaccurate if the local bandwidth is saturated at any point during
    /// the operation. If rating takes too long without this option then you should
    /// probably apply more filters to reduce the number of rated servers before using this
    /// option.
    #[arg(long, default_value_t = 0)]
    threads: usize,

    /// Print mirror information instead of a mirror list. Filter options apply.
    #[arg(long, default_value_t = false)]
    info: bool,

    #[command(flatten)]
    filters: Filters,
}

#[derive(Parser, Debug)]
#[command(
    next_help_heading = "filters",
    // FIXME: Display this after heading name, currently it is not displayed at all.
    after_long_help = "The following filters are inclusive, i.e. the returned list will only contain mirrors for which all of the given conditions are met."
)]
struct Filters {
    /// Only return mirrors that have synchronized in the last n hours. n may be an integer
    /// or a decimal number.
    #[arg(long, short, value_name = "n")]
    age: Option<f32>,

    /// Only return mirrors with a reported sync delay of n hours or less, where n is a float. For example. to limit the results to mirrors with a reported delay of 15 minutes or less, pass 0.25.
    #[arg(long, value_name = "n")]
    delay: Option<f32>,

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
    country: Vec<String>,

    /// Return the n fastest mirrors that meet the other criteria. Do not use this option
    /// without other filtering options.
    #[arg(long, short, value_name = "n")]
    fastest: Option<u16>,

    /// Include servers that match <regex>, where <regex> is a Rust regular express.
    #[arg(long, short, value_name = "regex", action = ArgAction::Append)]
    include: Vec<String>,

    /// Exclude servers that match <regex>, where <regex> is a Rust regular expression.
    #[arg(long, short, value_name = "regex", action = ArgAction::Append)]
    exclude: Vec<String>,

    /// Limit the list to the n most recently synchronized servers.
    #[arg(long, short, value_name = "n")]
    latest: Option<u16>,

    /// Limit the list to the n servers with the highest score.
    #[arg(long, value_name = "n")]
    score: Option<u16>,

    /// Return at most n mirrors.
    #[arg(long, short, value_name = "n")]
    number: Option<u16>,

    /// Match one of the given protocols, e.g. "https" or "ftp". Multiple protocols may be
    /// selected using commas (e.g. "https,http") or by passing this option multiple times.
    #[arg(long, short, value_delimiter=',', value_name = "protocol", action = ArgAction::Append)]
    protocol: Vec<Protocol>,

    /// Set the minimum completion percent for the returned mirrors. Check the mirror
    /// status webpage for the meaning of this parameter.
    #[arg(long, value_name = "[0-100]", default_value_t = 100, value_parser = value_parser!(u8).range(0..=100))]
    completion_percent: u8,

    /// Only return mirrors that host ISOs.
    #[arg(long, default_value_t = false)]
    isos: bool,

    /// Only return mirrors that support IPv4.
    #[arg(long, default_value_t = false)]
    ipv4: bool,

    /// Only return mirrors that support IPv6.
    #[arg(long, default_value_t = false)]
    ipv6: bool,
}

fn get_cache_file(name: Option<&str>) -> io::Result<PathBuf> {
    let name = name.unwrap_or("mirrorstatus.json");
    let base_dirs = BaseDirectories::new();
    let cache_dir = base_dirs
        .get_cache_home()
        .unwrap_or_else(|| PathBuf::from("~/.cache"));
    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir.join(name))
}

/// Retrieve the mirror status JSON object. The downloaded data will be cached locally and
/// re-used within the cache timeout period. Returns the object and the local cache's
/// modification time.
async fn get_mirror_status(
    http_client: &reqwest::Client,
    run_options: &RunOptions,
    url: &str,
    cache_file_path: Option<PathBuf>,
) -> Result<Status> {
    if let Some(cache_file_path) = cache_file_path {
        let mtime = cache_file_path
            .metadata()
            .ok()
            .and_then(|meta| meta.modified().ok());
        let is_invalid = mtime.is_none_or(|time| {
            let now = SystemTime::now();
            match now.duration_since(time) {
                Ok(elapsed) => elapsed.as_secs() > run_options.cache_timeout,
                Err(_) => true,
            }
        });
        let loaded = if is_invalid {
            let loaded = http_client.get(url).send().await?.json().await?;
            let to_write = serde_json::to_string_pretty(&loaded)?;
            fs::write(cache_file_path, to_write)?;
            loaded
        } else {
            serde_json::from_reader(File::open(cache_file_path)?)?
        };
        Ok(loaded)
    } else {
        Ok(http_client.get(url).send().await?.json().await?)
    }
}

#[derive(PartialEq, Eq, Hash)]
struct Country<'a> {
    country: &'a str,
    code: &'a str,
}

fn count_countries<'a>(
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

struct Metadata<'a> {
    when: Timestamp,
    origin: &'a str,
    retrieved: Timestamp,
    last_check: Timestamp,
}

async fn run(options: &Cli) -> anyhow::Result<()> {
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(options.run.download_timeout))
        .connect_timeout(Duration::from_secs(options.run.connection_timeout))
        .build()?;
    let cache_file = get_cache_file(None).ok();
    let when = Timestamp::now();
    let mut status =
        get_mirror_status(&http_client, &options.run, &options.url, cache_file).await?;

    if options.list_countries {
        list_countries(&status);
        return Ok(());
    }

    filter_status(&options.run.filters, &mut status);
    sort_status(&options.run, &http_client, &mut status).await;

    let metadata = Metadata {
        when,
        origin: options.url.as_ref(),
        retrieved: when,
        last_check: when,
    };

    if let Some(path) = &options.run.save {
        File::create(path)
            .and_then(move |file| format_output(&metadata, status.urls.iter(), file))?;
    } else {
        format_output(&metadata, status.urls.iter(), io::stdout())?;
    }

    Ok(())
}

fn format_output<'a>(
    metadata: &Metadata,
    mirrors: impl Iterator<Item = &'a Mirror>,
    mut out: impl Write,
) -> io::Result<()> {
    let command = std::env::args().collect::<Vec<_>>().join(" ");
    writeln!(
        out,
        "################################################################################\n\
         ################# Arch Linux mirrorlist generated by Reflector #################\n\
         ################################################################################\n"
    )?;
    writeln!(
        out,
        "# With:       {}\n# When:       {}\n# From:       {}\n# Retrieved:  {}\n# Last Check: {}\n",
        command, metadata.when, metadata.origin, metadata.retrieved, metadata.last_check
    )?;
    for mirror in mirrors {
        writeln!(out, "Server = {}$repo/os/$arch", mirror.url)?;
    }
    Ok(())
}

async fn sort_status(run_options: &RunOptions, http_client: &reqwest::Client, status: &mut Status) {
    match run_options.sort {
        Some(SortTypes::Age) => status.urls.sort_by_key(|mir| mir.last_sync),
        Some(SortTypes::Rate) => {
            let rates = rate_status(run_options, http_client, status).await;
            status
                .urls
                .sort_by(|a, b| match (rates.get(&a.url), rates.get(&b.url)) {
                    (Some(rate_a), Some(rate_b)) => rate_a
                        .partial_cmp(rate_b)
                        .unwrap_or(Ordering::Equal)
                        .reverse(),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => Ordering::Equal,
                });
        }
        Some(SortTypes::Country) => status.urls.sort_by(|a, b| a.country.cmp(&b.country)),
        Some(SortTypes::Score) => status.urls.sort_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(Ordering::Equal)
                .reverse()
        }),
        Some(SortTypes::Delay) => status.urls.sort_by_key(|mir| Reverse(mir.delay)),
        None => {}
    }
}

#[allow(clippy::cast_precision_loss)]
async fn rate_status(
    run_options: &RunOptions,
    http_client: &reqwest::Client,
    status: &Status,
) -> HashMap<Url, f64> {
    const DB_FILENAME: &str = "extra.db";
    const DB_SUBPATH: &str = "extra/os/x86_64/extra.db";

    let mut task_set = JoinSet::<anyhow::Result<(Url, f64)>>::new();
    let mut rates = HashMap::with_capacity(status.urls.len());
    let semaphore = Arc::new(Semaphore::new(run_options.threads.max(1)));
    let connection_timeout = run_options.connection_timeout;

    for mirror in &status.urls {
        let url = mirror.url.clone();
        let semaphore = semaphore.clone();
        match mirror.protocol {
            Protocol::Http | Protocol::Https => {
                let task_client = http_client.clone();
                task_set.spawn(async move {
                    let _guard = semaphore.acquire().await?;
                    let db_url = url.join(DB_SUBPATH)?;
                    let start = Instant::now();
                    let content_length = task_client.get(db_url).send().await?.bytes().await?.len();
                    let micros = Instant::elapsed(&start).as_secs_f64();
                    let rate = (content_length as f64) / micros;
                    Ok((url, rate))
                });
            }
            Protocol::Rsync => {
                task_set.spawn(async move {
                    let _guard = semaphore.acquire().await?;
                    let temp_dir = tempdir::TempDir::new("reflector")?;
                    let db_url = url.join(DB_SUBPATH)?;

                    let start = Instant::now();
                    let exit_status = tokio::process::Command::new("rsync")
                        .arg("-avL")
                        .arg("--no-h")
                        .arg("--no-motd")
                        .arg(format!("--contimeout={connection_timeout}"))
                        .arg(db_url.as_str())
                        .arg(temp_dir.path())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()?
                        .wait()
                        .await?;

                    if !exit_status.success() {
                        return Err(anyhow::anyhow!(exit_status));
                    }

                    let micros = Instant::elapsed(&start).as_secs_f64();
                    let file_path = Path::join(temp_dir.path(), DB_FILENAME);
                    let content_length = std::fs::metadata(file_path)?.len();

                    let rate = (content_length as f64) / micros;
                    Ok((url, rate))
                });
            }
        }
    }

    while let Some(result) = task_set.join_next().await {
        match result {
            Ok(Ok((url, rate))) => {
                rates.insert(url, rate);
            }
            Ok(Err(err)) => eprintln!("error while rating mirror: {err}"),
            Err(err) => eprintln!("error while rating mirror: {err}"),
        }
    }

    rates
}

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
fn filter_status(filters: &Filters, status: &mut Status) {
    let now = Timestamp::now();
    let min_completion_pct = f64::from(filters.completion_percent) / 100.0;
    let max_age = filters
        .age
        .and_then(|age| Span::new().try_hours(age as i64).ok());
    status.urls.retain(move |mirror| {
        if let Some(last_sync) = mirror.last_sync {
            // Filter by age. The age is given in hours and converted to seconds. Servers
            // with a last refresh older than the age are omitted.
            if let Some(max_age) = max_age {
                if matches!(max_age.compare(Span::new()), Ok(Ordering::Greater))
                    && last_sync + max_age < now
                {
                    return false;
                }
            }
        } else {
            // Filter unsynced mirrors.
            return false;
        }

        // Filter by completion "percent" [0-1].
        if let Some(completion_pct) = mirror.completion_pct {
            if completion_pct < min_completion_pct {
                return false;
            }
        }

        if !filters.country.is_empty()
            && !filters.country.contains(&mirror.country)
            && !filters.country.contains(&mirror.country_code)
        {
            return false;
        }

        // Filter by protocols.
        if !filters.protocol.is_empty() && !filters.protocol.contains(&mirror.protocol) {
            return false;
        }

        // Filter by delay. The delay is given as a float of hours and must be
        // converted to seconds.
        if let Some(delay) = filters.delay {
            let max_delay = (delay * 3600.0) as u32;
            if let Some(mirror_delay) = mirror.delay {
                if mirror_delay > max_delay {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Filter by ISO hosing.
        if filters.isos && !mirror.isos {
            return false;
        }

        // Filter by IPv4 support.
        if filters.ipv4 && !mirror.ipv4 {
            return false;
        }

        // Filter by IPv6 support.
        if filters.ipv6 && !mirror.ipv6 {
            return false;
        }

        true
    });
}

fn list_countries(status: &Status) {
    let counts = count_countries(&status.urls);
    let mut sorted = vec![];
    for (country, count) in counts {
        sorted.push((country, count));
    }
    sorted.sort_by(|c1, c2| c1.0.code.cmp(c2.0.code));

    let country_width = sorted
        .iter()
        .map(|(c, _)| c.country.len())
        .max()
        .unwrap_or(0)
        .max("Country".len());
    let code_width = sorted
        .iter()
        .map(|(c, _)| c.code.len())
        .max()
        .unwrap_or(0)
        .max("Code".len());
    let count_width = sorted
        .iter()
        .map(|(_, c)| c.ilog(10) as usize)
        .max()
        .unwrap_or(0)
        .max("Count".len());

    println!(
        "{0:1$} {2:3$} {4:5$}",
        "Country", country_width, "Code", code_width, "Count", count_width
    );
    println!(
        "{0:1$} {2:3$} {4:5$}",
        "=======", country_width, "====", code_width, "=====", count_width
    );
    for (country, count) in sorted {
        println!(
            "{0:1$} {2:3$} {4:5$}",
            country.country, country_width, country.code, code_width, count, count_width
        );
    }
}

fn main() {
    let cli = Cli::parse();
    let maybe_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(cli.run.threads.max(1))
        .build();

    let result = match maybe_runtime {
        Ok(runtime) => runtime.block_on(run(&cli)),
        Err(err) => {
            eprintln!("error: {err}");
            return;
        }
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
    }
}
