//! Output types that emit templates.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};

use askama::Template;
use serde::{Deserialize, Serialize};
use serde_with::SerializeDisplay;

use crate::{
    input::{self, Format},
    util::{self, Outline},
    Feed, Item, List,
};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(from = "input::Site")]
pub struct Site {
    // Use the magic underscore name to tell the directory writer to flatten
    // posts contents into the top level.
    pub _posts: BTreeMap<String, Post>,

    #[serde(rename(serialize = "index.html"))]
    pub index: List,

    #[serde(rename(serialize = "feed.xml"))]
    pub feed: Feed,

    #[serde(rename(serialize = "links.html"))]
    pub links: List,

    #[serde(rename(serialize = "feed-links.xml"))]
    pub links_feed: Feed,
}

impl From<input::Site> for Site {
    fn from(site: input::Site) -> Self {
        let mut posts: BTreeMap<String, Post> = site
            .posts
            .iter()
            .map(|(slug, ((data,), body))| {
                let p = Post::new(&site.settings, slug, data, body);
                (format!("{}.html", p.slug), p)
            })
            .collect();

        let mut topics: BTreeMap<String, BTreeSet<String>> = Default::default();

        for (tag, path) in site.tag_hierarchy.full_paths() {
            let bag = topics.entry(tag.to_owned()).or_default();
            for t in path {
                bag.insert(t.to_string());
            }
        }

        for (title, post) in posts.iter_mut() {
            util::add_topics(title, &mut post.tags, &topics);
        }

        let index = List::new(
            &site.settings.site_name,
            "posts",
            "feed.xml",
            posts.values().map(Item::new_post),
        );

        let mut links = List::new(
            format!("{}: Bookmarks", site.settings.site_name),
            "links",
            "feed-links.xml",
            site.links.iter().map(|(title, ((data,), content))| {
                Item::new_bookmark(&site.settings, title, data, content)
            }),
        );

        let mut seen_links: BTreeSet<String> = Default::default();
        for link in links.items.iter_mut() {
            // Check for duplicate links
            if !link.url.is_empty() {
                if seen_links.contains(&link.url) {
                    eprintln!("Duplicate link URL: {}", link.url);
                }
                seen_links.insert(link.url.clone());
            }

            util::add_topics(&link.title, &mut link.tags, &topics);
        }

        let feed = Feed::new(
            &site.settings.base_url,
            &site.settings.site_name,
            &site.settings.author,
            &format!("{}feed.xml", site.settings.base_url),
            posts.values(),
        );

        let links_feed = Feed::new(
            &format!("{}links", site.settings.base_url),
            &format!("{}: Bookmarks", site.settings.site_name),
            &site.settings.author,
            &format!("{}feed-links.xml", site.settings.base_url),
            &links.items,
        );

        Site {
            _posts: posts,
            index,
            feed,
            links,
            links_feed,
        }
    }
}

#[derive(Clone, Default, Debug, Template, SerializeDisplay)]
#[template(path = "post.html")]
pub struct Post {
    pub url: String,
    /// Only used for lists, always empty for posts.
    pub id: String,
    pub slug: String,
    pub title: String,
    pub date: String,
    pub updated: String,
    pub feed_date: String,
    pub tags: Vec<String>,
    pub content: String,
}

impl Post {
    pub fn new(
        settings: &input::Settings,
        slug: &str,
        data: &input::PostHeader,
        body: &str,
    ) -> Self {
        Post {
            url: format!("{}{}", settings.base_url, slug),
            id: Default::default(),
            slug: slug.to_string(),
            // Generate a title from the slug if not specified.
            title: if data.title.is_empty() {
                util::unslugify(slug)
            } else {
                data.title.clone()
            },
            date: data.date.clone(),
            updated: data.updated.clone(),
            feed_date: if !data.date.is_empty() {
                util::normalize_date(&data.date)
            } else {
                util::EPOCH.to_owned()
            },
            tags: data.tags.iter().cloned().map(String::from).collect(),

            content: match data.format {
                Format::Markdown => {
                    // Convert markdown content to HTML.
                    let mut html = String::new();
                    pulldown_cmark::html::push_html(&mut html, pulldown_cmark::Parser::new(body));
                    html
                }
                Format::Outline => {
                    fn push(buf: &mut String, outline: &Outline) {
                        if outline.0.is_empty() {
                            return;
                        }
                        let _ = write!(buf, "<ul class='outline'>");
                        for ((head,), body) in &outline.0 {
                            if head.is_empty() {
                                let _ = write!(buf, "<li><br/>");
                            } else {
                                let _ = write!(buf, "<li>{head}");
                            }
                            push(buf, body);
                            let _ = write!(buf, "</li>");
                        }
                        let _ = write!(buf, "</ul>");
                    }
                    let body: Outline = idm::from_str(body).expect("Bad outline body");
                    let mut ret = String::new();
                    push(&mut ret, &body);
                    ret
                }
            },
        }
    }
}
