#!/usr/bin/env python3
import os
import subprocess
import argparse
from pathlib import Path

def check_artwork(file_path):
    """Check if a file contains artwork."""
    result = subprocess.run([
        'ffprobe',
        '-v', 'error',
        '-select_streams', 'v',
        '-show_entries', 'stream=codec_type',
        '-of', 'default=noprint_wrappers=1:nokey=1',
        str(file_path)
    ], capture_output=True, text=True)
    return 'video' in result.stdout

def convert_flac_to_aiff(input_path, delete_original=True):
    """Convert a FLAC file to AIFF format."""
    output_path = input_path.with_suffix('.aif')
    
    if output_path.exists():
        print(f"Skipping {input_path} - {output_path} already exists")
        return True
    
    print(f"Converting {input_path} to {output_path}")
    
    try:
        subprocess.run([
            'ffmpeg',
            '-i', str(input_path),
            '-c:a', 'pcm_s16be',   # Use 16-bit PCM encoding
            '-map', '0:a',         # Map audio stream
            '-map', '0:v?',        # Map video (cover art) stream if it exists
            '-map_metadata', '0',  # Copy all metadata
            '-write_id3v2', '1',   # Write ID3v2 tags
            '-id3v2_version', '3', # Use ID3v2.3 for better compatibility
            '-f', 'aiff',
            str(output_path)
        ], check=True, capture_output=True)
        
        had_artwork = check_artwork(input_path)
        if had_artwork:
            if check_artwork(output_path):
                print(f"Successfully converted {input_path} (artwork preserved)")
            else:
                print(f"Warning: Artwork may not have transferred for {input_path}")
        else:
            print(f"Successfully converted {input_path}")
        
        if delete_original:
            try:
                input_path.unlink()
                print(f"Deleted original file: {input_path}")
            except Exception as e:
                print(f"Warning: Could not delete original file {input_path}: {str(e)}")
        
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"Error converting {input_path}:")
        print(f"ffmpeg error: {e.stderr.decode()}")
        return False
    except Exception as e:
        print(f"Unexpected error converting {input_path}: {str(e)}")
        return False

def main():
    parser = argparse.ArgumentParser(description='Convert FLAC files to AIFF format recursively.')
    parser.add_argument('folder_path', help='Path to the folder containing FLAC files')
    parser.add_argument('--keep-original', action='store_true', 
                       help='Keep original FLAC files after conversion (default: delete)')
    args = parser.parse_args()
    
    start_dir = Path(args.folder_path).resolve()
    if not start_dir.exists():
        print(f"Error: The directory '{start_dir}' does not exist.")
        return
    if not start_dir.is_dir():
        print(f"Error: '{start_dir}' is not a directory.")
        return
    
    
    converted = 0
    deleted = 0
    errors = 0
    
    for flac_file in start_dir.rglob('*.flac'):
        try:
            success = convert_flac_to_aiff(flac_file, not args.keep_original)
            if success:
                converted += 1
                if not args.keep_original and not flac_file.exists():
                    deleted += 1
        except Exception as e:
            print(f"Failed to process {flac_file}: {str(e)}")
            errors += 1
    
    print("\nConversion Complete!")
    print(f"Successfully converted: {converted} files")
    if not args.keep_original:
        print(f"Original files deleted: {deleted} files")
    if errors > 0:
        print(f"Errors encountered: {errors} files")

if __name__ == "__main__":
    main()
