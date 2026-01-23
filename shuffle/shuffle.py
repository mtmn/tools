#!/usr/bin/env python3
import argparse
import random
import subprocess
import sys
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(description="picks and plays random albums in mpd")
    _ = parser.add_argument("file", type=Path)
    _ = parser.add_argument(
        "-n",
        "--number",
        type=int,
        help="number of random albums (default: 5)",
    )
    _ = parser.add_argument(
        "-d",
        "--depth",
        type=int,
        help="minimum path depth to include (default: 0)",
    )

    class Arguments(argparse.Namespace):
        file: Path = Path()
        number: int = 5
        depth: int = 2

    args = parser.parse_args(namespace=Arguments())

    with open(args.file) as f:
        lines = [line.strip() for line in f if line.strip()]

    if args.depth > 0:
        lines = [line for line in lines if line.count("/") >= args.depth]

    if not lines:
        print("file is empty or no lines match depth criteria", file=sys.stderr)
        sys.exit(1)

    n: int = min(args.number, len(lines))
    selected: list[str] = random.sample(lines, n)

    for line in selected:
        print(f"  → {line}")
        try:
            result = subprocess.run(["mpc", "add", line], check=True)
            _ = result
        except subprocess.CalledProcessError as e:
            print(f"error adding '{line}': {e}", file=sys.stderr)
        except OSError:
            print("mpc not found", file=sys.stderr)
            sys.exit(1)


if __name__ == "__main__":
    main()
