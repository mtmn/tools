# Diggah

Diggah is a command-line tool to find files and directories that were modified within a specific time range. It supports local file system searches as well as remote execution via ssh.

## Installation

Ensure you have `fire`, and `paramiko` installed locally.

## Usage

The main command is `search`. You can run the script directly:

```bash
python3 diggah.py search [ARGS]
```

### Examples

**1. Search valid relative to today (last 24h) in current directory:**
```bash
python3 diggah.py search --today
```

**2. Search in a specific path:**
```bash
python3 diggah.py search --today --path=/var/log
```

**3. Search for a specific week or month:**
```bash
# Find modifications in the 1st week of January 2025
python3 diggah.py search --year=2025 --month=1 --week=1

# Find modifications in the whole month of January 2025
python3 diggah.py search --year=2025 --month=1
```

**4. Search with specific dates:**
```bash
python3 diggah.py search --start_date=2025-01-01 --end_date=2025-01-07
```

**5. Include files in output (default is dirs only):**
```bash
python3 diggah.py search --today --all
```

**6. Output Relative Paths:**
```bash
# Output paths relative to the search directory (strip the prefix)
# e.g., /long/path/to/foo/bar -> foo/bar
python3 diggah.py search --today -r --path=/long/path/to
```

**7. Write to File (`-w`):**
```bash
# Save output to a file with relative path (YYYY-MM-DD.txt)
python3 diggah.py search --relative --today -w

# Save to weekly files (1_01_2025.txt ... 4_01_2025.txt)
python3 diggah.py search --year=2025 --month=1 -w
```

**7. Remote Execution:**
```bash
# Execute the search on a remote server
python3 diggah.py search --today --host=user@example.com --path=/opt/app/logs
```

### Help
To see all available options:
```bash
python3 diggah.py search --help
```
