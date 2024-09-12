//! Input types that match the IDM site.

use indexmap::IndexMap;
use serde::Deserialize;

use crate::util::{Outline, Word};

#[derive(Copy, Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    /// Markdown-formatted post.
    #[default]
    Markdown,
    /// Indented lines outline formatted post.
    Outline,
}

/// Main site structure, the entire `site/` subdirectory is deserialized into
/// `Site`. All top-level elements are serialized here.
#[derive(Default, Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Site {
    /// Blog posts authored by the site owner.
    pub posts: IndexMap<String, ((PostHeader,), String)>,
    /// Links to external sites.
    pub links: Vec<(String, ((LinkHeader,), String))>,
    /// A tree of topic tags that will be automatically added if a sub-tag is
    /// present.
    ///
    /// For example having a hierarchy of
    ///
    /// ```notrust
    /// math
    ///   topology
    /// ```
    ///
    /// causes any item tagged with `topology` to automatically have a `math`
    /// tag added.
    pub tag_hierarchy: Outline,
    pub settings: Settings,
}

#[derive(Default, Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PostHeader {
    /// Title of the post.
    pub title: String,
    /// Date the post was published.
    pub date: String,
    /// Date the post was updated (can be empty).
    pub updated: String,
    /// Topic tags for the post.
    pub tags: Vec<Word>,
    /// Format of the post content.
    ///
    /// Default is markdown, but other formats can be supported as well,
    /// controls how the post content is turned into HTML.
    pub format: Format,
}

#[derive(Default, Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct LinkHeader {
    /// Link URI.
    ///
    /// Currently supports just URLs, but could also support ISBNs for books
    /// etc.
    pub uri: String,
    /// Mirror URL for link, if the canonical URI is paywalled.
    pub mirror: Option<String>,
    /// Date when the link was added to the links list, may be empty.
    pub added: String,
    /// Date when the link's content was originally published.
    pub date: String,
    /// Topic tags for the link.
    pub tags: Vec<Word>,
    /// Subsequent URLs if the link refers to a multi-part series.
    pub sequence: Vec<String>,
}

/// Site configuration.
#[derive(Default, Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Settings {
    /// Name of the site.
    pub site_name: String,
    /// Base URL the site is being deployed to.
    pub base_url: String,
    /// Default author for posts.
    pub author: String,
}
