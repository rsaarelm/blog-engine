//! Input types that match the IDM site.

use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Copy, Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    #[default]
    Markdown,
    Outline,
}

#[derive(Default, Debug, Deserialize)]
#[serde(default)]
pub struct Site {
    pub posts: IndexMap<String, ((PostHeader,), String)>,
    pub links: IndexMap<String, ((LinkHeader,), String)>,
}

#[derive(Default, Debug, Deserialize)]
#[serde(default)]
pub struct PostHeader {
    pub title: String,
    pub date: String,
    pub updated: String,
    pub tags: Vec<String>,
    pub format: Format,
}

#[derive(Default, Debug, Deserialize)]
#[serde(default)]
pub struct LinkHeader {
    pub title: String,
    pub added: String,
    pub date: String,
    pub tags: Vec<String>,
    pub sequence: Vec<String>,
}
