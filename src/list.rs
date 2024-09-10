use askama::Template;
use serde_with::SerializeDisplay;

use crate::{
    input,
    util::{self, Tag},
    Post,
};

#[derive(Default, Debug, Template, SerializeDisplay)]
#[template(path = "list.html")]
pub struct List {
    pub title: String,
    /// Identifier for template to deactivate banner link to this list.
    pub id: String,
    pub feed_path: String,
    pub items: Vec<Item>,
    /// Tag cloud.
    pub tags: Vec<Tag>,
}

impl List {
    pub fn new(
        title: impl Into<String>,
        id: impl Into<String>,
        feed_path: impl Into<String>,
        items: impl IntoIterator<Item = Item>,
    ) -> Self {
        let mut items: Vec<Item> = items.into_iter().collect();
        items.sort_by(|a, b| b.date.cmp(&a.date));

        let tags = util::build_tag_list(items.iter().map(|a| a.tags.as_ref()));

        List {
            title: title.into(),
            id: id.into(),
            feed_path: feed_path.into(),
            items,
            tags,
        }
    }
}

#[derive(Default, Debug)]
pub struct Item {
    /// URL to local site's bookmark list.
    pub home_url: String,

    /// URL of the item.
    pub url: String,

    /// Item URL's website.
    ///
    /// Empty for local links, can have special treatment for some URLs.
    pub site: String,

    /// Whether the URL is an archive link and the original is presumably no
    /// longer accessible.
    pub is_archived: bool,

    /// Original URL in case it's not usable and main link is a mirror.
    pub original: String,

    /// Title of the target page.
    pub title: String,

    /// Publication date of item.
    pub date: String,

    /// Date that the item should have in RSS feed.
    ///
    /// Preferrably `added`, if that's not available then `date`.
    pub feed_date: String,

    /// List of tags for the item.
    pub tags: Vec<String>,

    /// Sequence of subsequent URLs for a multi-part sequence item.
    pub sequence: Vec<String>,

    /// Optional note text in HTML.
    pub preview: String,

    /// Local anchor ID hashed from URL.
    pub id: String,
}

impl Item {
    pub fn new_bookmark(
        settings: &input::Settings,
        title: &str,
        data: &input::LinkHeader,
        content: &str,
    ) -> Self {
        let mut title = title.to_owned();

        // Mark PDF links
        let file_looks_like_pdf = data.uri.ends_with(".pdf")
            || data.mirror.as_ref().map_or(false, |a| a.ends_with(".pdf"));
        if file_looks_like_pdf && (!title.ends_with(".pdf") && !title.ends_with(" (pdf)")) {
            title.push_str(" (pdf)");
        }

        let canonical_url = util::canonical_url(&data.uri);
        let is_archived = canonical_url != data.uri;

        let site = util::extract_site(&canonical_url).unwrap_or_default();

        let mut url = data.uri.clone();
        let mut original = String::new();

        if let Some(mirror) = &data.mirror {
            original = url;
            url = mirror.clone();
        } else if site == "doi.org" {
            original = url;
            url = format!("https://sci-hub.se/{}", data.uri);
        }

        let id = base64_url::encode(&md5::compute(&canonical_url).0);

        Item {
            home_url: format!("{}links#{}", settings.base_url, id),
            url,
            site,
            is_archived,
            original,
            title: title.to_owned(),
            date: data.date.clone(),
            feed_date: if !data.added.is_empty() {
                util::normalize_date(&data.added)
            } else if !data.date.is_empty() {
                util::normalize_date(&data.date)
            } else {
                util::EPOCH.to_owned()
            },
            tags: data.tags.iter().cloned().map(String::from).collect(),
            sequence: data.sequence.clone(),
            preview: {
                let mut html = String::new();
                pulldown_cmark::html::push_html(&mut html, pulldown_cmark::Parser::new(content));
                html
            },
            id,
        }
    }

    pub fn new_post(post: &Post) -> Self {
        Item {
            url: post.slug.clone(),
            title: post.title.clone(),
            date: post.date.clone(),
            feed_date: post.feed_date.clone(),
            tags: post.tags.clone(),
            id: post.slug.clone(),
            ..Default::default()
        }
    }

    pub fn is_external(&self) -> bool {
        self.url.starts_with("http://") || self.url.starts_with("https://")
    }
}
