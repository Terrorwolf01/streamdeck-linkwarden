#!/usr/bin/env bash

set -euo pipefail

usage() {
	echo "Usage: $0 [-b|--binary <name>] [-t|--target <triple>]..."
	echo "  -b, --binary <name>    Defaults to the package name in Cargo.toml"
	echo "  -t, --target <triple>  Comma-separated, repeatable. Defaults to every target listed in assets/manifest.json's CodePaths"
	echo "Example: $0"
	echo "Example: $0 --binary myplugin --target x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu"
	exit 1
}

binary_name=""
targets=()

while [ $# -gt 0 ]; do
	case "$1" in
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

build_args=()
if [ -n "$binary_name" ]; then
	build_args+=(--binary "$binary_name")
fi
for target in "${targets[@]}"; do
	build_args+=(--target "$target")
done

uuid=$(jq -r '.UUID' assets/manifest.json)
work_dir=$(mktemp -d)
trap 'rm -rf "$work_dir"' EXIT

./build.sh --output "$work_dir/$uuid.sdPlugin" "${build_args[@]}"

output_file="$uuid.sdPlugin.streamDeckPlugin"
rm -f "$output_file"
(cd "$work_dir" && zip -rq - "$uuid.sdPlugin") >"$output_file"

echo "Packaged $output_file"