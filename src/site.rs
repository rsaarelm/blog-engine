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
    Feed, List,
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
            .map(|p| {
                let p = Post::from(p);
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

        for (_, post) in posts.iter_mut() {
            util::add_topics(&mut post.tags, &topics);
        }

        let index = List::new("rsaarelm's blog", "feed.xml", &posts);

        let mut links = List::new("rsaarelm's bookmarks", "feed-links.xml", &site.links);

        for link in links.items.iter_mut() {
            util::add_topics(&mut link.tags, &topics);
        }

        let feed = Feed::new(
            "rsaarelm's blog",
            "Risto Saarelma",
            "feed.xml",
            posts.values(),
        );

        let links_feed = Feed::new(
            "rsaarelm's links",
            "Risto Saarelma",
            "feed-links.xml",
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
    pub slug: String,
    pub title: String,
    pub date: String,
    pub updated: String,
    pub feed_date: String,
    pub tags: Vec<String>,
    pub content: String,
}

impl From<(&String, &((input::PostHeader,), String))> for Post {
    fn from((slug, ((data,), body)): (&String, &((input::PostHeader,), String))) -> Self {
        Post {
            url: format!("{}{}", crate::SITE_URL, slug),
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
