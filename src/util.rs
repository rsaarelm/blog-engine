use std::{
    fs::{self, File},
    io::{self, prelude::*},
    path::Path,
};

use askama::DynTemplate;

pub const EPOCH: &str = "1970-01-01T00:00:00Z";

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

/// Write a template to file.
pub fn write(
    value: &dyn DynTemplate,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let prefix = path.as_ref().parent().ok_or("err")?;
    fs::create_dir_all(prefix)?;
    let mut file = File::create(path.as_ref())?;
    Ok(value.dyn_write_into(&mut file)?)
}

/// Dump a directory tree into a single IDM expression.
pub fn read_path(path: impl AsRef<Path>) -> Result<String, std::fmt::Error> {
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

pub fn tag_set(tag: &str) -> Vec<&str> {
    let mut ret = vec![tag];
    for (i, c) in tag.char_indices() {
        if c == '/' {
            ret.push(&tag[0..i]);
        }
    }

    ret
}
