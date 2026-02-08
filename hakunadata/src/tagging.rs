use crate::context::AppContext;
use crate::metadata::fetch::{process_query, FetchedMetadata};
use anyhow::{Context, Result};
use lofty::config::WriteOptions;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::{ItemKey, Tag};
use std::fs::File;
use std::path::{Component, Path};

/// Process a music file to read or write metadata tags
pub async fn process_file(ctx: &AppContext, path: &Path, read: bool, write: bool) -> Result<()> {
    if !read && !write {
        return Ok(());
    }

    // Normalize the path to remove relative components like './' and '../'
    let normalized_path = normalize_path(path);
    let abs_path = std::env::current_dir().map_or_else(
        |_| normalized_path.clone(),
        |cwd| cwd.join(&normalized_path),
    );

    let path_display = abs_path.display();

    // Check if file exists
    if !abs_path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {path_display}"));
    }

    // Check if file is empty
    let metadata = abs_path
        .metadata()
        .with_context(|| format!("Failed to read metadata for: {path_display}"))?;

    if metadata.len() == 0 {
        eprintln!("Warning: File is empty, skipping: {path_display}");
        return Ok(());
    }

    // Read the file and extract current metadata
    let (mut tagged_file, artist, album) = {
        // Use the Probe API which can read from readers
        let file = File::open(&abs_path)
            .with_context(|| format!("Failed to open file: {path_display}"))?;
        let mut probe = Probe::new(file);

        // Hint the file type based on extension if possible
        if let Some(file_type) = path
            .extension()
            .and_then(|e| e.to_str())
            .and_then(lofty::file::FileType::from_ext)
        {
            probe = probe.set_file_type(file_type);
        }

        let mut tagged_file = probe
            .read()
            .with_context(|| format!("Failed to read tags from {path_display}"))?;

        let tag = tagged_file
            .primary_tag_mut()
            .context("No primary tag found")?;

        let artist = tag.artist().context("Artist not found")?.to_string();
        let album = tag.album().context("Album not found")?.to_string();

        (tagged_file, artist, album)
    }; // file handle goes out of scope here

    println!("Processing: {artist} - {album}");

    let result = process_query(ctx, &artist, &album).await?;

    if read {
        print_proposed_tags(&result);
    }

    if write {
        // Get mutable reference to tag for writing
        let tag = tagged_file
            .primary_tag_mut()
            .context("No primary tag found")?;
        write_tags(tag, &result);

        // Create a backup of the original file before modifying it to prevent data loss on failure
        let backup_path = abs_path.with_extension(format!(
            "backup.{}",
            abs_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("bak")
        ));

        // Copy the original file to backup
        std::fs::copy(&abs_path, &backup_path)
            .with_context(|| format!("Failed to create backup file: {}", backup_path.display()))?;

        // Attempt to save the modified file to the original location
        let save_result = tagged_file.save_to_path(&abs_path, WriteOptions::default());

        match save_result {
            Ok(()) => {
                // Success: remove the backup file
                let _ = std::fs::remove_file(&backup_path); // Ignore errors when removing backup
            }
            Err(e) => {
                // Failure: restore from backup before returning the error
                if std::path::Path::exists(&backup_path) {
                    if let Err(restore_err) = std::fs::copy(&backup_path, &abs_path) {
                        eprintln!(
                            "ERROR: Failed to restore from backup after write failure: {restore_err}",
                        );
                        eprintln!(
                            "WARNING: Original file may be corrupted. Backup preserved at: {}",
                            backup_path.display()
                        );
                        return Err(anyhow::anyhow!(
                            "Also failed to restore from backup: {restore_err}"
                        )
                        .context(e));
                    }
                    eprintln!("Restored file from backup after write failure");
                    let _ = std::fs::remove_file(&backup_path); // Clean up backup after successful restore
                }
                return Err(anyhow::anyhow!("Failed to write tags to file").context(e));
            }
        }

        println!("Tags written to {path_display}");
    }

    Ok(())
}

/// Helper function to normalize a path by removing relative components
fn normalize_path(path: &Path) -> std::path::PathBuf {
    let mut result = std::path::PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(prefix_component) => result.push(prefix_component.as_os_str()),
            Component::RootDir => result.push("/"),
            Component::CurDir => {} // Skip './'
            Component::ParentDir => {
                result.pop(); // Handle '../' by popping the last component
            }
            Component::Normal(os_string) => result.push(os_string),
        }
    }

    result
}

/// Print the proposed tags to stdout
fn print_proposed_tags(metadata: &FetchedMetadata) {
    println!("Proposed tags:");
    let genre_str = metadata.genres.join("/");
    println!("  Genre: {genre_str}");

    if let Some(label) = metadata.labels.first() {
        println!("  Label: {label}");
    }
}

/// Write metadata to the tag
fn write_tags(tag: &mut Tag, metadata: &FetchedMetadata) {
    let genre_str = metadata.genres.join("/");
    tag.insert_text(ItemKey::Genre, genre_str);

    if let Some(label) = metadata.labels.first() {
        tag.insert_text(ItemKey::Label, label.clone());
    }
}

/// Print metadata to stdout
pub fn print_metadata(result: &FetchedMetadata) {
    if result.genres.is_empty() {
        println!("Genres: (none)");
    } else {
        println!("Genres:");
        for genre in &result.genres {
            println!("  {genre}");
        }
    }

    if result.subgenres.is_empty() {
        println!("Subgenres: (none)");
    } else {
        println!("Subgenres:");
        for subgenre in &result.subgenres {
            println!("  {subgenre}");
        }
    }

    if result.labels.is_empty() {
        println!("Label: (none)");
    } else {
        println!("Label:");
        for label in &result.labels {
            println!("  {label}");
        }
    }
}
