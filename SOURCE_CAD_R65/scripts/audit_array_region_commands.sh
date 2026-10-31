#!/usr/bin/env bash
set -euo pipefail

# Focused regression audit for associative arrays, groups, data extraction and regions.
# Run from the repository root. Each filter is intentionally narrow so failures identify
# the command family that regressed.
if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo is not installed; source audit could not run" >&2
    exit 2
fi

cargo test -p cadcraft-engine arraypath_associative_keeps_path_model_and_serializable_samples
cargo test -p cadcraft-engine group_display_mode_is_persisted_and_validated
cargo test -p cadcraft-engine dataextraction_aliases_extract_block_references
cargo test -p cadcraft-engine regioncheck_reports_loops_and_validity
cargo test -p cadcraft-doc valid_region_round_trips_and_rejects_non_finite_vertices
