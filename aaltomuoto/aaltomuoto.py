#!/usr/bin/env python3
"""
Uses audiowaveform and ffmpeg to create frequency-colored waveforms
with 6 bands mapped to a blue-cyan color gradient:
  - Sub-bass (< 60Hz)     → Deep blue
  - Bass (60-250Hz)       → Blue
  - Low-mid (250-500Hz)   → Light blue
  - Mid (500-2kHz)        → Cyan
  - High-mid (2-6kHz)     → Bright cyan
  - High (> 6kHz)         → White/cyan
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from typing import Any

import mutagen
import numpy as np
from numpy.typing import NDArray
from PIL import Image, ImageDraw, ImageFont

# Type aliases
FloatArray = NDArray[np.floating[Any]]
ColorTuple = tuple[int, int, int]

# Frequency bands: (low_hz, high_hz, name)
BANDS: list[tuple[int, int, str]] = [
    (0, 60, "sub"),
    (60, 250, "bass"),
    (250, 500, "lowmid"),
    (500, 2000, "mid"),
    (2000, 6000, "highmid"),
    (6000, 20000, "high"),
]

# Colors for each band (RGB) - blue to cyan to white gradient
BAND_COLORS: dict[str, NDArray[np.int64]] = {
    "sub": np.array([20, 60, 140]),
    "bass": np.array([30, 100, 180]),
    "lowmid": np.array([50, 140, 210]),
    "mid": np.array([70, 180, 230]),
    "highmid": np.array([120, 210, 250]),
    "high": np.array([200, 240, 255]),
}

# Waveform detail level
PIXELS_PER_SECOND = 500

# Layout constants
TITLE_HEIGHT = 25
TIMELINE_HEIGHT = 20

# Color palette
BG_COLOR: ColorTuple = (5, 10, 20)
CENTER_LINE_COLOR: ColorTuple = (25, 50, 80)
QUIET_COLOR: ColorTuple = (40, 80, 120)
TITLE_COLOR: ColorTuple = (150, 200, 240)
TIMELINE_COLOR: ColorTuple = (30, 60, 100)
TICK_COLOR: ColorTuple = (60, 120, 180)
TEXT_COLOR: ColorTuple = (100, 160, 220)

# Amplitude threshold for "quiet" sections
QUIET_THRESHOLD = 8


def get_track_info(file_path: str) -> str:
    """Extract artist and title from audio file metadata."""
    try:
        audio = mutagen.File(file_path, easy=True)
        if audio is None:
            return os.path.basename(file_path)

        artist_list = audio.get("artist")
        title_list = audio.get("title")

        artist = str(artist_list[0]) if artist_list else ""
        title = str(title_list[0]) if title_list else ""

        if artist and title:
            return f"{artist} - {title}"
        return title or artist or os.path.basename(file_path)
    except Exception:
        return os.path.basename(file_path)


def run_ffmpeg_filter(input_file: str, output_file: str, filter_str: str) -> None:
    """Run ffmpeg with an audio filter."""
    subprocess.run(
        [
            "ffmpeg",
            "-y",
            "-i",
            input_file,
            "-af",
            filter_str,
            "-ar",
            "44100",
            output_file,
        ],
        check=True,
        capture_output=True,
    )


def generate_waveform_json(
    input_file: str,
    output_json: str,
    pixels_per_second: int,
) -> dict[str, Any]:
    """Generate waveform data using audiowaveform."""
    subprocess.run(
        [
            "audiowaveform",
            "-i",
            input_file,
            "-o",
            output_json,
            "--pixels-per-second",
            str(pixels_per_second),
            "--bits",
            "8",
        ],
        check=True,
        capture_output=True,
    )
    with open(output_json) as f:
        result: dict[str, Any] = json.load(f)
        return result


def extract_amplitudes(data: dict[str, Any]) -> FloatArray:
    """Extract absolute max amplitudes from waveform data."""
    raw_data: list[int] = data["data"]
    raw = np.array(raw_data, dtype=np.int8)
    mins = raw[0::2].astype(np.float64)
    maxs = raw[1::2].astype(np.float64)
    return np.maximum(np.abs(mins), np.abs(maxs))


def resize_array(arr: FloatArray, target_len: int) -> FloatArray:
    """Resize array to target length using nearest-neighbor interpolation."""
    if len(arr) == target_len:
        return arr
    indices = np.linspace(0, len(arr) - 1, target_len).astype(np.intp)
    return arr[indices]


def load_font(path: str, size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    """Load a TrueType font, falling back to default if unavailable."""
    try:
        return ImageFont.truetype(path, size)
    except OSError:
        return ImageFont.load_default()


def render_waveform(
    original_amps: FloatArray,
    band_amps: dict[str, FloatArray],
    duration_seconds: float,
    track_info: str,
    output_file: str,
    width: int = 4000,
    height: int = 350,
) -> None:
    """Render the CDJ-style waveform image with 6-band coloring."""
    n_samples = len(original_amps)
    waveform_height = height - TIMELINE_HEIGHT - TITLE_HEIGHT

    # Resize all band arrays to match original
    resized_bands: dict[str, FloatArray] = {
        name: resize_array(amps, n_samples) for name, amps in band_amps.items()
    }

    # Create image
    img = Image.new("RGB", (width, height), color=BG_COLOR)
    pixels = img.load()
    if pixels is None:
        msg = "Failed to load image pixels"
        raise RuntimeError(msg)
    draw = ImageDraw.Draw(img)

    # Draw title
    title_font = load_font("/usr/share/fonts/TTF/DejaVuSansMono-Bold.ttf", 14)
    draw.text((10, 5), track_info, fill=TITLE_COLOR, font=title_font)

    # Calculate center line position
    center_y = TITLE_HEIGHT + waveform_height // 2

    # Draw center line (visible in quiet sections)
    for x in range(width):
        pixels[x, center_y] = CENTER_LINE_COLOR

    # Map samples to pixel columns
    x_indices = np.linspace(0, n_samples - 1, width).astype(np.intp)

    # Draw waveform bars
    for x in range(width):
        idx: int = int(x_indices[x])
        amp: float = float(original_amps[idx])
        bar_height = int((amp / 127.0) * (waveform_height // 2))

        if amp < QUIET_THRESHOLD:
            # Quiet section: draw visible marker
            for dy in range(-1, 2):
                y = center_y + dy
                if 0 <= y < height:
                    pixels[x, y] = QUIET_COLOR
            continue

        # Calculate color from band weights
        band_vals: dict[str, float] = {
            name: float(resized_bands[name][idx]) for name in resized_bands
        }
        total = sum(band_vals.values()) + 1e-6
        color = np.zeros(3, dtype=np.float64)
        for name, val in band_vals.items():
            color += BAND_COLORS[name].astype(np.float64) * (val / total)
        r, g, b = np.clip(color.astype(np.int64), 0, 255)
        color_tuple: ColorTuple = (int(r), int(g), int(b))

        # Draw vertical bar
        for y in range(center_y - bar_height, center_y + bar_height + 1):
            if TITLE_HEIGHT <= y < TITLE_HEIGHT + waveform_height:
                pixels[x, y] = color_tuple

    # Draw timeline
    timeline_y = TITLE_HEIGHT + waveform_height
    for x in range(width):
        pixels[x, timeline_y] = TIMELINE_COLOR

    # Draw time markers
    timeline_font = load_font("/usr/share/fonts/TTF/DejaVuSansMono.ttf", 12)
    tick_interval = 30  # seconds

    for i in range(int(duration_seconds / tick_interval) + 2):
        time_sec = i * tick_interval
        if time_sec > duration_seconds:
            break
        x = int((time_sec / duration_seconds) * (width - 1))

        # Draw tick
        for y in range(timeline_y, min(timeline_y + 5, height)):
            if 0 <= x < width:
                pixels[x, y] = TICK_COLOR

        # Draw label
        label = f"{int(time_sec // 60)}:{int(time_sec % 60):02d}"
        draw.text((x + 3, timeline_y + 3), label, fill=TEXT_COLOR, font=timeline_font)

    img.save(output_file)
    print(output_file)


def build_ffmpeg_filter(low_hz: int, high_hz: int) -> str:
    """Build ffmpeg filter string for a frequency band."""
    if low_hz == 0:
        return f"lowpass=f={high_hz}"
    if high_hz >= 20000:
        return f"highpass=f={low_hz}"
    return f"highpass=f={low_hz},lowpass=f={high_hz}"


def main() -> None:
    """Main entry point."""
    if len(sys.argv) < 2:
        print(
            "aaltomuoto <input_audio> [output.png] [width] [height]"
        )
        print("aaltomuoto track.flac waveform.png 4000 350")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2] if len(sys.argv) > 2 else "waveform.png"
    width = int(sys.argv[3]) if len(sys.argv) > 3 else 4000
    height = int(sys.argv[4]) if len(sys.argv) > 4 else 350

    if not os.path.exists(input_file):
        print(f"Error: File '{input_file}' not found")
        sys.exit(1)

    print(input_file)
    print(f"{output_file} ({width}x{height})")
    print(f"{PIXELS_PER_SECOND} pixels/second, {len(BANDS)} frequency bands")

    with tempfile.TemporaryDirectory() as tmpdir:
        # Split audio into frequency bands
        band_wavs: dict[str, str] = {}
        for low_hz, high_hz, name in BANDS:
            wav_path = os.path.join(tmpdir, f"{name}.wav")
            run_ffmpeg_filter(
                input_file, wav_path, build_ffmpeg_filter(low_hz, high_hz)
            )
            band_wavs[name] = wav_path
            print(f"  {name}: {low_hz}-{high_hz}Hz")

        # Generate waveform data
        orig_json = os.path.join(tmpdir, "original.json")
        orig_data = generate_waveform_json(input_file, orig_json, PIXELS_PER_SECOND)
        orig_amps = extract_amplitudes(orig_data)

        band_amps: dict[str, FloatArray] = {}
        for name, wav_path in band_wavs.items():
            json_path = os.path.join(tmpdir, f"{name}.json")
            data = generate_waveform_json(wav_path, json_path, PIXELS_PER_SECOND)
            band_amps[name] = extract_amplitudes(data)

        # Calculate duration
        samples_per_pixel: int = int(orig_data.get("samples_per_pixel", 256))
        sample_rate: int = int(orig_data.get("sample_rate", 44100))
        raw_data: list[int] = orig_data["data"]
        n_pixels = len(raw_data) // 2
        duration_seconds = float(n_pixels * samples_per_pixel) / sample_rate

        # Get track metadata
        track_info = get_track_info(input_file)
        print(track_info)

        # Render waveform
        render_waveform(
            orig_amps,
            band_amps,
            duration_seconds,
            track_info,
            output_file,
            width,
            height,
        )


if __name__ == "__main__":
    main()
