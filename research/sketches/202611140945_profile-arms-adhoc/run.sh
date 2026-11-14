#!/bin/sh
# The commands of this ad-hoc spike, from this directory. Each rewrites one
# out_*.txt. The timing binary first, in both arms the attribute emits, then
# the three caller probes in both arms. A probe that does not build is the
# result, so nothing here stops on a failure. In the audit session the lines
# were issued one after another by hand, since the session could not run a
# script file.

cargo run -q --release --bin tiers > out_tiers_release_debug_arm.txt 2>&1
cargo run -q --release --features internal --bin tiers > out_tiers_release_internal_arm.txt 2>&1

cargo build --lib --features probe_question_mark > out_probe_question_mark_debug_arm.txt 2>&1
cargo build --lib --release --features internal,probe_question_mark > out_probe_question_mark_release_arm.txt 2>&1
cargo build --lib --features probe_match > out_probe_match_debug_arm.txt 2>&1
cargo build --lib --release --features internal,probe_match > out_probe_match_release_arm.txt 2>&1
cargo run -q --release --features probe_is_err --bin recover > out_recover_debug_arm.txt 2>&1
cargo run -q --release --features internal,probe_is_err --bin recover > out_recover_release_arm.txt 2>&1
