use askama::Template;
use serde_with::SerializeDisplay;

use crate::{util, Item, Post};

#[derive(Default, Debug, Template, SerializeDisplay)]
#[template(path = "feed.xml")]
/// Atom feed.
pub struct Feed {
    title: String,
    author: String,
    updated: String,
    link: String,
    feed_link: String,
    entries: Vec<FeedEntry>,
}

// TODO All feeds are at root, have different filenames, change behavior of
// path parameter and the feed.xml template.

impl Feed {
    pub fn new<T: Into<FeedEntry>>(
        title: &str,
        author: &str,
        filename: &str,
        entries: impl IntoIterator<Item = T>,
    ) -> Self {
        let mut entries: Vec<FeedEntry> = entries.into_iter().map(Into::into).collect();
        entries.sort_by(|a, b| a.updated.cmp(&b.updated));
        if entries.len() > crate::FEED_LINK_COUNT {
            entries = entries.split_off(entries.len() - crate::FEED_LINK_COUNT);
        }

        let updated = entries
            .iter()
            .map(|a| &a.updated)
            .max()
            .cloned()
            .unwrap_or(util::EPOCH.to_owned());

        let link = crate::SITE_URL.to_owned();
        let feed_link = format!("{link}{filename}");

        Feed {
            title: title.into(),
            author: author.into(),
            updated,
            link,
            feed_link,
            entries,
        }
    }
}

#[derive(Debug)]
pub struct FeedEntry {
    pub title: String,
    pub link: String,
    pub updated: String,
    pub content: String,
}

impl From<&Item> for FeedEntry {
    fn from(value: &Item) -> Self {
        FeedEntry {
            title: value.title.clone(),
            link: format!("{}links#{}", crate::SITE_URL, value.id),
            updated: value.feed_date.clone(),
            content: format!(
                "<a href='{}'>{}</a> ({})<br/>{} {}",
                value.url,
                value.title,
                value.site,
                value.date,
                value.tags.join(", ")
            ),
        }
    }
}

impl From<&Post> for FeedEntry {
    fn from(value: &Post) -> Self {
        FeedEntry {
            title: value.title.clone(),
            link: format!("{}{}", crate::SITE_URL, value.slug),
            updated: value.feed_date.clone(),
            content: "".to_owned(),
        }
    }
}