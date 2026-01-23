mod config;

mod prompt;
mod track;
mod util;

use anyhow::Result;
use clap::Parser;
use funcfmt::{FormatPieces, ToFormatPieces, fm};
use jwalk::WalkDir;
use lofty::prelude::*;
use lofty::tag::{Accessor, ItemKey};
use rayon::prelude::*;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use config::Config;
use track::{Track, get_track};

const ALLOWED_EXTS: &[&str] = &["mp3", "flac", "m4a"];

fn fix_track(track: &mut Track, dry_run: bool) {
    let fix_results = track::fixers::run_fixers(track, dry_run);
    match fix_results {
        Ok(applied_fixers) => {
            if applied_fixers {
                print_updated_tags(track);
            }
        }
        Err(err) => eprintln!("cannot fix {}: {:?}", track.path.display(), err),
    }
}

fn print_updated_tags(track: &Track) {
    println!(
        "{}: updated tags: artist: '{}', album: '{}', title: '{}', label: '{}'",
        track.path.display(),
        track.tag.artist().unwrap_or(std::borrow::Cow::Borrowed("")),
        track.tag.album().unwrap_or(std::borrow::Cow::Borrowed("")),
        track.tag.title().unwrap_or(std::borrow::Cow::Borrowed("")),
        util::get_label(&track.tag).unwrap_or(std::borrow::Cow::Borrowed(""))
    );
}

fn print_tags(track: &Track) {
    let label = util::get_label(&track.tag).unwrap_or(std::borrow::Cow::Borrowed("Unknown Label"));

    println!(
        "Folder: {}\nArtist: {}\nAlbum: {}\nLabel: {}\n",
        track
            .path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .display(),
        track
            .tag
            .artist()
            .unwrap_or(std::borrow::Cow::Borrowed("Unknown Artist")),
        track
            .tag
            .album()
            .unwrap_or(std::borrow::Cow::Borrowed("Unknown Album")),
        label
    );
}

fn rename_track(track: &Track, fp: &FormatPieces<Track>, output_path: &Path, dry_run: bool) {
    let new_path = track::rename::rename_track(track, fp, output_path, dry_run);

    match new_path {
        Ok(Some(new_path)) => println!(
            "{}: renamed to {}",
            track.path.display(),
            new_path.display()
        ),
        Ok(None) => (),
        Err(err) => eprintln!("cannot rename {}: {:?}", track.path.display(), err),
    }
}

const ADDITIONAL_ACCEPTED_CHARS: &[char] = &['.', '-', '(', ')', ','];

fn clean_part(path_part: &str) -> String {
    path_part
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() || ADDITIONAL_ACCEPTED_CHARS.contains(&c) {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn get_format_pieces(tmpl: &str) -> Result<funcfmt::FormatPieces<Track>> {
    let formatters = fm!(
        "artist" => |t: &Track| Some(clean_part(t.tag.artist().as_deref().unwrap_or("Unknown Artist"))),
        "album" => |t: &Track| Some(clean_part(t.tag.album().as_deref().unwrap_or("Unknown Album"))),
        "title" => |t: &Track| Some(clean_part(t.tag.title().as_deref().unwrap_or("Unknown Title"))),
        "track" => |t: &Track| Some(format!("{:02}", t.tag.track().unwrap_or_default())),
        "label" => |t: &Track| Some(clean_part(util::get_label(&t.tag).as_deref().unwrap_or("Unknown Label"))),
    );

    Ok(formatters.to_format_pieces(tmpl)?)
}

fn run_write_tags(paths: &[PathBuf], no_label: bool) {
    let mut by_folder: std::collections::HashMap<PathBuf, Vec<PathBuf>> =
        std::collections::HashMap::new();
    for path in paths {
        let parent = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        by_folder.entry(parent).or_default().push(path.clone());
    }

    // Sort folders for consistent UX
    let mut parents: Vec<_> = by_folder.keys().cloned().collect();
    parents.sort();

    for parent in parents {
        let mut files = by_folder[&parent].clone();
        if files.is_empty() {
            continue;
        }

        if no_label {
            files.retain(|path| {
                if let Ok(track) = get_track(path.clone()) {
                    util::get_label(&track.tag).is_none()
                } else {
                    false
                }
            });
        }

        if files.is_empty() {
            continue;
        }

        // Use first file to guess current tags
        let first_path = &files[0];
        let first_track = match get_track(first_path.clone()) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error reading {}: {e:?}", first_path.display());
                continue;
            }
        };

        // Prompt user once for the folder
        println!("\nEditing Folder: {}", parent.display());
        let (new_artist, new_album, new_label) = prompt::prompt_tags(&first_track);

        // Apply to all files in folder if values are present via prompt_tags
        for path in files {
            match get_track(path.clone()) {
                Ok(mut track) => {
                    let mut changed = false;
                    if let Some(ref a) = new_artist {
                        track.tag.set_artist(a.clone());
                        changed = true;
                    }
                    if let Some(ref a) = new_album {
                        track.tag.set_album(a.clone());
                        changed = true;
                    }
                    if let Some(ref l) = new_label {
                        track.tag.insert_text(ItemKey::Label, l.clone());
                        changed = true;
                    }

                    if changed
                        && let Err(e) = track
                            .tag
                            .save_to_path(&track.path, lofty::config::WriteOptions::default())
                    {
                        eprintln!("Error saving {}: {e:?}", track.path.display());
                    }
                }
                Err(e) => eprintln!("Error reading {}: {e:?}", path.display()),
            }
        }
    }
}

fn run_read_tags(paths: &[PathBuf], no_label: bool) {
    let mut seen_dirs = std::collections::HashSet::new();
    for path in paths {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        if seen_dirs.contains(parent) {
            continue;
        }
        if let Ok(track) = get_track(path.clone()) {
            if no_label && util::get_label(&track.tag).is_some() {
                continue;
            }
            print_tags(&track);
            seen_dirs.insert(parent.to_path_buf());
        }
    }
}

fn run_add_cover(paths: &[PathBuf]) {
    paths.par_iter().for_each(|path| {
        if let Ok(mut track) = get_track(path.clone())
            && let Err(e) = add_cover_art(&mut track)
        {
            eprintln!("Error adding cover for {}: {e:?}", path.display());
        }
    });
}

fn run_fix_rename(cfg: &Config, output_path: &Path, paths: &[PathBuf]) {
    let default_fmt = "{label}/{artist}/{album}/{track} {title}";
    let fallback_fmt = "{artist}/{album}/{track} {title}";

    let fp_default = match get_format_pieces(&cfg.fmt) {
        Ok(fp) => fp,
        Err(err) => {
            eprintln!("fatal: {err}");
            std::process::exit(1);
        }
    };

    // Only compile fallback if we are using the default format
    let fp_fallback = if cfg.fmt == default_fmt {
        Some(match get_format_pieces(fallback_fmt) {
            Ok(fp) => fp,
            Err(err) => {
                eprintln!("fatal: {err}");
                std::process::exit(1);
            }
        })
    } else {
        None
    };

    paths
        .par_iter()
        .for_each(|path| match get_track(path.clone()) {
            Ok(mut track) => {
                fix_track(&mut track, cfg.dry_run);

                // Determine format to use
                let format_to_use = if let Some(ref fp_fb) = fp_fallback {
                    if util::get_label(&track.tag).is_some() {
                        &fp_default
                    } else {
                        fp_fb
                    }
                } else {
                    &fp_default
                };

                rename_track(&track, format_to_use, output_path, cfg.dry_run);
            }
            Err(err) => eprintln!("error: {}: {err:?}", path.display()),
        });
}

fn fix_all_tracks(cfg: &Config, base_path: &PathBuf, output_path: &Path) {
    let walker = WalkDir::new(base_path).skip_hidden(false);
    let paths: Vec<PathBuf> = walker
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path())
        .filter(|e| {
            let ext = e
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or("")
                .to_lowercase();
            ALLOWED_EXTS.iter().any(|a| a == &ext)
        })
        .collect();

    // Use sequential interaction for write_tags, otherwise parallel
    if cfg.write_tags {
        run_write_tags(&paths, cfg.no_label);
        return;
    }

    if cfg.read_tags {
        run_read_tags(&paths, cfg.no_label);
        return;
    }

    if cfg.add_cover {
        run_add_cover(&paths);
        return;
    }

    run_fix_rename(cfg, output_path, &paths);
}

fn add_cover_art(track: &mut Track) -> Result<()> {
    // Check for cover.jpg, cover.png, folder.jpg, folder.png in the same directory
    let parent = track.path.parent().unwrap_or_else(|| Path::new("."));
    let candidates = ["cover.jpg", "cover.png", "folder.jpg", "folder.png"];

    let mut cover_path = None;
    for c in candidates {
        let p = parent.join(c);
        if p.exists() {
            cover_path = Some(p);
            break;
        }
    }

    if let Some(path) = cover_path {
        println!(
            "Adding cover art from {} to {}",
            path.display(),
            track.path.display()
        );
        let mut picture = lofty::picture::Picture::from_reader(&mut std::fs::File::open(&path)?)?;
        track
            .tag
            .remove_picture_type(lofty::picture::PictureType::CoverFront);
        picture.set_pic_type(lofty::picture::PictureType::CoverFront);
        track.tag.push_picture(picture);
        track
            .tag
            .save_to_path(&track.path, lofty::config::WriteOptions::default())?;
    }

    Ok(())
}

fn main() {
    let mut cfg = Config::parse();

    let paths = cfg.paths.take().unwrap_or_else(|| vec![PathBuf::from(".")]);

    for path in paths {
        let output_path = cfg.output_dir.clone().unwrap_or_else(|| path.clone());
        fix_all_tracks(&cfg, &path, &output_path);
    }
}
