use std::path::Path;
use std::{fs, path::PathBuf};

use color_eyre::eyre::{OptionExt, Result};
use gray_matter::{engine::YAML, Matter};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct Note {
    name: String,
    meta: NoteMetadata,
    body: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NoteMetadata {
    source: Option<String>,
    scope: String,
    r#type: ZettelType,
    created: String,  // for now
    modified: String, // for now
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ZettelType {
    Main,
    Source,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let workdir = "/nix/persist/active-externalism/data";

    // flat because my vault is flat, at least for now
    let notes: Vec<Note> = fs::read_dir(workdir)?
        .map(|entry| entry.unwrap().path())
        .filter(|path| !path.is_dir())
        .map(read_note)
        .filter(|result| result.is_ok()) // not all files are notes
        .flatten()
        .collect();

    dbg!(notes
        .iter()
        .find(|note| note.name == "website-parser-test.md"));

    Ok(())
}

fn read_note<P: AsRef<Path>>(path: P) -> Result<Note> {
    let matter = Matter::<YAML>::new();

    let file = fs::read_to_string(&path)?;
    let file = matter.parse(&file);

    let body = file.content;
    let regex = Regex::new(r"\[\[(.+?)(\|.+?)?\]\]").unwrap();

    let body = regex
        .replace_all(&body, |caps: &Captures| {
            let link = caps.get(1).unwrap().as_str();
            let label = match caps.get(2) {
                Some(label) => &label.as_str()[1..],
                None => link,
            };

            format!("[{}]({})", label, link)
        })
        .to_string();

    Ok(Note {
        name: path
            .as_ref()
            .iter()
            .last()
            .ok_or_eyre("Encountered a file without a name?")?
            .to_str()
            .expect("The file should still have a name after type conversion")
            .to_string(),
        meta: file
            .data
            .ok_or_eyre("The file has no frontmatter")?
            .deserialize()?,
        body,
    })
}
