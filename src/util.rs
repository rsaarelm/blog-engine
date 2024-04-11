use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fs::{self, File},
    io::{self, prelude::*},
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::bail;
use lazy_regex::regex;
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use tldextract::{TldExtractor, TldResult};
use url::Url;

pub const EPOCH: &str = "1970-01-01T00:00:00Z";

#[derive(Default, Debug, Deserialize)]
pub struct Outline(pub Vec<((String,), Outline)>);

impl fmt::Display for Outline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print(f: &mut fmt::Formatter<'_>, depth: usize, outline: &Outline) -> fmt::Result {
            for ((head,), body) in &outline.0 {
                for _ in 0..depth {
                    write!(f, "  ")?;
                }
                writeln!(f, "{head}")?;
                print(f, depth + 1, body)?;
            }
            Ok(())
        }

        print(f, 0, self)
    }
}

impl Outline {
    /// Create list of outline leaves and all the parent headlines leading to
    /// them.
    pub fn full_paths(&self) -> Vec<(&'_ str, Vec<&'_ str>)> {
        fn walk<'a>(
            prefix: Vec<&'a str>,
            outline: &'a Outline,
            output: &mut Vec<(&'a str, Vec<&'a str>)>,
        ) {
            for ((head,), body) in &outline.0 {
                if body.0.is_empty() {
                    output.push((&head, prefix.clone()));
                } else {
                    let mut new_prefix = prefix.clone();
                    new_prefix.push(head);
                    walk(new_prefix, body, output);
                }
            }
        }

        let mut ret = Vec::new();
        walk(Vec::new(), self, &mut ret);
        ret
    }
}

/// Exactly one non-empty word of non-whitespace characters.
#[derive(Clone, Debug, DeserializeFromStr)]
pub struct Word(String);

impl FromStr for Word {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            bail!("Empty word");
        }
        if s.contains(char::is_whitespace) {
            bail!("Word has whitespace");
        }

        Ok(Word(s.to_owned()))
    }
}

impl From<Word> for String {
    fn from(value: Word) -> Self {
        value.0
    }
}

/// Convert a title slug into the corresponding title string.
///
/// NB. Only use as a fallback, this will produce bad results with titles with
/// proper nouns, eg. "the-diaries-of-winston-churchill" -> "The diaries of
/// winston churchill".
///
/// ```
/// assert_eq!(unslugify("post-title"), "Post title");
/// ```
pub fn unslugify(slug: &str) -> String {
    let mut ret = slug.replace('-', " ");
    // Only ASCII-7 text supported for now.
    ret[..1].make_ascii_uppercase();
    ret
}

/// Fill in missing parts of a partial date string that's only a year or only
/// a year and a month. Default to start of the year or the month.
///
/// ```
/// assert_eq!(normalize_date("1984-03"), "1984-03-01T00:00:00Z");
/// ```
pub fn normalize_date(partial_date: &str) -> String {
    let mut ret = partial_date.to_string();
    let skip = ret.chars().count();
    for c in EPOCH.chars().skip(skip) {
        ret.push(c);
    }
    ret
}

/// Dump a directory tree into a single IDM expression.
pub fn read_directory(path: impl AsRef<Path>) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;

    let mut ret = String::new();
    for e in walkdir::WalkDir::new(path) {
        let e = e.expect("read_path failed");
        let depth = e.depth();
        if depth == 0 {
            // The root element, do not print out.
            continue;
        }
        for _ in 1..depth {
            write!(ret, "  ")?;
        }
        let is_dir = e.file_type().is_dir();
        if is_dir {
            writeln!(ret, "{}", e.file_name().to_string_lossy())?;
        } else {
            let path = Path::new(e.file_name());

            if !matches!(
                path.extension()
                    .map(|a| a.to_str().unwrap_or(""))
                    .unwrap_or(""),
                "idm" | "md"
            ) {
                // Only read IDM and Markdown files.
                continue;
            }

            let name = path
                .file_stem()
                .expect("read_path failed")
                .to_string_lossy();
            writeln!(ret, "{}", name)?;

            // Print lines
            let file = File::open(e.path()).expect("read_path: Open file failed");
            for line in io::BufReader::new(file).lines() {
                let line = line.expect("read_path failed");
                let mut ln = &line[..];
                let mut depth = depth;
                // Turn tab indentation into spaces.
                while ln.starts_with('\t') {
                    depth += 1;
                    ln = &ln[1..];
                }
                for _ in 1..(depth + 1) {
                    write!(ret, "  ")?;
                }
                writeln!(ret, "{ln}")?;
            }
        }
    }

    Ok(ret)
}

/// Write a data structure into a directory tree.
///
/// Headlines with a period in them are interpreted as file names, anything
/// above them is considered directories. Field names that start with
/// underscore are flattened and their contents are written into the current
/// level (like serde-flatten, but serde-flatten does not work well with IDM
/// since it loses information about the value type).
///
/// Caveats are that directory names cannot contain periods and file names
/// must contain periods.
pub fn write_directory(root: impl AsRef<Path>, data: &impl Serialize) -> anyhow::Result<()> {
    let output: String = idm::to_string(data)?;

    // Read into tree
    let tree: Outline = idm::from_str(&output)?;

    // Fails if the dir doesn't exist, but this is ok. Ignore the result.
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;

    fn write(path: impl AsRef<Path>, outline: &Outline) -> anyhow::Result<()> {
        for ((head,), body) in &outline.0 {
            if head.contains('.') {
                // It's a file. Write it.
                let path = path.as_ref().join(head);
                if let Some(dir) = path.parent() {
                    // It might contain some dirs too so create those first...
                    fs::create_dir_all(dir)?;
                }
                fs::write(path, &body.to_string())?;
            } else if head.starts_with('_') {
                // HACK: Allow flattening things around a structural element
                // if it's prefixed with an underscore.
                write(path.as_ref(), body)?;
            } else {
                // Treat it as directory.
                // If the name has slashes, they'll generate deeper subdirs.
                let path: PathBuf = path.as_ref().join(head);
                fs::create_dir_all(&path)?;
                // Recurse into body using the new path.
                write(path, body)?;
            }
        }

        Ok(())
    }

    write(root, &tree)
}

/// Strip archive site prefixes from URL.
pub fn canonical_url(url: &str) -> String {
    let rs = [
        regex!(r"^https://web.archive.org/web/\d+/(http.*)$"),
        regex!(r"^https://archive.today/\d+/(http.*)$"),
    ];

    for r in rs {
        if let Some(caps) = r.captures(url) {
            return caps[1].into();
        }
    }

    url.to_owned()
}

/// Extract site name from URL, with special handling for select sites.
pub fn extract_site(url: &str) -> Option<String> {
    let Ok(url) = Url::parse(url) else {
        return None;
    };
    let Some(domain) = url.domain() else {
        return None;
    };

    // Because the internet is cursed, we need to use the TLD database library
    // to split apart domains.
    let extractor = TldExtractor::new(Default::default());
    let Ok(TldResult {
        subdomain,
        domain: Some(main_domain),
        suffix: Some(suffix),
    }) = extractor.extract(domain)
    else {
        return None;
    };

    let truncated_domain = format!("{main_domain}.{suffix}");

    // keep the subdomain if the domain is [subdomain].[any of these]
    let keep_subdomain = [
        "blogspot.com",
        "dreamwidth.org",
        "github.io",
        "ibiblio.org",
        "medium.com",
        "neocities.org",
        "substack.com",
        "tumblr.com",
        "typepad.com",
        "wordpress.com",
    ];

    // Keep the subdomain if it's this exact one.
    let keep_specific_subdomain = ["gist.github.com", "groups.google.com"];

    // XXX: One-off special case for gist.github.com full domain.
    let mut domain = if keep_subdomain.contains(&truncated_domain.as_str())
        || keep_specific_subdomain.contains(&domain)
    {
        if let Some(subdomain) = subdomain {
            format!("{subdomain}.{truncated_domain}")
        } else {
            truncated_domain
        }
    } else {
        truncated_domain
    };

    // Add the first segment if the whole domain string is exactly one of
    // these.
    //
    // Tumblr and medium have both [username].domain.com (keep subdomain) and
    // domain.com/[username] (add segment) style URLs.
    //
    let add_segment = [
        "facebook.com",
        "gist.github.com",
        "github.com",
        "medium.com",
        "oocities.org",
        "scienceblogs.com",
        "tumblr.com",
        "twitter.com",
        "www.facebook.com",
        "x.com",
    ];

    if add_segment.contains(&domain.as_str()) {
        if let Some(mut segs) = url.path_segments() {
            domain.push('/');
            domain.push_str(segs.next().unwrap_or(""));
        }
    }

    Some(domain)
}

pub fn add_topics(tags: &mut Vec<String>, topics: &BTreeMap<String, BTreeSet<String>>) {
    let mut new_tags: Vec<String> = Vec::new();
    let mut redundant: Vec<String> = Vec::new();
    for t in tags.iter() {
        if let Some(us) = topics.get(t) {
            for u in us {
                if tags.contains(u) {
                    redundant.push(u.clone());
                } else if !new_tags.contains(u) {
                    new_tags.push(u.clone());
                }
            }
        }
    }

    // Print some lints.
    // XXX: Lints could be printed with log::warn! instead
    if !redundant.is_empty() {
        eprintln!(
            "List {tags:?} has redundant topic tags: {}",
            redundant.join(", ")
        );
    }
    new_tags.append(tags);
    *tags = new_tags;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_mangle() {
        for (a, b) in [
            ("https://www.example.com/xyzzy", "example.com"),
            ("https://github.com/foozbulator", "github.com/foozbulator"),
            ("https://tumblr.com/user/wotsit", "tumblr.com/user"),
            ("https://user.tumblr.com/wotsit", "user.tumblr.com"),
            ("https://gist.github.com/user", "gist.github.com/user"),
            ("https://www.facebook.com/user", "facebook.com/user"),
            ("https://twitter.com/user", "twitter.com/user"),
            ("https://user.blogspot.com/wotsit", "user.blogspot.com"),
            ("https://cambridge.ac.uk/wotsit", "cambridge.ac.uk"),
            ("https://www.cambridge.ac.uk/wotsit", "cambridge.ac.uk"),
        ] {
            assert_eq!(extract_site(a).unwrap(), b);
        }
    }
}
