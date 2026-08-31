#!/usr/bin/env bash
# What `feature-matrix.sh` does when an arm goes wrong.
#
# The runner is the part that can silently stop working: a matrix that reports
# every arm green because it stopped noticing failure is worse than not having
# one, and nothing about its output would say so.
#
# Three ways an arm is not ok, and each is checked against a fake command
# rather than a real cargo run:
#
#   - it exits non-zero
#   - it exits zero having run no tests at all
#   - it exits zero and prints nothing a result line could be read out of
#
# Usage: scripts/feature-matrix_test.sh
set -uo pipefail

cd "$(dirname "$0")/.."

# shellcheck source=scripts/feature-matrix.sh
source scripts/feature-matrix.sh

failures=0

expect() {
    local what="$1" want="$2" got="$3"
    if [ "$want" = "$got" ]; then
        printf '  ok    %s\n' "$what"
    else
        printf '  FAIL  %s: wanted %s, got %s\n' "$what" "$want" "$got"
        failures=$((failures + 1))
    fi
}

# `run` reports through the shared `fail`, so each case starts from clean.
check() {
    local what="$1" want_fail="$2" want_out="$3"; shift 3
    fail=0
    # Not `$(run ...)`. A command substitution is a subshell, so the `fail` the
    # runner sets would be set in a copy and read back as zero here, and every
    # one of these cases would pass while checking nothing.
    local captured
    captured=$(mktemp)
    run "probe" "$@" >"$captured"
    out_line=$(cat "$captured")
    rm -f "$captured"
    expect "$what: fail flag" "$want_fail" "$fail"
    case "$out_line" in
        *"$want_out"*) printf '  ok    %s: output\n' "$what" ;;
        *) printf '  FAIL  %s: wanted %s in %s\n' "$what" "$want_out" "$out_line"
           failures=$((failures + 1)) ;;
    esac
}

echo "an arm that exits non-zero"
check "non-zero" 1 "FAILED" bash -c 'echo "error: nope" >&2; exit 1'

echo "an arm that passes and runs nothing"
# The case the counter exists for. Cargo prints one of these per target, so an
# arm whose targets are all filtered out looks identical to a working one until
# the number is read.
check "zero tests" 1 "ran no tests" bash -c 'echo "test result: ok. 0 passed; 0 failed"'

echo "an arm that prints no result line at all"
check "no output" 1 "ran no tests" bash -c 'echo "nothing to do"'

echo "an arm that really did run tests"
check "real tests" 0 "ok    7" bash -c 'echo "test result: ok. 7 passed; 0 failed"'

echo "an arm whose targets each report separately"
# Several `test result` lines is the normal shape: lib, each integration test,
# then doctests. The count is their sum, not the number of lines.
check "summed" 0 "ok    12" bash -c 'printf "test result: ok. 5 passed; 0 failed\ntest result: ok. 7 passed; 0 failed\n"'

if [ "$failures" -eq 0 ]; then
    echo "all good"
    exit 0
fi
echo "$failures check(s) failed"
exit 1
