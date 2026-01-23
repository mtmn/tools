use crate::track::Track;
use anyhow::Result;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::Tag;
use std::path::PathBuf;

pub fn get_track(path: PathBuf) -> Result<Track> {
    let tagged_file = Probe::open(&path)?.read()?;
    let tag = tagged_file
        .primary_tag()
        .cloned()
        .or_else(|| tagged_file.first_tag().cloned())
        .unwrap_or_else(|| Tag::new(tagged_file.file_type().primary_tag_type()));
    Ok(Track { path, tag })
}
