#!/bin/sh
# The commands of sketch N3, run from this directory with CARGO_TARGET_DIR set
# by the caller. Each cell rewrites one out_*.txt and ends it with the exit
# status of the program. A cell that panics or aborts is a result, so nothing
# here stops on a failure.

tc=+nightly-2026-05-28

cell() {
    out=$1
    shift
    cargo $tc run -q --bin refuse "$@" > "$out" 2>&1
    echo "exit status: $?" >> "$out"
}

cell out_dev_feature_off.txt
cell out_dev_feature_on.txt --features internal
cell out_release_feature_off.txt --release
cell out_release_feature_on.txt --release --features internal
cell out_release_checked_feature_on.txt --profile release-checked --features internal
cell out_release_abort_feature_on.txt --profile release-abort --features internal
