mod input;
mod site;
mod util;

use site::Site;

pub const WEBSITE: &str = "https://example.com/";
pub const BLOG_TITLE: &str = "EXAMPLE blog";
pub const FEED_LINK_COUNT: usize = 10;

fn main() {
    let site_text = util::read_directory("site/").expect("Failed to read site data");
    let site: Site = idm::from_str(&site_text).expect("Failed to parse site data");
    util::write_directory("public/", &site).expect("Failed to write site web page");
    dircpy::copy_dir("static/", "public/").unwrap();
}
