#!/usr/bin/env bash
set -euo pipefail

material_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_dir="$(cd "$material_dir/../../.." && pwd)"
exec "$project_dir/scripts/generate_tech_maps" "$material_dir"
