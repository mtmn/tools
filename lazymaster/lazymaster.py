#!/usr/bin/env python3
import subprocess
import sys
import json
from typing import cast


def main():
    if len(sys.argv) < 2:
        print("lazymaster input.wav [output.wav]")
        sys.exit(1)

    input_file = sys.argv[1]

    # Measure loudness
    cmd_measure = [
        "ffmpeg",
        "-i",
        input_file,
        "-af",
        "loudnorm=I=-13:TP=-1:LRA=8:print_format=json",
        "-f",
        "null",
        "-",
    ]

    result = subprocess.run(cmd_measure, capture_output=True, text=True)

    # Parse output values
    try:
        stderr_output = result.stderr
        json_start = stderr_output.find("{")
        json_end = stderr_output.rfind("}") + 1
        if json_start == -1 or json_end == -1:
            raise ValueError("json output is missing")

        stats = cast(dict[str, str], json.loads(stderr_output[json_start:json_end]))

        measured_i = stats["input_i"]
        measured_tp = stats["input_tp"]
        measured_lra = stats["input_lra"]
        measured_thresh = stats["input_thresh"]
        offset = stats["target_offset"]
    except (ValueError, json.JSONDecodeError, KeyError) as e:
        print(f"error parsing loudness stats from ffmpeg output {e}")
        sys.exit(1)

    print(
        f"I={measured_i}, TP={measured_tp}, LRA={measured_lra}, Thresh={measured_thresh}, Offset={offset}"
    )

    # Normalize loudness
    if len(sys.argv) > 2:
        output_file = sys.argv[2]
        cmd_normalize = [
            "ffmpeg",
            "-i",
            input_file,
            "-af",
            f"loudnorm=I=-13:TP=-1:LRA=8:measured_I={measured_i}:measured_LRA={measured_lra}:measured_TP={measured_tp}:measured_thresh={measured_thresh}:offset={offset}:linear=true",
            "-y",
            output_file,
        ]

        _ = subprocess.run(cmd_normalize, check=True)


if __name__ == "__main__":
    main()
