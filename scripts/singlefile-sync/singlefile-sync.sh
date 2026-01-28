#!/bin/bash
/usr/sbin/inotifywait -m -e create,moved_to --format '%w%f' --include '.*(_ff(\([0-9]+\))?)\.html$' "$HOME"/Downloads |
	while IFS= read -r fullpath; do
		filename=$(basename "$fullpath")
		filename_sanitized=$(echo "$filename" | sed -E 's/(_ff(\([0-9]+\))?)\.html$//')

		if [[ -f "$fullpath" ]]; then
			mdfile="$HOME/misc/notes/www/${filename_sanitized}.md"

			attempt=1
			until {
				"$HOME"/bin/html2markdown <"$fullpath" >"$mdfile"
				[[ -s "$mdfile" ]] && [[ $(wc -c <"$mdfile") -gt 10 ]]
			} || ((attempt >= 3)); do
				sleep $((attempt * 2))
				((attempt++))
			done

			if [[ ! -s "$mdfile" ]] || [[ $(wc -c <"$mdfile") -le 10 ]]; then
				echo "err: Failed to convert $filename after 3 attempts"
			fi

			mkdir -p "$HOME"/misc/notes/www/html
			mv "$fullpath" "$HOME"/misc/notes/www/html
			echo "done: $filename_sanitized.md"
		fi
	done

