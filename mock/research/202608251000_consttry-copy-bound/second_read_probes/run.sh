#!/usr/bin/env bash
# Second-read probe set for the const_destruct vetting. One command, from this dir:
#   ./run.sh
# Each probe states its own question and its expected outcome in its header comment.
# Probes b_, c_, e3_, f_, g_ and i_ are NEGATIVE CONTROLS: they MUST fail to compile.
# If any of them ever compiles, the matching positive probe proves nothing, because
# the compiler would be accepting everything rather than reasoning about the feature.
set -u
TOOLCHAIN="${TOOLCHAIN:-nightly-2026-05-28}"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

# probe -> expected outcome. "pass" = must compile, "fail" = must be refused.
expect() { case "$1" in
  b_*|c_*|e3_*|f_*|g_*|i_*) echo fail ;;
  *) echo pass ;;
esac; }

bad=0
for f in *.rs; do
  b="${f%.rs}"
  out="$(rustc "+$TOOLCHAIN" --edition 2024 --crate-type bin --out-dir "$OUT" "$f" 2>&1)"
  st=$?
  { echo "### $f"; echo "### $(rustc "+$TOOLCHAIN" --version)"
    echo "### expected: $(expect "$b")"; echo
    echo "$out"; echo; echo "### rustc exit status: $st"; } > "$b.out.txt"
  want="$(expect "$b")"
  got=pass; [ "$st" -ne 0 ] && got=fail
  verdict=OK; [ "$want" != "$got" ] && { verdict="MISMATCH"; bad=1; }
  printf '%-42s want=%-4s got=%-4s %s\n' "$f" "$want" "$got" "$verdict"
done
[ "$bad" -eq 0 ] && echo "all probes matched expectation" || echo "SOME PROBES DID NOT MATCH"
exit "$bad"
