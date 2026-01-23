# virittaa

Opinionated tool for analyzing audio files to detect `BPM` and `Key`, with optional metadata tagging and report generation.

## Usage

```bash
# Analyze files and print results
virittaa path/to/music

# Analyze and write metadata tags
virittaa --write-tags path/to/music

# Generate a CSV report
virittaa --report path/to/music

# Generate waveforms (requires aaltomuoto)
virittaa --waveform path/to/music
```
