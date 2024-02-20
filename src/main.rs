use clap::Parser;

mod input;
mod site;
mod util;

use site::Site;

pub const WEBSITE: &str = "https://example.com/";
pub const BLOG_TITLE: &str = "EXAMPLE blog";
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

fn main() {
    let args = Args::parse();

    let site_text = util::read_directory(&args.source).expect("Failed to read site data");
    let site: Site = idm::from_str(&site_text).expect("Failed to parse site data");
    util::write_directory(&args.output, &site).expect("Failed to write site web page");
    dircpy::copy_dir("static/", &args.output).unwrap();
}
