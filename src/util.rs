use std::{
    fmt,
    fs::{self, File},
    io::{self, prelude::*},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

pub const EPOCH: &str = "1970-01-01T00:00:00Z";

#[derive(Deserialize)]
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

pub fn tag_set(tag: &str) -> Vec<&str> {
    let mut ret = vec![tag];
    for (i, c) in tag.char_indices() {
        if c == '/' {
            ret.push(&tag[0..i]);
        }
    }

    ret
}
