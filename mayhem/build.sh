#!/usr/bin/env bash
#
# bat/mayhem/build.sh — build the `bat-fuzz` Mayhem target as an in-process
# libFuzzer+ASan harness (cargo-fuzz, the fleet's proven Rust route) over bat's own
# PrettyPrinter library API, plus bat's own test suite with normal flags so
# mayhem/test.sh only RUNS it.
#
# The harness (mayhem/fuzz/fuzz_targets/bat_stdin_input.rs) depends on bat as a library and
# drives PrettyPrinter — the same surface as the fork's original raw libFuzzer
# `bat_stdin_input` harness — and keeps the legacy target name `bat-fuzz`.
#
# AIR-GAPPED CONTRACT (SPEC §6.5): the PATCH tier re-runs THIS script OFFLINE.
#   - This FIRST build (in CI, online) populates the cargo registry under $CARGO_HOME.
#   - The PATCH re-run resolves crates from that cache. The rlenv runtime exports
#     CARGO_NET_OFFLINE=true for the re-run so cargo won't refresh the crates.io
#     index — so do NOT hard-code `--offline` here.
set -euo pipefail

# clang rejects SOURCE_DATE_EPOCH='' — must be unset or a valid integer.
[ -n "${SOURCE_DATE_EPOCH:-}" ] || unset SOURCE_DATE_EPOCH

: "${MAYHEM_JOBS:=$(nproc)}"
export CARGO_BUILD_JOBS="$MAYHEM_JOBS"

cd "$SRC"

# Debug-info contract (SPEC §6.2 item 10): DWARF <= 3 on the fuzz binary (Mayhem
# triage cannot read DWARF >= 4). Overridable via $RUST_DEBUG_FLAGS.
: "${RUST_DEBUG_FLAGS:=-Cdebuginfo=2 -Zdwarf-version=3}"
# Sanitizer contract: $SANITIZER_FLAGS comes from the base ENV (clang syntax); rustc
# takes -Zsanitizer instead, so map non-empty -> ASan and an EXPLICIT empty -> none.
SANITIZER_FLAGS="${SANITIZER_FLAGS=-fsanitize=address}"
RUST_SANITIZER=""
[ -n "$SANITIZER_FLAGS" ] && RUST_SANITIZER="-Zsanitizer=address"
export RUSTFLAGS="${RUSTFLAGS:-} --cfg fuzzing ${RUST_SANITIZER} ${RUST_DEBUG_FLAGS} -Cforce-frame-pointers"

# DWARF<4 first-CU anchor: rustc's prebuilt ASan runtime ships DWARF-5 and would
# land at .debug_info offset 0. Link a clang -gdwarf-3 anchor object FIRST via a
# -Clinker cc-wrapper so the first CU is DWARF-3.
ANCHOR_DIR=/tmp/mayhem-dwarf3
mkdir -p "$ANCHOR_DIR"
echo 'int mayhem_dwarf3_anchor(void) { return 0; }' > "$ANCHOR_DIR/anchor.c"
clang -c -gdwarf-3 -O2 -o "$ANCHOR_DIR/anchor.o" "$ANCHOR_DIR/anchor.c"
printf '#!/usr/bin/env bash\nexec cc %s "$@"\n' "$ANCHOR_DIR/anchor.o" > "$ANCHOR_DIR/cc-wrap.sh"
chmod +x "$ANCHOR_DIR/cc-wrap.sh"
export RUSTFLAGS="$RUSTFLAGS -Clinker=$ANCHOR_DIR/cc-wrap.sh"

FUZZ_DIR="mayhem/fuzz"
TRIPLE="x86_64-unknown-linux-gnu"

echo "=== cargo fuzz build (image nightly, ASan via RUSTFLAGS) ==="
echo "RUSTFLAGS=$RUSTFLAGS"
# --debug-assertions: overflow/bounds checks become panics -> halting bugs libFuzzer
# catches. Uses the image's DEFAULT toolchain (the Dockerfile pinned it).
cargo fuzz build --fuzz-dir "$FUZZ_DIR" -O --debug-assertions bat-fuzz
bin="$SRC/$FUZZ_DIR/target/$TRIPLE/release/bat-fuzz"
[ -x "$bin" ] || { echo "ERROR: expected fuzz binary not found at $bin" >&2; exit 1; }
cp "$bin" /mayhem/bat-fuzz
echo "built /mayhem/bat-fuzz"

# Build bat's OWN test suite with the project's NORMAL flags (a clean, non-sanitized
# build) so mayhem/test.sh only RUNS it. Build the library/unit tests and the
# integration test binaries; the integration suite drives the `bat` binary, so build
# that too. Restrict to the default feature set (application + git).
echo "=== cargo test --no-run (normal flags) ==="
env -u RUSTFLAGS cargo test --no-run --locked
env -u RUSTFLAGS cargo build --locked   # integration tests exec target/debug/bat

echo "build.sh complete"
