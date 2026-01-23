# lazymaster
2-pass audio loudness normalization using `ffmpeg` and [loudnorm](https://ffmpeg.org/ffmpeg-filters.html#loudnorm) filter.

It normalizes audio to **-13 LUFS**, **-1 dBTP**, and **LRA 8**.

## Usage

**Analysis:**
```bash
./lazymaster.py input.wav
```

**Two-pass normalization:**
```bash
./lazymaster input.wav output.wav
```
