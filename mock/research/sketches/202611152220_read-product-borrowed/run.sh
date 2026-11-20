#!/bin/sh
# The commands of sketch N2, run from this directory with CARGO_TARGET_DIR set
# by the caller. Each attempt rewrites one out_*.txt and ends it with the exit
# status of the command. An attempt that does not build is a result, so
# nothing here stops on a failure. The compiler names the shipped crate by its
# absolute path, so the path of the repository root is cut from the output and
# what is left reads from that root.

tc=+nightly-2026-05-28
root=$(cd ../../.. && pwd)

keep() {
    out=$1
    status=$2
    sed "s#$root/##g" "$out.raw" > "$out"
    rm "$out.raw"
    echo "exit status: $status" >> "$out"
}

build() {
    out=$1
    shift
    cargo $tc build -q "$@" > "$out.raw" 2>&1
    keep "$out" $?
}

run() {
    out=$1
    shift
    cargo $tc run -q "$@" > "$out.raw" 2>&1
    keep "$out" $?
}

# Attempts 1 to 3, against the shipped crate.
build out_a1_the_member_refuses.txt -p read-product-borrowed --bin a1_the_member_refuses
run out_a2_frame_carrier.txt -p read-product-borrowed --bin a2_frame_carrier
build out_a2_twin_outlives_the_frame.txt -p read-product-borrowed --bin a2_twin_outlives_the_frame
build out_a3_form_per_member.txt -p read-product-borrowed --bin a3_form_per_member

# Attempt 4, the copy with the two `'static` bounds struck, on both of its
# feature paths.
build out_b0_copy_default_features.txt -p notko-hlist-no-static
build out_b0_copy_no_default_features.txt -p notko-hlist-no-static --no-default-features

# Attempt 5, the model over the shipped list, and its twin.
run out_c1_form_per_member.txt -p read-product-borrowed --bin c1_form_per_member
build out_c1_twin_outlives_the_frame.txt -p read-product-borrowed --bin c1_twin_outlives_the_frame
