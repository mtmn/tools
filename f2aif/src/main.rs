use anyhow::{Context, Result, bail};
use clap::Parser;
use id3::{Tag, TagLike, Timestamp, Version, frame};
use rayon::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "flac2aiff")]
#[command(about = "Convert FLAC files to AIFF format recursively")]
struct Args {
    folder_path: PathBuf,
    #[arg(long)]
    keep_original: bool,
    #[arg(short = 'j', long, default_value_t = num_cpus::get())]
    jobs: usize,
}

struct Stats {
    converted: AtomicUsize,
    deleted: AtomicUsize,
    errors: AtomicUsize,
    skipped: AtomicUsize,
}

impl Stats {
    fn new() -> Self {
        Self {
            converted: AtomicUsize::new(0),
            deleted: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            skipped: AtomicUsize::new(0),
        }
    }
}

struct Converter {
    input: PathBuf,
    output: PathBuf,
}

impl Converter {
    fn new(input: &Path) -> Self {
        Self {
            input: input.to_path_buf(),
            output: input.with_extension("aif"),
        }
    }

    fn convert(&self) -> Result<bool> {
        if self.output.exists() {
            return Ok(false);
        }

        let file = File::open(&self.input).context("Failed to open input file")?;

        let mss = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());

        let mut hint = Hint::new();
        hint.with_extension("flac");

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .context("Failed to probe file")?;

        let mut format = probed.format;
        let track = format.default_track().context("No default track found")?;

        let track_id = track.id;
        let codec_params = track.codec_params.clone();

        let sample_rate = codec_params.sample_rate.context("Sample rate not found")?;
        let channels = u16::try_from(codec_params.channels.context("Channels not found")?.count())
            .context("Channel count too large")?;

        let mut decoder = symphonia::default::get_codecs()
            .make(&codec_params, &DecoderOptions::default())
            .context("Failed to create decoder")?;

        let mut samples: Vec<i16> = Vec::new();

        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::IoError(ref e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(e) => return Err(e).context("Failed to read packet")?,
            };

            if packet.track_id() != track_id {
                continue;
            }

            let audio_buf = decoder.decode(&packet).context("Failed to decode packet")?;

            match audio_buf {
                AudioBufferRef::S16(buf) => {
                    for &sample in buf.chan(0) {
                        samples.push(sample);
                    }
                    if channels == 2 {
                        for &sample in buf.chan(1) {
                            samples.push(sample);
                        }
                    }
                }
                AudioBufferRef::S32(buf) => {
                    for &sample in buf.chan(0) {
                        samples.push((sample >> 16) as i16);
                    }
                    if channels == 2 {
                        for &sample in buf.chan(1) {
                            samples.push((sample >> 16) as i16);
                        }
                    }
                }
                AudioBufferRef::F32(buf) => {
                    #[allow(clippy::cast_possible_truncation)]
                    for &sample in buf.chan(0) {
                        samples.push((sample * 32767.0) as i16);
                    }
                    if channels == 2 {
                        #[allow(clippy::cast_possible_truncation)]
                        for &sample in buf.chan(1) {
                            samples.push((sample * 32767.0) as i16);
                        }
                    }
                }
                _ => bail!("Unsupported sample format"),
            }
        }

        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer =
            hound::WavWriter::create(&self.output, spec).context("Failed to create AIFF writer")?;

        for sample in samples {
            writer
                .write_sample(sample)
                .context("Failed to write sample")?;
        }

        writer.finalize().context("Failed to finalize AIFF")?;

        self.copy_metadata(&mut format)?;

        Ok(true)
    }

    fn copy_metadata(
        &self,
        format: &mut Box<dyn symphonia::core::formats::FormatReader>,
    ) -> Result<()> {
        let mut tag = Tag::new();

        if let Some(metadata_rev) = format.metadata().current() {
            for tag_item in metadata_rev.tags() {
                let key = tag_item.key.as_str();
                let value = tag_item.value.to_string();

                match key.to_uppercase().as_str() {
                    "TITLE" => tag.set_title(&value),
                    "ARTIST" => tag.set_artist(&value),
                    "ALBUM" => tag.set_album(&value),
                    "ALBUMARTIST" | "ALBUM_ARTIST" => tag.set_album_artist(&value),
                    "DATE" => {
                        if let Ok(timestamp) = value.parse::<Timestamp>() {
                            tag.set_date_released(timestamp);
                        }
                    }
                    "TRACK" | "TRACKNUMBER" => {
                        if let Ok(num) = value.parse::<u32>() {
                            tag.set_track(num);
                        }
                    }
                    "GENRE" => tag.set_genre(&value),
                    _ => {}
                }
            }

            for visual in metadata_rev.visuals() {
                tag.add_frame(frame::Picture {
                    mime_type: visual.media_type.clone(),
                    picture_type: frame::PictureType::CoverFront,
                    description: String::new(),
                    data: visual.data.to_vec(),
                });
            }
        }

        tag.write_to_path(&self.output, Version::Id3v23)
            .context("Failed to write tags")?;

        Ok(())
    }

    fn delete_original(&self) -> Result<()> {
        std::fs::remove_file(&self.input)
            .with_context(|| format!("Failed to delete: {}", self.input.display()))
    }
}

fn process_file(path: &Path, delete: bool) -> Result<bool> {
    let converter = Converter::new(path);

    if !converter.convert()? {
        return Ok(false);
    }

    if delete {
        converter.delete_original().ok();
    }

    Ok(true)
}

fn collect_flac_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|s| s.eq_ignore_ascii_case("flac"))
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn main() -> Result<()> {
    let args = Args::parse();

    let dir = args
        .folder_path
        .canonicalize()
        .context("Could not access directory")?;

    if !dir.is_dir() {
        bail!("Not a directory: {}", dir.display());
    }

    let files = collect_flac_files(&dir);

    if files.is_empty() {
        println!("No FLAC files found");
        return Ok(());
    }

    println!(
        "Processing {} files with {} threads",
        files.len(),
        args.jobs
    );

    let stats = Arc::new(Stats::new());
    let delete = !args.keep_original;

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.jobs)
        .build_global()
        .unwrap();

    files
        .par_iter()
        .for_each(|file| match process_file(file, delete) {
            Ok(true) => {
                stats.converted.fetch_add(1, Ordering::Relaxed);
                if delete && !file.exists() {
                    stats.deleted.fetch_add(1, Ordering::Relaxed);
                }
            }
            Ok(false) => {
                stats.skipped.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                eprintln!("Error: {}: {:#}", file.display(), e);
                stats.errors.fetch_add(1, Ordering::Relaxed);
            }
        });

    let converted = stats.converted.load(Ordering::Relaxed);
    let skipped = stats.skipped.load(Ordering::Relaxed);
    let errors = stats.errors.load(Ordering::Relaxed);

    println!("\nConverted: {converted}");
    if skipped > 0 {
        println!("Skipped: {skipped}");
    }
    if delete {
        println!("Deleted: {}", stats.deleted.load(Ordering::Relaxed));
    }
    if errors > 0 {
        println!("Errors: {errors}");
    }

    Ok(())
}
