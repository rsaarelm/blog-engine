//! Output types that emit templates.

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;

use askama::Template;
use serde::Deserialize;

use crate::{input::{self, Format}, util};

#[derive(Default, Debug, Deserialize)]
#[serde(from = "input::Site")]
pub struct Site {
    pub posts: Vec<Post>,
    pub links: Links,
    pub tags: Tags,
}

impl Site {
    /// Return full set of tags in the existing posts.
    pub fn post_tags(&self) -> HashSet<String> {
        let mut ret: HashSet<String> = Default::default();

        for post in &self.posts {
            for tag in &post.tags {
                for tag in util::tag_set(tag) {
                    ret.insert(tag.to_owned());
                }
            }
        }

        ret
    }
}

impl From<input::Site> for Site {
    fn from(site: input::Site) -> Self {
        let posts: Vec<Post> = site.posts.iter().map(From::from).collect();
        let mut links = Links {
            links: site.links.iter().map(From::from).collect(),
        };
        links.links.reverse();

        let mut tags: BTreeMap<String, usize> = Default::default();
        for post in &posts {
            for tag in &post.tags {
                let name = tag.to_string();
                *tags.entry(name).or_insert(0) += 1;
            }
        }

        let tags = Tags {
            tags: tags.into_iter().collect(),
        };

        Site { posts, links, tags }
    }
}

#[derive(Clone, Default, Debug, Template)]
#[template(path = "post.html")]
pub struct Post {
    pub url: String,
    pub slug: String,
    pub title: String,
    pub date: String,
    pub updated: String,
    pub feed_date: String,
    pub tags: Vec<String>,
    pub content: String,
}

impl Post {
    pub fn matches_tag(&self, tag: &str) -> bool {
        let prefix = format!("{tag}/");

        self.tags.iter().any(|t| t == tag || t.starts_with(&prefix))
    }
}

impl From<(&String, &((input::PostHeader,), String))> for Post {
    fn from((slug, ((data,), body)): (&String, &((input::PostHeader,), String))) -> Self {
        Post {
            url: format!("{}{}/", crate::WEBSITE, slug),
            slug: slug.clone(),
            // Generate a title from the slug if not specified.
            title: if data.title.is_empty() {
                util::unslugify(slug)
            } else {
                data.title.clone()
            },
            date: data.date.clone(),
            updated: data.updated.clone(),
            feed_date: if !data.updated.is_empty() {
                util::normalize_date(&data.updated)
            } else if !data.date.is_empty() {
                util::normalize_date(&data.date)
            } else {
                util::EPOCH.to_owned()
            },
            tags: data.tags.clone(),

            content: match data.format {
                Format::Markdown => {
                    // Convert markdown content to HTML.
                    let mut html = String::new();
                    pulldown_cmark::html::push_html(
                        &mut html,
                        pulldown_cmark::Parser::new(body),
                    );
                    html
                }
                Format::Outline => {
                    fn push(buf: &mut String, outline: &Outline) {
                        if outline.0.is_empty() {
                            return;
                        }
                        let _ = write!(buf, "<ul class='outline'>");
                        for ((head,), body) in &outline.0 {
                            let _ = write!(buf, "<li>{head}");
                            push(buf, body);
                            let _ = write!(buf, "</li>");
                        }
                        let _ = write!(buf, "</ul>");
                    }
                    #[derive(Deserialize)]
                    struct Outline(Vec<((String,), Outline)>);
                    let body: Outline = idm::from_str(body).expect("Bad outline body");
                    let mut ret = String::new();
                    push(&mut ret, &body);
                    ret
                }
            },
        }
    }
}

#[derive(Default, Debug, Template)]
#[template(path = "tags.html")]
/// Tags listing
pub struct Tags {
    pub tags: Vec<(String, usize)>,
}

#[derive(Default, Debug, Template)]
#[template(path = "links.html")]
pub struct Links {
    pub links: Vec<Link>,
}

#[derive(Default, Debug)]
pub struct Link {
    pub url: String,
    pub title: String,
    pub added: String,
    pub date: String,
    pub feed_date: String,
    pub tags: Vec<String>,
    pub sequence: Vec<String>,
    pub content: String,
    pub id: String,
}

impl From<(&String, &((input::LinkHeader,), String))> for Link {
    fn from((url, ((data,), content)): (&String, &((input::LinkHeader,), String))) -> Self {
        // Use URL as title if input didn't specify a title.
        let mut title = if data.title.is_empty() {
            url.clone()
        } else {
            data.title.clone()
        };

        // Mark PDF links
        if url.ends_with(".pdf") && (!title.ends_with(".pdf") && !title.ends_with(" (pdf)")) {
            title.push_str(" (pdf)");
        }

        Link {
            url: url.clone(),
            title,
            added: data.added.clone(),
            date: data.date.clone(),
            feed_date: if !data.added.is_empty() {
                util::normalize_date(&data.added)
            } else if !data.date.is_empty() {
                util::normalize_date(&data.date)
            } else {
                util::EPOCH.to_owned()
            },
            tags: data.tags.clone(),
            sequence: data.sequence.clone(),
            content: {
                let mut html = String::new();
                pulldown_cmark::html::push_html(&mut html, pulldown_cmark::Parser::new(content));
                html
            },
            id: base64_url::encode(&md5::compute(url).0),
        }
    }
}

pub struct FeedEntry {
    pub title: String,
    pub link: String,
    pub updated: String,
    pub content: String,
}

impl From<&Link> for FeedEntry {
    fn from(value: &Link) -> Self {
        FeedEntry {
            title: value.title.clone(),
            link: format!("{}links/#{}", crate::WEBSITE, value.id),
            updated: value.feed_date.clone(),
            content: format!("<a href='{}'>{}</a>", value.url, value.title),
        }
    }
}

impl From<&Post> for FeedEntry {
    fn from(value: &Post) -> Self {
        FeedEntry {
            title: value.title.clone(),
            link: format!("{}{}/", crate::WEBSITE, value.slug),
            updated: value.feed_date.clone(),
            content: "".to_owned(),
        }
    }
}

#[derive(Default, Template)]
#[template(path = "listing.html")]
/// List of articles page.
pub struct Listing<'a> {
    title: String,
    tag_path: Vec<String>,
    posts: Vec<&'a Post>,
}

impl<'a> Listing<'a> {
    pub fn new(
        title: impl Into<String>,
        tag: &str,
        posts: impl IntoIterator<Item = &'a Post>,
    ) -> Self {
        let mut posts: Vec<&'a Post> = posts.into_iter().collect();
        posts.sort_by_key(|a| &a.date);
        posts.reverse();

        Listing {
            title: title.into(),
            tag_path: tag
                .split('/')
                .filter(|a| !a.trim().is_empty())
                .map(str::to_string)
                .collect(),
            posts,
        }
    }
}

#[derive(Default, Template)]
#[template(path = "feed.xml")]
/// Atom feed.
pub struct Feed {
    title: String,
    updated: String,
    link: String,
    entries: Vec<FeedEntry>,
}

impl Feed {
    pub fn new<T: Into<FeedEntry>>(
        title: impl Into<String>,
        path: &str,
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

        Feed {
            title: title.into(),
            updated,
            link: format!("{}{}", crate::WEBSITE, path),
            entries,
        }
    }
}
