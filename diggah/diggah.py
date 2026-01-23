#!/usr/bin/env python3
import os
import sys
from collections.abc import Iterable
from datetime import datetime, timedelta
from typing import cast

import fire
import paramiko


class DateCalculator:
    """Handles date validation and range calculations."""

    @staticmethod
    def validate_date(date_str: str) -> datetime:
        try:
            return datetime.strptime(date_str, "%Y-%m-%d")
        except ValueError:
            raise ValueError(f"'{date_str}' is incorrent, please use YYYY-MM-DD")

    def get_range_from_week(
        self, year: int, month: int, week: int
    ) -> tuple[datetime, datetime]:
        """
        Calculates date range for a specific week.
        """
        first_of_month = datetime(int(year), int(month), 1)
        weeks_delta = timedelta(weeks=int(week) - 1)
        start_date = first_of_month + weeks_delta
        end_date = start_date + timedelta(days=7)
        return start_date, end_date

    def get_range_from_month(self, year: int, month: int) -> tuple[datetime, datetime]:
        """
        Calculates date range for a specific month.
        """
        start_date = datetime(int(year), int(month), 1)
        if month == 12:
            end_date = datetime(int(year) + 1, 1, 1)
        else:
            end_date = datetime(int(year), int(month) + 1, 1)
        return start_date, end_date

    def get_range_from_year(self, year: int) -> tuple[datetime, datetime]:
        """
        Calculates date range for a specific year.
        """
        start_date = datetime(int(year), 1, 1)
        end_date = datetime(int(year) + 1, 1, 1)
        return start_date, end_date


class FileSystemSearcher:
    """Handles searching the file system for modified items."""

    def find_items(
        self,
        start_ts: float | None,
        end_ts: float | None,
        include_files: bool = False,
        root: str = ".",
    ):
        """
        Yields paths of items modified between start_ts and end_ts (inclusive).
        """
        for dirpath, _, filenames in os.walk(root):
            if self._is_modified_in_range(dirpath, start_ts, end_ts):
                yield dirpath

            if include_files:
                for filename in filenames:
                    filepath = os.path.join(dirpath, filename)
                    if self._is_modified_in_range(filepath, start_ts, end_ts):
                        yield filepath

    def _is_modified_in_range(
        self, path: str, start_ts: float | None, end_ts: float | None
    ) -> bool:
        try:
            mtime = os.path.getmtime(path)
            if start_ts is not None and mtime <= start_ts:
                return False
            if end_ts is not None and mtime >= end_ts:
                return False
            return True
        except OSError:
            return False


class RemoteExecutor:
    """Handles executing the script on a remote host via SSH using Paramiko."""

    def execute(
        self,
        host_str: str,
        path: str,
        start_dt: datetime | None,
        end_dt: datetime | None,
        include_files: bool,
    ) -> Iterable[str]:
        """
        Executes a find command on the remote host and yields the results.
        """
        username = None
        hostname = host_str
        if "@" in host_str:
            username, hostname = host_str.split("@", 1)

        # Load SSH config
        ssh_config = paramiko.SSHConfig()
        user_config_file = os.path.expanduser("~/.ssh/config")
        if os.path.exists(user_config_file):
            with open(user_config_file) as f:
                ssh_config.parse(f)

        user_config = ssh_config.lookup(hostname)

        if "hostname" in user_config:
            hostname = user_config["hostname"]

        if username is None and "user" in user_config:
            username = user_config["user"]

        port = int(user_config.get("port", 22))
        key_filename = user_config.get("identityfile")

        cmd_parts = ["find", path]

        if not include_files:
            cmd_parts.extend(["-type", "d"])

        if start_dt:
            start_str = start_dt.strftime("%Y-%m-%d %H:%M:%S")
            cmd_parts.extend(["-newermt", f"'{start_str}'"])

        if end_dt:
            end_str = end_dt.strftime("%Y-%m-%d %H:%M:%S")
            cmd_parts.extend(["!", "-newermt", f"'{end_str}'"])

        cmd = " ".join(cmd_parts)

        client = paramiko.SSHClient()
        client.load_system_host_keys()
        client.set_missing_host_key_policy(paramiko.AutoAddPolicy())

        try:
            sock = None
            if "proxycommand" in user_config:
                sock = paramiko.ProxyCommand(user_config["proxycommand"])

            client.connect(
                hostname=hostname,
                username=username,
                port=port,
                key_filename=key_filename,
                sock=sock,
            )

            stdin, stdout, stderr = client.exec_command(cmd)
            stdin.close()

            # Read stderr first/async or just handle stdout for now.
            for line in cast(Iterable[str], stdout):
                line_str = str(line).strip()
                if line_str:
                    yield line_str

            for line in cast(Iterable[str], stderr):
                print(str(line), end="", file=sys.stderr)

            exit_status = stdout.channel.recv_exit_status()
            if exit_status != 0:
                sys.exit(exit_status)

        except Exception as e:
            print(f"({hostname}): {e}", file=sys.stderr)
            sys.exit(1)
        finally:
            client.close()


class Diggah:
    """
    Diggah: A tool to find modified files in a date range.
    """

    def search(
        self,
        start_date: str | None = None,
        end_date: str | None = None,
        year: int | None = None,
        month: int | None = None,
        week: int | None = None,
        today: bool = False,
        files: bool = False,
        all: bool = False,
        path: str = ".",
        dry_run: bool = False,
        host: str | None = None,
        relative: bool = False,
        output: str | bool = False,
    ):
        """
        Search for files/directories modified within a specified timeframe.

        Args:
            start_date: Start date (YYYY-MM-DD).
            end_date: End date (YYYY-MM-DD).
            year: Year for searching.
            month: Month for searching (optional).
            week: Week number for searching (optional).
            today: If True, search the last 24 hours.
            files: If True, include files in output (default is dirs only).
            all: If True, show all finds regardless of time.
            path: Directory to search in (default: ".").
            dry_run: If True, print calculated dates and exit.
            host: SSH host to run the command on (e.g., user@example.com).
            relative: If True, output paths relative to the search root.
            output: If True or a path, write output to file(s) instead of stdout.
               If a path is provided, writes to that file.
               If True (flag only), writes to default filename(s).
               For month search with default filename, this splits output into 4 weekly files.
        """

        ranges = self._determine_search_ranges(
            start_date, end_date, year, month, week, today, all, output
        )

        if not ranges:
            fire.Fire(self.search, command=["--help"], name="diggah search")
            return

        for start_dt, end_dt, outfile in ranges:
            if dry_run:
                self._print_dry_run(
                    start_dt, end_dt, path, files, host, relative, output, outfile
                )
            else:
                self._run_search(start_dt, end_dt, path, files, host, relative, outfile)

    def _determine_search_ranges(
        self,
        start_date: str | None,
        end_date: str | None,
        year: int | None,
        month: int | None,
        week: int | None,
        today: bool,
        all_time: bool,
        output: str | bool,
    ) -> list[tuple[datetime | None, datetime | None, str | None]]:
        """
        Determines the list of (start_dt, end_dt, outfile) tuples based on arguments.
        """
        calc = DateCalculator()
        ranges: list[tuple[datetime | None, datetime | None, str | None]] = []

        custom_outfile = os.path.expanduser(output) if isinstance(output, str) else None

        if all_time:
            if any([today, year, start_date, end_date]):
                raise ValueError("cannot combine with other time constraints")
            outfile = custom_outfile or ("all.txt" if output else None)
            ranges.append((None, None, outfile))

        elif today:
            if start_date or year:
                raise ValueError("cannot combine with specific dates")
            now = datetime.now()
            start_dt = now - timedelta(days=1)
            end_dt = now
            outfile = custom_outfile or (
                f"{now.strftime('%Y-%m-%d')}.txt" if output else None
            )
            ranges.append((start_dt, end_dt, outfile))

        elif year is None and month is not None:
            # Default to current year if month is specified but year is not
            year = datetime.now().year

        if year is not None:
            year = int(year)
            if start_date:
                raise ValueError("cannot combine with positional dates")

            if month is None:
                # Year only
                start_dt, end_dt = calc.get_range_from_year(year)
                outfile = custom_outfile or (f"{year}.txt" if output else None)
                ranges.append((start_dt, end_dt, outfile))
            else:
                month = int(month)
                if week is not None:
                    # Specific week
                    week = int(week)
                    start_dt, end_dt = calc.get_range_from_week(year, month, week)
                    outfile = custom_outfile or (
                        f"{week}_{month:02d}_{year}.txt" if output else None
                    )
                    ranges.append((start_dt, end_dt, outfile))
                else:
                    # Whole month
                    if custom_outfile:
                        start_dt, end_dt = calc.get_range_from_month(year, month)
                        ranges.append((start_dt, end_dt, custom_outfile))
                    elif output:
                        # Split by week if writing to default files
                        for i in range(1, 5):
                            s, e = calc.get_range_from_week(year, month, i)
                            outfile = f"{i}_{month:02d}_{year}.txt"
                            ranges.append((s, e, outfile))
                    else:
                        start_dt, end_dt = calc.get_range_from_month(year, month)
                        outfile = None
                        ranges.append((start_dt, end_dt, outfile))

        elif start_date and end_date:
            start_dt = calc.validate_date(start_date)
            end_dt = calc.validate_date(end_date)
            outfile = custom_outfile or (
                f"{start_date}_{end_date}.txt" if output else None
            )
            ranges.append((start_dt, end_dt, outfile))

        return ranges

    def _print_dry_run(
        self,
        start_dt: datetime | None,
        end_dt: datetime | None,
        path: str,
        include_files: bool,
        host: str | None,
        relative: bool,
        output: str | bool,
        outfile: str | None,
    ):
        print(f"Start:         {start_dt if start_dt else 'None (All Time)'}")
        print(f"End:           {end_dt if end_dt else 'None (All Time)'}")
        print(f"Path:          {path}")
        print(f"Include Files: {include_files}")
        if host:
            print(f"Host:          {host}")
            print("Mode:          Remote (find command)")
        else:
            print("Mode:          Local (python walk)")
        if relative:
            print("Output:        Relative paths")
        if output:
            print(f"Write to:      {outfile}")

    def _run_search(
        self,
        start_dt: datetime | None,
        end_dt: datetime | None,
        path: str,
        include_files: bool,
        host: str | None,
        relative: bool,
        outfile: str | None,
    ):
        results: Iterable[str]
        if host:
            results = RemoteExecutor().execute(
                host, path, start_dt, end_dt, include_files
            )
        else:
            start_ts = start_dt.timestamp() if start_dt else None
            end_ts = end_dt.timestamp() if end_dt else None
            results = FileSystemSearcher().find_items(
                start_ts,
                end_ts,
                include_files=include_files,
                root=path,
            )

        f = None
        if outfile:
            try:
                f = open(outfile, "w")
                print(f"{outfile}")
            except IOError as e:
                print(f"opening output file: {e} failed", file=sys.stderr)
                return

        try:
            for item in results:
                if relative:
                    item = os.path.relpath(item, start=path)

                if f:
                    _ = f.write(item + "\n")
                else:
                    print(item)
        finally:
            if f:
                f.close()


def main():
    try:
        fire.Fire(Diggah())
    except ValueError as e:
        print(f"{e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
