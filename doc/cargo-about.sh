#! /bin/bash

# Utility script that invokes cargo-about [1] to
# generate a file copying.html with all related licenses.
#
# [1] https://github.com/EmbarkStudios/cargo-about

set -euo pipefail

project_dir="$(dirname "$(realpath -e "$0")")/.."
copying_file="$project_dir/copying.html"

# Copy the required files into the root directory.
cd "$project_dir"
cp doc/about.hbs doc/about.toml .

# Genreate the copying file.
cargo about generate about.hbs > "$copying_file"

# Clean up and finish.
rm -f about.hbs about.toml
echo "$0: created $(realpath $copying_file)" >&2
