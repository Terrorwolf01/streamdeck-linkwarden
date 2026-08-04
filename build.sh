#!/usr/bin/env bash

set -euo pipefail

usage() {
	echo "Usage: $0 [-u|--user | -f|--flatpak | -o|--output <path>] [-r|--reload] [-b|--binary <name>] [-t|--target <triple>]..."
	echo "  -u, --user             Install to the native OpenDeck config dir (~/.config/opendeck/plugins/<uuid>.sdPlugin) [default]"
	echo "  -f, --flatpak          Install to the OpenDeck Flatpak's config dir"
	echo "  -o, --output <path>    Install to a custom directory, e.g. ~/.config/opendeck/plugins/com.example.myplugin.sdPlugin"
	echo "  -r, --reload           Reload the plugin in OpenDeck after installing. Implied by -u/-f; needed with -o if that custom"
	echo "                         path is still a live OpenDeck plugin dir. No-op if OpenDeck isn't running."
	echo "      --no-reload        Don't reload, even with -u/-f."
	echo "  -b, --binary <name>    Defaults to the package name in Cargo.toml"
	echo "  -t, --target <triple>  Comma-separated, repeatable. Defaults to every target listed in assets/manifest.json's CodePaths"
	echo "Example: $0"
	echo "Example: $0 --flatpak"
	echo "Example: $0 --user --binary myplugin --target x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu"
	exit 1
}

uuid=$(jq -r '.UUID' assets/manifest.json)
output_directory=""
binary_name=""
targets=()
location="user"
reload_override=""

while [ $# -gt 0 ]; do
	case "$1" in
	-u | --user)
		output_directory="$HOME/.config/opendeck/plugins/$uuid.sdPlugin"
		location="user"
		shift
		;;
	-f | --flatpak)
		output_directory="$HOME/.var/app/me.amankhanna.opendeck/config/opendeck/plugins/$uuid.sdPlugin"
		location="flatpak"
		shift
		;;
	-o | --output)
		[ $# -ge 2 ] || usage
		output_directory=$2
		location="output"
		shift 2
		;;
	-r | --reload)
		reload_override="on"
		shift
		;;
	--no-reload)
		reload_override="off"
		shift
		;;
	-b | --binary)
		[ $# -ge 2 ] || usage
		binary_name=$2
		shift 2
		;;
	-t | --target)
		[ $# -ge 2 ] || usage
		IFS=',' read -ra new_targets <<<"$2"
		targets+=("${new_targets[@]}")
		shift 2
		;;
	*)
		usage
		;;
	esac
done

if [ -z "$output_directory" ]; then
	output_directory="$HOME/.config/opendeck/plugins/$uuid.sdPlugin"
fi

# -u/-f always reload since they're known-live OpenDeck dirs; -o only reloads if -r/--reload
# was also passed, since a custom path isn't necessarily where OpenDeck is looking. -r/--no-reload
# always win over that default, regardless of where they appear relative to -u/-f/-o.
if [ -n "$reload_override" ]; then
	[ "$reload_override" = "on" ] && reload=true || reload=false
elif [ "$location" = "output" ]; then
	reload=false
else
	reload=true
fi

if [ -z "$binary_name" ]; then
	binary_name=$(awk -F'"' '/^name = /{print $2; exit}' Cargo.toml)
fi

if [ ${#targets[@]} -eq 0 ]; then
	mapfile -t targets < <(jq -r '.CodePaths | keys[]' assets/manifest.json)
fi

cd pi
deno task build
cd ..

rm -rf "$output_directory"
cp -r assets/ "$output_directory"

for target in "${targets[@]}"; do
	cargo build --release --target="$target"
	suffix=""
	[[ "$target" == *windows* ]] && suffix=".exe"
	cp "target/$target/release/$binary_name$suffix" "$output_directory/$binary_name-$target$suffix"
done

if [ "$reload" = true ]; then
	if pgrep -x opendeck >/dev/null 2>&1; then
		opendeck --reload-plugin "$uuid.sdPlugin"
	else
		echo "OpenDeck is not running, skipping reload"
	fi
fi
