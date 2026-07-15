#!/usr/bin/env bash
# SR-17 adversarial demonstration.
#
# The two-axis gate behind HASH_SCHEMA_VERSION (crates/engine/tests/core/hash_schema.rs)
# is only worth anything if it actually reddens when the schema moves. Each attack
# below perturbs real source, then asserts that EXACTLY the intended gate goes red
# — and, per the SR-track rule (SR-9a/9b/9c), first asserts the perturbation
# actually changed the file. A "passing" attack that changed nothing is the failure
# mode that has bitten every prior SR task.
#
# The four things demonstrated (per the SR-17 brief):
#   1. a serde SHAPE change without a bump fails the declaration gate
#      (and the stream gate stays green — the two axes are independent);
#   2. a HashInto STREAM change without a bump fails the stream gate
#      (and the declaration gate stays green — independence, the other way);
#   3. a RE-PIN of the shipped row without a bump fails the append-only gate,
#      even after the declaration gate has been silenced by the re-pin;
#   4. every attack asserts it changed a file before its verdict is trusted.
#
# Run from the workspace root:  bash crates/engine/tests/scripts/sr17_adversarial_demo.sh
# It mutates tracked source and restores it with `git checkout`, so commit first.
set -u
CARGO="$HOME/.cargo/bin/cargo"
MOD=crates/engine/src/state/mod.rs
HASH=crates/engine/src/state/hash.rs
fails=0

# Run one named hash_schema test. Echoes: green | red | absent.
run_test() { # <test-name>
  local out
  out=$("$CARGO" test -p mtg-engine --test core "hash_schema::$1" -- --exact 2>&1)
  if   echo "$out" | grep -q "test result: ok. 1 passed"; then echo green
  elif echo "$out" | grep -q "test result: FAILED. 0 passed; 1 failed"; then echo red
  else echo absent; fi
}

# Assert a file changed, then that <test> has <want> verdict. Restores nothing —
# the caller restores, because attack 3 juggles two files across several steps.
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

restore() { git checkout -q -- "$@"; }

echo "SR-17 adversarial demonstration"

# ── Attack 1: serde shape change, no bump ────────────────────────────────────
# A #[serde(rename)] on an existing GameState field is a wire change (it renames a
# serialized key) that compiles cleanly and never touches the hasher. The
# declaration digest must catch it; the stream digest must not.
perl -0pi -e 's/(\n\s*)pub\(crate\) monarch: Option<PlayerId>,/\1#[serde(rename = "sr17_probe")]\1pub(crate) monarch: Option<PlayerId>,/' "$MOD"
check red   declaration_fingerprint_is_pinned "serde rename moves the declaration digest" "$MOD"
check green stream_fingerprint_is_pinned      "...and the hash stream is untouched by it" "$MOD"
restore "$MOD"

# ── Attack 2: HashInto stream change, no bump ────────────────────────────────
# An extra byte fed inside public_state_hash changes the hash stream and nothing
# about any type declaration. The stream digest must catch it; the declaration
# digest must not.
perl -0pi -e 's{(HASH_SCHEMA_VERSION\.hash_into\(&mut hasher\); // schema version \(see HASH_SCHEMA_VERSION const\))}{$1\n        99u8.hash_into(&mut hasher); // SR-17 demo perturbation}' "$HASH"
check red   stream_fingerprint_is_pinned      "an extra HashInto feed moves the stream digest" "$HASH"
check green declaration_fingerprint_is_pinned "...and the declaration shape is untouched by it" "$HASH"
restore "$HASH"

# ── Attack 3: re-pin the shipped row, no bump ────────────────────────────────
# The realistic cheat: make a shape change, then instead of bumping the version
# and appending a row, edit the version-39 row's fingerprint in place to the new
# value. That silences declaration_fingerprint_is_pinned — but the frozen baseline
# constants make history_is_append_only red anyway.
OLD_DECL=$(grep -oE 'decl_fingerprint: "[0-9a-f]{64}"' "$HASH" | head -1 | grep -oE '[0-9a-f]{64}')
# 3a. Re-apply the shape change so the real digest moves.
perl -0pi -e 's/(\n\s*)pub\(crate\) monarch: Option<PlayerId>,/\1#[serde(rename = "sr17_probe")]\1pub(crate) monarch: Option<PlayerId>,/' "$MOD"
# 3b. Capture the NEW computed decl fingerprint from the gate's own failure text.
NEW_DECL=$("$CARGO" test -p mtg-engine --test core "hash_schema::declaration_fingerprint_is_pinned" -- --exact 2>&1 \
  | grep -A1 'set its decl_fingerprint to:' | grep -oE '[0-9a-f]{64}' | head -1)
if [ -z "${NEW_DECL:-}" ] || [ "$NEW_DECL" = "$OLD_DECL" ]; then
  echo "  ✗ SETUP FAILED: could not capture a moved decl fingerprint"; fails=$((fails+1))
else
  # 3c. Re-pin the shipped row in hash.rs to the new value (the cheat), no bump.
  perl -pi -e "s/\Q$OLD_DECL\E/$NEW_DECL/" "$HASH"
  check green declaration_fingerprint_is_pinned "re-pinning silences the declaration gate" "$MOD" "$HASH"
  check red   history_is_append_only            "...but the frozen baseline still rejects the re-pin" "$MOD" "$HASH"
fi
restore "$MOD" "$HASH"

echo
if [ "$fails" -eq 0 ]; then
  echo "SR-17: all attacks caught exactly as intended."
else
  echo "SR-17: $fails attack(s) did not behave as intended."; exit 1
fi
