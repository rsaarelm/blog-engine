//! Output types that emit templates.

use std::{
    collections::{BTreeMap, HashSet},
    fmt::Write,
};

use askama::Template;
use serde::{Deserialize, Serialize};
use serde_with::SerializeDisplay;

use crate::{
    input::{self, Format},
    util::{self, Outline},
    BLOG_TITLE,
};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(from = "input::Site")]
pub struct Site {
    // Use the magic underscore name to tell the directory writer to flatten
    // posts contents into the top level.
    pub _posts: BTreeMap<String, Post>,
    pub links: Links,
    pub tags: Tags,
    /// Mapping from tags to topic tags, like "topology" -> "math".
    pub topics: BTreeMap<String, String>,

    pub _listing: Listing,
}

impl From<input::Site> for Site {
    fn from(site: input::Site) -> Self {
        let posts: BTreeMap<String, Post> = site
            .posts
            .iter()
            .map(|p| {
                let p = Post::from(p);
                (format!("{}.html", p.slug), p)
            })
            .collect();
        let mut links_page = LinksPage {
            links: site.links.iter().map(From::from).collect(),
        };
        links_page.links.reverse();
        let links_feed = Feed::new(format!("{BLOG_TITLE}: links"), "links/", &links_page.links);

        let mut topics = BTreeMap::default();
        for (topic, tags) in &site.tag_hierarchy {
            for tag in tags {
                topics.insert(tag.clone(), topic.clone());
            }
        }

        for link in links_page.links.iter_mut() {
            link.add_topics(&topics);
        }

        let links = Links {
            page: links_page,
            feed: links_feed,
        };

        let tags = Tags::new(posts.values());

        let listing = Listing::new(BLOG_TITLE, "", posts.values());

        Site {
            _posts: posts,
            links,
            tags,
            topics,
            _listing: listing,
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

impl Post {
    /// List concrete and implicit tags for this post.
    pub fn all_tags(&self) -> HashSet<String> {
        let mut ret = HashSet::new();

        for tag in &self.tags {
            for tag in util::tag_set(tag) {
                ret.insert(tag.to_owned());
            }
        }

        ret
    }
}

impl From<(&String, &((input::PostHeader,), String))> for Post {
    fn from((slug, ((data,), body)): (&String, &((input::PostHeader,), String))) -> Self {
        Post {
            url: format!("{}{}", crate::WEBSITE, slug),
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

#[derive(Default, Debug, Serialize)]
/// Index of all tags.
pub struct Tags {
    /// Sub-pages for different tags
    pub _listings: BTreeMap<String, Listing>,
    #[serde(rename(serialize = "index.html"))]
    pub page: TagIndex,
}

impl Tags {
    pub fn new<'a>(posts: impl IntoIterator<Item = &'a Post>) -> Self {
        let mut tags: BTreeMap<String, usize> = Default::default();
        let mut bins: BTreeMap<String, Vec<&'a Post>> = Default::default();

        for p in posts {
            for tag in p.all_tags() {
                // Update tag counts.
                *tags.entry(tag.to_owned()).or_default() += 1;
                // Store post entry in bin.
                bins.entry(tag.to_owned()).or_default().push(p);
            }
        }

        let page = TagIndex {
            tags: tags.into_iter().collect(),
        };

        let _listings = bins
            .into_iter()
            .map(|(name, posts)| {
                (
                    name.clone(),
                    Listing::new(
                        format!("{BLOG_TITLE}: {name}"),
                        &format!("tags/{name}/"),
                        posts,
                    ),
                )
            })
            .collect();

        Tags { _listings, page }
    }
}

#[derive(Default, Debug, Template, SerializeDisplay)]
#[template(path = "tags.html")]
pub struct TagIndex {
    pub tags: Vec<(String, usize)>,
}

#[derive(Default, Debug, Serialize)]
/// Links listing
pub struct Links {
    #[serde(rename(serialize = "index.html"))]
    pub page: LinksPage,
    #[serde(rename(serialize = "feed.xml"))]
    pub feed: Feed,
}

#[derive(Default, Debug, Template, SerializeDisplay)]
#[template(path = "links.html")]
pub struct LinksPage {
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

impl Link {
    pub fn add_topics(&mut self, topics: &BTreeMap<String, String>) {
        let mut new_tags: Vec<String> = Vec::new();
        let mut redundant: Vec<String> = Vec::new();
        for t in &self.tags {
            if let Some(u) = topics.get(t) {
                if self.tags.contains(u) {
                    redundant.push(u.clone());
                } else if !new_tags.contains(u) {
                    new_tags.push(u.clone());
                }
            }
        }
        if !redundant.is_empty() {
            eprintln!("Lint: Link {} has redundant topic tags: {}",
                self.url, redundant.join(", "));
        }
        new_tags.append(&mut self.tags);
        self.tags = new_tags;
    }
}

impl From<&(String, ((input::LinkHeader,), String))> for Link {
    fn from((title, ((data,), content)): &(String, ((input::LinkHeader,), String))) -> Self {
        let mut title = title.clone();

        // Mark PDF links
        if data.uri.ends_with(".pdf") && (!title.ends_with(".pdf") && !title.ends_with(" (pdf)")) {
            title.push_str(" (pdf)");
        }

        Link {
            url: data.uri.clone(),
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
            id: base64_url::encode(&md5::compute(&data.uri).0),
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
            link: format!("{}{}", crate::WEBSITE, value.slug),
            updated: value.feed_date.clone(),
            content: "".to_owned(),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct PostEntry {
    pub slug: String,
    pub title: String,
    pub date: String,
}

impl From<&Post> for PostEntry {
    fn from(value: &Post) -> Self {
        PostEntry {
            slug: value.slug.clone(),
            title: value.title.clone(),
            date: value.date.clone(),
        }
    }
}

#[derive(Default, Debug, Serialize)]
/// List of articles page.
pub struct Listing {
    #[serde(rename(serialize = "index.html"))]
    page: ListingPage,
    #[serde(rename(serialize = "feed.xml"))]
    feed: Feed,
}

impl Listing {
    pub fn new<'a>(
        title: impl Into<String>,
        tag_path: &str,
        posts: impl IntoIterator<Item = &'a Post>,
    ) -> Self {
        let title = title.into();
        let posts: Vec<&Post> = posts.into_iter().collect();

        let page = ListingPage::new(title.clone(), tag_path, posts.clone());
        let feed = Feed::new(title, tag_path, posts);

        Listing { page, feed }
    }
}

#[derive(Default, Debug, Template, SerializeDisplay)]
#[template(path = "listing.html")]
/// List of articles page.
pub struct ListingPage {
    title: String,
    tag_path: Vec<String>,
    posts: Vec<PostEntry>,
}

impl ListingPage {
    pub fn new<'a>(
        title: impl Into<String>,
        tag_path: &str,
        posts: impl IntoIterator<Item = &'a Post>,
    ) -> Self {
        let mut posts: Vec<PostEntry> = posts.into_iter().map(From::from).collect();
        posts.sort_by_key(|a| a.date.clone());
        posts.reverse();

        ListingPage {
            title: title.into(),
            tag_path: tag_path
                .split('/')
                .filter(|a| !a.trim().is_empty())
                .map(str::to_string)
                .collect(),
            posts,
        }
    }
}

#[derive(Default, Debug, Template, SerializeDisplay)]
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
