#!/bin/sh
# The commands of this sketch, run from this directory as one command. Each
# output lands in `out_<bin>.txt` with the checkout path struck and the exit
# status appended; nothing here stops on a failure, since half the binaries
# are expected to fail.
root=$(cd ../../.. && pwd)
tc=+nightly-2026-05-28
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/../target/sketch-carrier}"

keep() {
    out=$1
    status=$2
    sed "s#$root/##g" "$out.raw" > "$out"
    rm "$out.raw"
    echo "exit status: $status" >> "$out"
}

run() {
    bin=$1
    cargo $tc run -q -p consumer --bin "$bin" > "out_$bin.txt.raw" 2>&1
    keep "out_$bin.txt" $?
}

run w1_form_per_member
run r1_two_forms_for_one_member
run r2_a_uniform_carrier_overridden
run r3_no_form_for_a_member
run r4_same_fields_other_list
run r5_same_list_other_carrier
run r6_twin_outlives_the_frame
