#!/usr/bin/env bash
# SR-27 adversarial demonstration.
#
# The gates added to tests/core/protocol_schema.rs are only worth anything if they
# actually redden when someone defeats strict lockstep. Each attack below perturbs
# real source, then asserts EXACTLY the intended gate goes red — and, per the SR
# track rule, first asserts the perturbation actually changed the file. An attack
# that changed nothing proves nothing; that has been the recurring failure mode.
#
# What is demonstrated (per the SR-27 brief):
#   A. a wire-shape change with no pin update fails the recompute gate
#      (baseline: proves the SR-8 gate still fires);
#   B. THE re-pin cheat — change the wire, re-pin PROTOCOL_SCHEMA_FINGERPRINT *and*
#      the tail history row, and DON'T bump PROTOCOL_VERSION — silences the recompute
#      gate but is caught by the append-only frozen-baseline gate (the SR-27 finding);
#   C. bumping PROTOCOL_VERSION but forgetting to append a history row fails the
#      append-only gate (tail.version != current).
#
# The token-anchor (serialize_guard_is_token_anchored) and serde-conversion
# (serde_conversion_scan_detects_the_attribute + no_serde_conversion_attributes_in_closure)
# gates are proven by unit tests, not a source patch: dropping a `Serialize` derive or
# adding a compile-valid `serde(with/from/into)` both require a companion impl/module a
# text patch cannot inject compile-safely, and a test binary that will not compile
# cannot be observed reddening. This script runs those unit tests and asserts they pass.
#
# Run from the workspace root:  bash crates/engine/tests/sr27_adversarial_demo.sh
# It mutates tracked source and restores it with `git checkout`, so commit first.
set -u
CARGO="$HOME/.cargo/bin/cargo"
PROTO=crates/engine/src/rules/protocol.rs
OLD_FP="ba7907d9f51a65acba39ccf020a14bd6234f637731c934490a7cbf749e5f97b6"
fails=0

run_test() { # <test-name> -> green | red | absent
  local out
  out=$("$CARGO" test -p mtg-engine --test core "protocol_schema::$1" -- --exact 2>&1)
  if   echo "$out" | grep -q "test result: ok. 1 passed"; then echo green
  elif echo "$out" | grep -q "test result: FAILED. 0 passed; 1 failed"; then echo red
  else echo absent; fi
}

check() { # <want:red|green> <test> <desc> <changed-file>...
  local want="$1" test="$2" desc="$3"; shift 3
  local f changed=0
  for f in "$@"; do git diff --quiet -- "$f" || changed=1; done
  if [ "$changed" -eq 0 ]; then
    echo "  ✗ ATTACK NO-OP: $desc — no file changed; proves nothing"; fails=$((fails+1)); return
  fi
  local verdict; verdict=$(run_test "$test")
  if [ "$verdict" = "$want" ]; then
    echo "  ✓ $test = $want:  $desc"
  else
    echo "  ✗ $test = $verdict (wanted $want):  $desc"; fails=$((fails+1))
  fi
}

# Same as check but does not require a file change (used for the unit-test gates).
check_nochange() { # <want> <test> <desc>
  local want="$1" test="$2" desc="$3"
  local verdict; verdict=$(run_test "$test")
  if [ "$verdict" = "$want" ]; then
    echo "  ✓ $test = $want:  $desc"
  else
    echo "  ✗ $test = $verdict (wanted $want):  $desc"; fails=$((fails+1))
  fi
}

restore() { git checkout -q -- "$@"; }

echo "SR-27 adversarial demonstration"

# ── Attack A: wire-shape change, no pin update ────────────────────────────────
# A #[serde(rename)] on a wire-frame field renames a serialized key — a wire change
# that compiles cleanly. The recompute gate must catch it.
perl -0pi -e 's/(\n\s*)pub commands: Vec<Command>,/\1#[serde(rename = "sr27_probe")]\1pub commands: Vec<Command>,/' "$PROTO"
check red protocol_schema_fingerprint_is_pinned "serde rename on a wire field moves the digest" "$PROTO"
restore "$PROTO"

# ── Attack B: the re-pin cheat, no bump ───────────────────────────────────────
# The realistic cheat: make the shape change, then instead of bumping the version
# and appending a row, re-pin BOTH PROTOCOL_SCHEMA_FINGERPRINT and the tail history
# row (they hold the same value) to the new digest. That silences the recompute gate
# — but the FROZEN baseline constant makes history_is_append_only red anyway.
perl -0pi -e 's/(\n\s*)pub commands: Vec<Command>,/\1#[serde(rename = "sr27_probe")]\1pub commands: Vec<Command>,/' "$PROTO"
NEW_FP=$("$CARGO" test -p mtg-engine --test core "protocol_schema::protocol_schema_fingerprint_is_pinned" -- --exact 2>&1 \
  | grep -A1 'APPEND a new PROTOCOL_HISTORY row' | grep -oE '[0-9a-f]{64}' | head -1)
if [ -z "${NEW_FP:-}" ] || [ "$NEW_FP" = "$OLD_FP" ]; then
  echo "  ✗ SETUP FAILED: could not capture a moved fingerprint"; fails=$((fails+1))
else
  # Re-pin both occurrences of the old digest in protocol.rs (const + tail row).
  perl -pi -e "s/\Q$OLD_FP\E/$NEW_FP/g" "$PROTO"
  check green protocol_schema_fingerprint_is_pinned "re-pinning silences the recompute gate" "$PROTO"
  check red   history_is_append_only               "...but the frozen baseline rejects the re-pin" "$PROTO"
  check green history_tail_matches_the_fingerprint_const "...and the tail still matches the const (both were edited)" "$PROTO"
fi
restore "$PROTO"

# ── Attack C: bump the version, forget to append a row ────────────────────────
# The other half: bump PROTOCOL_VERSION but leave PROTOCOL_HISTORY at version 2.
perl -pi -e 's/pub const PROTOCOL_VERSION: u32 = 2;/pub const PROTOCOL_VERSION: u32 = 3;/' "$PROTO"
check red history_is_append_only "version bumped to 3 but no history row appended" "$PROTO"
restore "$PROTO"

# ── Unit-proven gates (no compile-safe source patch exists) ───────────────────
check_nochange green serialize_guard_is_token_anchored \
  "token-anchor rejects Serializer / SerializeStruct substrings"
check_nochange green serde_conversion_scan_detects_the_attribute \
  "the serde(with/from/into/try_from) detector actually fires"
check_nochange green no_serde_conversion_attributes_in_closure \
  "the live closure carries no serde conversion attribute"

echo
if [ "$fails" -eq 0 ]; then
  echo "SR-27: all attacks caught exactly as intended."
else
  echo "SR-27: $fails attack(s) did not behave as intended."; exit 1
fi
