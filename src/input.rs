//! Input types that match the IDM site.

use indexmap::IndexMap;
use serde::Deserialize;

use crate::util::Word;

#[derive(Copy, Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    #[default]
    Markdown,
    Outline,
}

#[derive(Default, Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Site {
    pub posts: IndexMap<String, ((PostHeader,), String)>,
    pub links: Vec<(String, ((LinkHeader,), String))>,
    pub tag_hierarchy: IndexMap<String, Vec<String>>,
}

#[derive(Default, Debug, Deserialize)]
#[serde(default)]
pub struct PostHeader {
    pub title: String,
    pub date: String,
    pub updated: String,
    pub tags: Vec<Word>,
    pub format: Format,
}

#[derive(Default, Debug, Deserialize)]
#[serde(default)]
pub struct LinkHeader {
    pub uri: String,
    pub added: String,
    pub date: String,
    pub tags: Vec<Word>,
    pub sequence: Vec<String>,
}
