use anyhow::Result;
use bliss_audio::chroma::bench::{chroma_stft, estimate_tuning};
use bliss_audio::decoder::Decoder as DecoderTrait;
use bliss_audio::decoder::ffmpeg::FFmpegDecoder as Decoder;
use bliss_audio::utils::bench::stft;
use bliss_audio_aubio_rs::{OnsetMode, Tempo};
use chrono::Local;
use clap::{Arg, Command};
use indicatif::{ProgressBar, ProgressStyle};
use ndarray::{Array1, Axis};
use ndarray_stats::Quantile1dExt;
use noisy_float::types::n64;
use rayon::prelude::*;
use std::borrow::Cow;

use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub const SAMPLE_RATE: u32 = 22050;

const BPM_WINDOW_SIZE: usize = 1024;
const BPM_HOP_SIZE: usize = 128;
const BPM_OFFSET: f32 = -1.15;
const BPM_MIN: f32 = 80.0;
const BPM_MAX: f32 = 160.0;

const KEY_WINDOW_SIZE: usize = 8192;
const KEY_HOP_SIZE: usize = 2205;
const TUNING_PRECISION: f64 = 0.01;
const CHROMA_BINS: usize = 12;

const WAVEFORM_WIDTH: usize = 2000;
const WAVEFORM_HEIGHT: usize = 350;

use lofty::config::WriteOptions;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::{ItemKey, Tag};
use walkdir::WalkDir;

struct TrackInfo {
    path: PathBuf,
    artist: Option<String>,
    album: Option<String>,
    track: Option<String>,
    label: Option<String>,
    bpm: f32,
    key: String,
}

struct TrackError {
    path: PathBuf,
    reason: String,
}

fn calculate_bpm(samples: &[f32]) -> f32 {
    let window_size = BPM_WINDOW_SIZE;
    let hop_size = BPM_HOP_SIZE;
    let mut tempo = Tempo::new(OnsetMode::SpecFlux, window_size, hop_size, SAMPLE_RATE).unwrap();
    let mut bpms = Vec::new();

    for chunk in samples.chunks(hop_size) {
        let mut padded = chunk.to_vec();
        if padded.len() < hop_size {
            padded.resize(hop_size, 0.0);
        }
        if tempo.do_result(&padded).unwrap() > 0.0 {
            bpms.push(tempo.get_bpm());
        }
    }

    if bpms.is_empty() {
        return 0.0;
    }

    let bpms_array = Array1::from(bpms);
    #[allow(clippy::cast_possible_truncation)]
    let median_bpm = *bpms_array
        .mapv(|x| n64(f64::from(x)))
        .quantile_mut(n64(0.5), &ndarray_stats::interpolate::Midpoint)
        .unwrap()
        .as_ref() as f32;

    normalize_bpm(median_bpm)
}

fn normalize_bpm(mut bpm: f32) -> f32 {
    if bpm > 0.0 {
        bpm += BPM_OFFSET;
    }

    // Constraints BPM to 80-160
    if bpm > 0.0 {
        while bpm > BPM_MAX {
            bpm /= 2.0;
        }
        while bpm < BPM_MIN {
            bpm *= 2.0;
        }
    }

    bpm
}

fn calculate_key(samples: &[f32]) -> Result<String> {
    // Pearson correlation
    fn correlation(v1: &Array1<f64>, v2: &Array1<f64>) -> f64 {
        let mean1 = v1.mean().unwrap();
        let mean2 = v2.mean().unwrap();
        let num: f64 = v1
            .iter()
            .zip(v2.iter())
            .map(|(x, y)| (x - mean1) * (y - mean2))
            .sum();
        let den1: f64 = v1.iter().map(|x| (x - mean1).powi(2)).sum();
        let den2: f64 = v2.iter().map(|y| (y - mean2).powi(2)).sum();
        if den1 == 0.0 || den2 == 0.0 {
            0.0
        } else {
            num / (den1.sqrt() * den2.sqrt())
        }
    }

    let window_size = KEY_WINDOW_SIZE;
    let hop_size = KEY_HOP_SIZE;

    let mut spectrum = stft(samples, window_size, hop_size);

    let tuning = estimate_tuning(
        SAMPLE_RATE,
        &spectrum,
        window_size,
        TUNING_PRECISION,
        CHROMA_BINS.try_into()?,
    )?;

    let n_chroma = u32::try_from(CHROMA_BINS)?;
    let chroma = chroma_stft(SAMPLE_RATE, &mut spectrum, window_size, n_chroma, tuning)?;

    // 4. Sum chroma vectors (collapse time)
    let global_chroma = chroma.sum_axis(Axis(1));

    // 5. Krumhansl-Schmuckler
    // Profiles from http://rnhart.net/articles/key-finding/ (Krumhansl-Schmuckler)
    // C Major
    let major_profile = Array1::from(vec![
        6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
    ]);

    // C Minor
    let minor_profile = Array1::from(vec![
        6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
    ]);

    let mut max_corr = -1.0;
    let mut best_key_idx = 0;
    let mut best_mode = "Major";

    let key_names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    // Iterate through all 12 keys
    for i in 0..CHROMA_BINS {
        // Rotate profiles to match the key
        let mut rotated_major = Array1::zeros(CHROMA_BINS);
        let mut rotated_minor = Array1::zeros(CHROMA_BINS);

        for j in 0..CHROMA_BINS {
            rotated_major[j] = major_profile[(j + CHROMA_BINS - i) % CHROMA_BINS];
            rotated_minor[j] = minor_profile[(j + CHROMA_BINS - i) % CHROMA_BINS];
        }

        let corr_major = correlation(&global_chroma, &rotated_major);
        if corr_major > max_corr {
            max_corr = corr_major;
            best_key_idx = i;
            best_mode = "Major";
        }

        let corr_minor = correlation(&global_chroma, &rotated_minor);
        if corr_minor > max_corr {
            max_corr = corr_minor;
            best_key_idx = i;
            best_mode = "Minor";
        }
    }

    Ok(format!("{} {}", key_names[best_key_idx], best_mode))
}

fn main() -> Result<()> {
    let matches = Command::new("virittaa")
        .arg(Arg::new("path").required(true).num_args(1..))
        .arg(
            Arg::new("write-tags")
                .long("write-tags")
                .action(clap::ArgAction::SetTrue)
                .help("Write BPM and Key to file metadata"),
        )
        .arg(
            Arg::new("jobs")
                .short('j')
                .long("jobs")
                .help("Number of threads to use")
                .default_value("0")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("batch-size")
                .short('b')
                .long("batch-size")
                .help("Number of files to process before cooldown (0 = off)")
                .default_value("5")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("cooldown")
                .short('c')
                .long("cooldown")
                .help("Cooldown in seconds between batches")
                .default_value("5")
                .value_parser(clap::value_parser!(u64)),
        )
        .arg(
            Arg::new("report")
                .short('r')
                .long("report")
                .action(clap::ArgAction::SetTrue)
                .help("Generate a CSV report file in the current directory"),
        )
        .arg(
            Arg::new("waveform")
                .short('w')
                .long("waveform")
                .action(clap::ArgAction::SetTrue)
                .help("Generate waveform images using aaltomuoto"),
        )
        .get_matches();

    let write_tags = matches.get_flag("write-tags");
    let generate_report = matches.get_flag("report");
    let generate_waveforms = matches.get_flag("waveform");
    let jobs = *matches.get_one::<usize>("jobs").unwrap();
    let batch_size = *matches.get_one::<usize>("batch-size").unwrap();
    let cooldown_secs = *matches.get_one::<u64>("cooldown").unwrap();

    if jobs > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .unwrap();
    }

    let path: Vec<String> = matches
        .get_many::<String>("path")
        .unwrap()
        .map(std::clone::Clone::clone)
        .collect();

    run_processing(
        path,
        batch_size,
        cooldown_secs,
        write_tags,
        generate_report,
        generate_waveforms,
    )
}

#[allow(clippy::fn_params_excessive_bools)]
fn run_processing(
    paths: Vec<String>,
    batch_size: usize,
    cooldown_secs: u64,
    write_tags: bool,
    generate_report: bool,
    generate_waveforms: bool,
) -> Result<()> {
    let mut files_to_process = Vec::new();

    for path_str in paths {
        let path = PathBuf::from(&path_str);
        if path.is_dir() {
            for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file()
                    && let Some(ext) = path.extension()
                {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if [
                        "aac", "ape", "aif", "aiff", "flac", "mp3", "mp4", "m4a", "mpc", "opus",
                        "ogg", "spx", "wav", "wv",
                    ]
                    .contains(&ext_str.as_str())
                    {
                        files_to_process.push(path.to_path_buf());
                    }
                }
            }
        } else {
            files_to_process.push(path);
        }
    }

    let total_files = files_to_process.len();
    let results: Mutex<Vec<TrackInfo>> = Mutex::new(Vec::new());
    let errors: Mutex<Vec<TrackError>> = Mutex::new(Vec::new());

    if batch_size == 0 {
        // Process all files in parallel
        files_to_process
            .par_iter()
            .for_each(|path| match process_file(path, write_tags) {
                Ok(info) => results.lock().unwrap().push(info),
                Err(reason) => errors.lock().unwrap().push(TrackError {
                    path: path.clone(),
                    reason,
                }),
            });
    } else {
        // Batch processing with cooldown
        let chunks: Vec<_> = files_to_process.chunks(batch_size).collect();
        let total_batches = chunks.len();

        let pb = ProgressBar::new(total_files as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        for (batch_idx, batch) in chunks.iter().enumerate() {
            // Process batch in parallel
            batch.par_iter().for_each(|path| {
                match process_file(path, write_tags) {
                    Ok(info) => results.lock().unwrap().push(info),
                    Err(reason) => errors.lock().unwrap().push(TrackError {
                        path: path.clone(),
                        reason,
                    }),
                }
                pb.inc(1);
            });

            // Cooldown between batches (skip after the last batch)
            if batch_idx < total_batches - 1 {
                pb.set_message(format!("Cooldown {cooldown_secs} seconds..."));
                thread::sleep(Duration::from_secs(cooldown_secs));
                pb.set_message("");
            }
        }

        pb.finish_with_message("Done!");
    }

    if generate_report || generate_waveforms {
        let results = results.into_inner().unwrap();
        let errors = errors.into_inner().unwrap();

        if generate_report {
            write_report(&results, &errors)?;
        }

        if generate_waveforms {
            generate_waveform_images(&results);
        }
    }

    Ok(())
}

fn process_file(path: &std::path::Path, write_tags: bool) -> Result<TrackInfo, String> {
    let song = Decoder::decode(path).map_err(|e| format!("Error decoding: {e}"))?;
    let samples = &song.sample_array;

    let bpm = calculate_bpm(samples);
    let key = calculate_key(samples).map_err(|e| format!("Error calculating key: {e}"))?;

    // Read existing metadata from file
    let (artist, album, track, label) = match Probe::open(path).and_then(Probe::read) {
        Ok(tagged_file) => {
            let tag = tagged_file
                .primary_tag()
                .or_else(|| tagged_file.first_tag());
            if let Some(t) = tag {
                (
                    t.get_string(&ItemKey::TrackArtist)
                        .or_else(|| t.get_string(&ItemKey::AlbumArtist))
                        .map(String::from),
                    t.get_string(&ItemKey::AlbumTitle).map(String::from),
                    t.get_string(&ItemKey::TrackTitle).map(String::from),
                    t.get_string(&ItemKey::Label).map(String::from),
                )
            } else {
                (None, None, None, None)
            }
        }
        Err(_) => (None, None, None, None),
    };

    println!("File: {}, BPM: {bpm:.1}, Key: {key}", path.display());

    if write_tags && let Err(e) = write_metadata(path, bpm, &key) {
        eprintln!("Error writing tags for {}: {e}", path.display());
    }

    Ok(TrackInfo {
        path: path.to_path_buf(),
        artist,
        album,
        track,
        label,
        bpm,
        key,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct ReportRecord<'a> {
    path: Cow<'a, str>,
    artist: Option<&'a str>,
    album: Option<&'a str>,
    track: Option<&'a str>,
    label: Option<&'a str>,
    #[serde(rename = "BPM")]
    bpm: Option<String>,
    key: Option<&'a str>,
    status: &'a str,
    error: Option<&'a str>,
}

fn write_report(results: &[TrackInfo], errors: &[TrackError]) -> Result<()> {
    let timestamp = Local::now().format("%Y-%m-%dT%H-%M-%S").to_string();
    let filename = format!("virittaa-report-{timestamp}.csv");

    let mut wtr = csv::Writer::from_path(&filename)?;

    for track in results {
        wtr.serialize(ReportRecord {
            path: track.path.to_string_lossy(),
            artist: track.artist.as_deref(),
            album: track.album.as_deref(),
            track: track.track.as_deref(),
            label: track.label.as_deref(),
            bpm: Some(format!("{:.1}", track.bpm)),
            key: Some(&track.key),
            status: "Success",
            error: None,
        })?;
    }

    for error in errors {
        wtr.serialize(ReportRecord {
            path: error.path.to_string_lossy(),
            artist: None,
            album: None,
            track: None,
            label: None,
            bpm: None,
            key: None,
            status: "Error",
            error: Some(&error.reason),
        })?;
    }

    wtr.flush()?;
    println!("Report written to: {filename}");
    Ok(())
}

fn generate_waveform_images(results: &[TrackInfo]) {
    for track in results {
        // artist_album_track_waveform.png
        let artist = track.artist.as_deref().unwrap_or("unknown");
        let album = track.album.as_deref().unwrap_or("unknown");
        let title = track.track.as_deref().unwrap_or("unknown");

        let output_name = format!(
            "{}_{}_{}_{}.png",
            sanitize_filename(artist),
            sanitize_filename(album),
            sanitize_filename(title),
            "waveform"
        )
        .to_lowercase();

        let track_path = track.path.to_string_lossy();

        match ProcessCommand::new("aaltomuoto")
            .arg(&*track_path)
            .arg(&output_name)
            .arg(WAVEFORM_WIDTH.to_string())
            .arg(WAVEFORM_HEIGHT.to_string())
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    println!("Waveform generated: {output_name}");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("Error generating waveform for {track_path}: {stderr}");
                }
            }
            Err(e) => {
                eprintln!("Failed to run aaltomuoto for {track_path}: {e}");
            }
        }
    }
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn write_metadata(path: &std::path::Path, bpm: f32, key: &str) -> Result<()> {
    let mut tagged_file = Probe::open(path)?.read()?;
    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            if let Some(first_tag) = tagged_file.first_tag_mut() {
                first_tag
            } else {
                // If there are no tags, create a new one based on the file type
                let tag_type = tagged_file.primary_tag_type();
                tagged_file.insert_tag(Tag::new(tag_type));
                tagged_file.primary_tag_mut().unwrap()
            }
        }
    };

    tag.insert_text(ItemKey::Bpm, bpm.round().to_string());
    tag.insert_text(ItemKey::InitialKey, key.to_string());
    tag.insert_text(ItemKey::Unknown("TKEY".to_string()), key.to_string());

    tag.save_to_path(path, WriteOptions::default())?;
    Ok(())
}
