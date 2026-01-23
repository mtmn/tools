use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use similar::{ChangeTag, TextDiff};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[arg(long, required = true)]
    host: String,

    #[arg(long, required = true)]
    local: PathBuf,

    #[arg(long)]
    remote: Option<PathBuf>,

    #[arg(long, conflicts_with = "remote")]
    same_as_local: bool,

    #[arg(long)]
    #[arg(long)]
    sync: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FileStatus {
    Created,
    Updated,
    Unchanged,
}

#[derive(Debug)]
struct FileSyncWorker {
    host_alias: String,
    local_path: PathBuf,
    remote_path: PathBuf,
    sync: bool,
}

impl FileSyncWorker {
    fn new(host_alias: String, local_path: PathBuf, remote_path: PathBuf, sync: bool) -> Self {
        Self {
            host_alias,
            local_path,
            remote_path,
            sync,
        }
    }

    fn sync(&self) -> Result<()> {
        fs::create_dir_all(&self.local_path).context("Failed to create local filess directory")?;

        let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
        let temp_path = temp_dir.path();

        println!("Syncing files from {}", self.host_alias);

        let remote_src = format!("{}:{}/", self.host_alias, self.remote_path.display());

        let status = Command::new("rsync")
            .arg("-az")
            .arg(&remote_src)
            .arg(temp_path)
            .status()
            .context("Failed to execute rsync")?;

        if !status.success() {
            anyhow::bail!("Rsync failed with status: {status}");
        }

        let mut created = 0;
        let mut updated = 0;
        let mut unchanged = 0;
        let mut errors = 0;

        let entries = fs::read_dir(temp_path).context("Failed to read temp directory")?;

        // Process files sequentially
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                match self.process_files(&path) {
                    Ok(FileStatus::Created) => created += 1,
                    Ok(FileStatus::Updated) => updated += 1,
                    Ok(FileStatus::Unchanged) => unchanged += 1,
                    Err(_) => errors += 1,
                }
            }
        }

        println!("\nSync completed:");
        println!("  Created: {created}");
        println!("  Updated: {updated}");
        println!("  Unchanged: {unchanged}");
        if errors > 0 {
            println!("  Errors: {errors}");
        }

        Ok(())
    }

    fn process_files(&self, temp_file_path: &Path) -> Result<FileStatus> {
        let filename = temp_file_path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid filename")?;

        let remote_content = fs::read_to_string(temp_file_path)
            .with_context(|| format!("Error reading temp file {filename}"))?;

        let remote_entries: Vec<String> = remote_content.lines().map(ToString::to_string).collect();

        self.merge_and_write(filename, remote_entries)
    }

    fn merge_and_write(&self, filename: &str, remote_entries: Vec<String>) -> Result<FileStatus> {
        let local_files = self.local_path.join(filename);
        let exists = local_files.exists();

        let final_entries = if exists {
            let local_content = fs::read_to_string(&local_files)?;
            let local_entries = local_content.lines().map(ToString::to_string).collect();
            Self::merge_entries(local_entries, remote_entries)
        } else {
            // For new files, copy remote content as-is without filtering or sorting
            remote_entries
        };

        let new_content = if final_entries.is_empty() {
            String::new()
        } else {
            format!("{}\n", final_entries.join("\n"))
        };

        if exists {
            let current_content = fs::read_to_string(&local_files)?;
            if current_content == new_content {
                return Ok(FileStatus::Unchanged);
            }
        }

        if !self.sync {
            let current_content = if exists {
                fs::read_to_string(&local_files)?
            } else {
                String::new()
            };

            println!("Diff for {filename}:");
            let diff = TextDiff::from_lines(&current_content, &new_content);
            for change in diff.iter_all_changes() {
                let (sign, style) = match change.tag() {
                    ChangeTag::Delete => ("-", style(change).red()),
                    ChangeTag::Insert => ("+", style(change).green()),
                    ChangeTag::Equal => (" ", style(change)),
                };
                print!("{sign}{style}");
            }

            return Ok(if exists {
                FileStatus::Updated
            } else {
                FileStatus::Created
            });
        }

        let status = if exists {
            println!("Updating: {filename}");
            FileStatus::Updated
        } else {
            println!("Creating: {filename}");
            FileStatus::Created
        };

        if !new_content.is_empty() || exists {
            fs::write(&local_files, new_content)?;
        }

        Ok(status)
    }

    fn merge_entries(local: Vec<String>, remote: Vec<String>) -> Vec<String> {
        // Union of local and remote, preserving unique entries
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        // Add all local entries first
        for entry in local {
            if !entry.trim().is_empty() && seen.insert(entry.clone()) {
                result.push(entry);
            }
        }

        // Add remote entries that aren't already seen
        for entry in remote {
            if !entry.trim().is_empty() && seen.insert(entry.clone()) {
                result.push(entry);
            }
        }

        result.sort_by(|a, b| natord::compare(a, b));
        result
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let remote = if cli.same_as_local {
        cli.local.clone()
    } else {
        cli.remote
            .context("--remote or --same-as-local must be specified")?
    };

    let syncer = FileSyncWorker::new(cli.host, cli.local, remote, cli.sync);
    syncer.sync()?;

    Ok(())
}
