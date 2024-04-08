use clap::Parser;

mod feed;
mod input;
mod list;
mod site;
mod util;

use anyhow::{Context, Result};
pub use feed::Feed;
pub use list::{Item, List};
pub use site::{Post, Site};

pub const SITE_URL: &str = "https://example.com/";
pub const FEED_LINK_COUNT: usize = 10;

#[derive(Parser, Debug)]
struct Args {
    /// Path of site source.
    #[clap(long, value_name = "PATH", default_value = "./site/")]
    source: std::path::PathBuf,

    /// Path for generated HTML site.
    #[clap(long, value_name = "PATH", default_value = "./public/")]
    output: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let site_text =
        util::read_directory(&args.source).with_context(|| "Failed to read site data")?;
    let site: Site = idm::from_str(&site_text).with_context(|| "Failed to parse site data")?;
    util::write_directory(&args.output, &site).with_context(|| "Failed to write site web page")?;
    dircpy::copy_dir("static/", &args.output).with_context(|| "Failed to copy static files")?;

    Ok(())
}
