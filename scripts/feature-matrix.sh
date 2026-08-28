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

# Count the tests that ran, not the result lines. An arm that compiled and ran
# nothing prints one `test result: ok` per target and would otherwise pass for
# a working configuration.
run() {
    local label="$1"; shift
    printf '%-46s ' "$label"
    if out=$("$@" 2>&1); then
        local ran
        ran=$(echo "$out" | sed -nE 's/^test result: ok\. ([0-9]+) passed.*/\1/p' \
            | awk '{n += $1} END {print n + 0}')
        if [ "$ran" -eq 0 ]; then
            printf 'FAILED (compiled and ran no tests)\n'
            fail=1
        else
            printf 'ok    %s\n' "$ran"
        fi
    else
        printf 'FAILED\n'
        echo "$out" | grep -E '^error' | head -5 | sed 's/^/    /'
        fail=1
    fi
}

# Sourcing gets the functions and runs nothing, which is what the test beside
# this file needs. Executing runs the matrix.
main() {
    run "nightly, defaults"            cargo test -p notko
    run "nightly, no defaults"         cargo test -p notko --no-default-features
    run "nightly, all features"        cargo test -p notko --all-features
    run "nightly, const only"          cargo test -p notko --no-default-features --features const
    run "nightly, try_trait_v2 only"   cargo test -p notko --no-default-features --features try_trait_v2
    run "nightly, macros only"         cargo test -p notko --no-default-features --features macros

    # `#[profile(Hot)]` rewrites to a different return type under `internal` with
    # `debug_assertions` off, and that arm is the crate's headline behaviour. A
    # workspace run never reaches it, since neither flag is on by default.
    run "macros, defaults"             cargo test -p notko-macros
    run "macros, internal + release"   cargo test -p notko-macros --features internal --release

    # The README tells a reader on stable to turn the defaults off. Both halves of
    # that sentence are checked: the crate works without them, and it genuinely
    # cannot be built with them, so the instruction is necessary rather than
    # cautious.
    if rustup toolchain list 2>/dev/null | grep -q '^stable'; then
        # `--skip` on the one target whose fixtures are blessed against the
        # pinned nightly's diagnostic wording. Stable words three of them
        # differently while every refusal still refuses, so running it here
        # measures the compiler's prose rather than the crate. The target
        # itself refuses to report a pass off the pin rather than skipping
        # quietly, which is why this has to say so out loud.
        run "stable, no defaults" cargo +stable test -p notko --no-default-features \
            -- --skip every_refusal_still_refuses
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

    return "$fail"
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then
    main
    exit $?
fi
