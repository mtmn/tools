use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    #[arg(
        long,
        short = 'n',
        help = "Don't actually rename or tag files, only display what would happen"
    )]
    pub dry_run: bool,

    #[arg(
        long,
        short,
        help = "Use a different output directory (by default, it's the same as the input dir)"
    )]
    pub output_dir: Option<PathBuf>,

    /// The format to apply to files, excluding the extension.
    ///
    /// Substitutions can be applied inside curly brackets, for example with {artist} to get the
    /// track artist. Any formats returning data with "/" will have it transformed to "_".
    ///
    /// Available formats:
    ///
    /// TAG:
    ///
    ///   label
    ///   artist
    ///   album
    ///   track  (width: 2)
    ///   title
    ///
    /// LITERAL:
    ///
    ///   {{ and }} indicate literal brackets.
    #[arg(
        long,
        verbatim_doc_comment,
        default_value = "{label}/{artist}/{album}/{track} {title}"
    )]
    pub fmt: String,

    #[arg(help = "Directories to find music files in.")]
    pub paths: Option<Vec<PathBuf>>,

    #[arg(long, short = 'r', help = "Read tags)")]
    pub read_tags: bool,

    #[arg(long, short = 'w', help = "Write tags interactively")]
    pub write_tags: bool,

    #[arg(long, short = 'c', help = "Add cover art)")]
    pub add_cover: bool,

    #[arg(long, short = 'l', help = "Filter for files without a label")]
    pub no_label: bool,
}
