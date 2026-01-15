mod cli;
mod mirrors;

pub use cli::Cli;
use comfy_table::{Cell, CellAlignment, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use mirrors::{count_countries, get_cache_file, get_mirror_status};

pub async fn run(options: &Cli) {
    if options.list_countries {
        list_countries(&options.url).await;
    }
}

pub async fn list_countries(url: &str) {
    let cache_file = get_cache_file(None);
    let status = get_mirror_status(10, 10, url, &cache_file).await.unwrap();
    let counts = count_countries(&status.urls).await;
    let mut sorted = vec![];
    for (country, count) in counts {
        sorted.push((country, count));
    }
    sorted.sort_by(|c1, c2| c1.0.code.cmp(&c2.0.code));

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Country", "Code", "Count"]);

    for (country, count) in sorted {
        table.add_row(vec![
            Cell::new(country.country.to_string()),
            Cell::new(country.code.to_string()),
            Cell::new(count.to_string()).set_alignment(CellAlignment::Right),
        ]);
    }
    println!("{}", table);
}
