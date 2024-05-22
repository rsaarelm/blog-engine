use askama::Template;
use serde_with::SerializeDisplay;

use crate::{input, util, Post};

#[derive(Default, Debug, Template, SerializeDisplay)]
#[template(path = "list.html")]
pub struct List {
    pub title: String,
    pub feed_path: String,
    pub items: Vec<Item>,
}

impl List {
    pub fn new<T: Into<Item>>(
        title: impl Into<String>,
        feed_path: impl Into<String>,
        items: impl IntoIterator<Item = T>,
    ) -> Self {
        let mut items: Vec<Item> = items.into_iter().map(Into::into).collect();
        items.sort_by(|a, b| b.date.cmp(&a.date));

        List {
            title: title.into(),
            feed_path: feed_path.into(),
            items,
        }
    }
}

#[derive(Default, Debug)]
pub struct Item {
    /// URL of the item.
    pub url: String,

    /// Item URL's website.
    ///
    /// Empty for local links, can have special treatment for some URLs.
    pub site: String,

    /// Whether the URL is an archive link and the original is presumably no
    /// longer accessible.
    pub is_archived: bool,

    /// Alternative URL
    pub bypass: String,

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
    pub fn is_external(&self) -> bool {
        self.url.starts_with("http://") || self.url.starts_with("https://")
    }
}

impl From<&(String, ((input::LinkHeader,), String))> for Item {
    fn from((title, ((data,), content)): &(String, ((input::LinkHeader,), String))) -> Self {
        let mut title = title.clone();

        // Mark PDF links
        if data.uri.ends_with(".pdf") && (!title.ends_with(".pdf") && !title.ends_with(" (pdf)")) {
            title.push_str(" (pdf)");
        }

        let canonical_url = util::canonical_url(&data.uri);
        let is_archived = canonical_url != data.uri;

        let site = if let Some(site) = util::extract_site(&canonical_url) {
            site
        } else {
            String::new()
        };

        let bypass = if let Some(mirror) = &data.mirror {
            mirror.clone()
        } else if site == "doi.org" {
            format!("https://sci-hub.se/{}", data.uri)
        } else {
            String::new()
        };

        Item {
            url: data.uri.clone(),
            site,
            is_archived,
            bypass,
            title,
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
            id: base64_url::encode(&md5::compute(&canonical_url).0),
        }
    }
}

impl From<(&String, &Post)> for Item {
    fn from((_, post): (&String, &Post)) -> Self {
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
}
