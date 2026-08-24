#!/usr/bin/env bash
# Run the suite in every feature configuration this crate claims to support,
# plus the two the README makes promises about on stable.
#
# `cargo test --workspace` does NOT cover the published default configuration.
# notko-macros dev-depends on notko with `try_trait_v2`, and a workspace build
# unifies features, so that flag is on for every target including the ones
# declaring `required-features` against it. The per-package runs below are what
# actually exercise the shipped shapes.
#
# Usage: scripts/feature-matrix.sh
set -euo pipefail

cd "$(dirname "$0")/.."

fail=0

run() {
    local label="$1"; shift
    printf '%-46s ' "$label"
    if out=$("$@" 2>&1); then
        printf 'ok    %s\n' "$(echo "$out" | grep -cE '^test result: ok')"
    else
        printf 'FAILED\n'
        echo "$out" | grep -E '^error' | head -5 | sed 's/^/    /'
        fail=1
    fi
}

run "nightly, defaults"            cargo test -p notko
run "nightly, no defaults"         cargo test -p notko --no-default-features
run "nightly, all features"        cargo test -p notko --all-features
run "nightly, const only"          cargo test -p notko --no-default-features --features const
run "nightly, try_trait_v2 only"   cargo test -p notko --no-default-features --features try_trait_v2
run "nightly, macros only"         cargo test -p notko --no-default-features --features macros

# The README tells a reader on stable to turn the defaults off. Both halves of
# that sentence are checked: the crate works without them, and it genuinely
# cannot be built with them, so the instruction is necessary rather than
# cautious.
if rustup toolchain list 2>/dev/null | grep -q '^stable'; then
    run "stable, no defaults" cargo +stable test -p notko --no-default-features
    printf '%-46s ' "stable, defaults refused"
    if cargo +stable build -p notko >/dev/null 2>&1; then
        printf 'FAILED (it built, so the README overstates the need)\n'
        fail=1
    else
        printf 'ok\n'
    fi
else
    echo "stable toolchain absent, skipping the two stable arms"
fi

exit "$fail"
