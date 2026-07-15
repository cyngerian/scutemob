#!/usr/bin/env bash
# SR-19 adversarial demonstration.
#
# The HashInto-vs-struct field-coverage gate (crates/engine/tests/core/hash_schema.rs)
# is only worth anything if it actually reddens when a field silently drops out of a
# HashInto impl, and if its NOT_HASHED allowlist cannot be used to wave a covered
# field through. Each attack below perturbs real source, then asserts EXACTLY the
# intended gate goes red — and, per the SR-track rule (SR-9a/9b/9c), first asserts
# the perturbation actually changed the file. A "passing" attack that changed nothing
# is the failure mode that has bitten every prior SR task.
#
# The three things demonstrated (per the SR-19 brief):
#   1. removing a live field from a struct's HashInto impl (the SR-7 haunt-field
#      failure mode) reddens the coverage gate;
#   2. an unlisted/bogus NOT_HASHED entry naming a field that IS in fact hashed is
#      rejected by the dead-entry guard (so the allowlist can't hide a real field);
#   3. a NOT_HASHED entry naming a field that does not exist is also rejected (the
#      other dead-entry arm);
#   ... and every attack asserts it changed a file before its verdict is trusted.
#
# Run from the workspace root:  bash crates/engine/tests/scripts/sr19_adversarial_demo.sh
# It mutates tracked source and restores it with `git checkout`, so commit first.
set -u
CARGO="$HOME/.cargo/bin/cargo"
HASH=crates/engine/src/state/hash.rs
GATE=crates/engine/tests/core/hash_schema.rs
fails=0

# Run one named hash_schema test. Echoes: green | red | absent.
run_test() { # <test-name>
  local out
  out=$("$CARGO" test -p mtg-engine --test core "hash_schema::$1" -- --exact 2>&1)
  if   echo "$out" | grep -q "test result: ok. 1 passed"; then echo green
  elif echo "$out" | grep -q "test result: FAILED. 0 passed; 1 failed"; then echo red
  else echo absent; fi
}

# Assert a file changed, then that <test> has <want> verdict. The caller restores.
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

echo "SR-19 adversarial demonstration"

# ── Attack 1: drop a live field from a HashInto impl ─────────────────────────
# Delete the `self.combat_damage_amount.hash_into(hasher);` feed from
# `impl HashInto for PendingTrigger`. It compiles cleanly (a dropped statement),
# and two states differing only in a combat-damage-amount trigger would then hash
# identically. The coverage gate must catch the now-uncovered field.
# -0 slurps the file; no /g, so only the FIRST feed (PendingTrigger's) is removed.
perl -0pi -e 's{\n\s*self\.combat_damage_amount\.hash_into\(hasher\);}{}' "$HASH"
check red every_hashed_struct_field_is_hashed_or_allowlisted \
  "dropping self.combat_damage_amount from PendingTrigger's HashInto reddens the gate" "$HASH"
restore "$HASH"

# ── Attack 2: bogus NOT_HASHED entry for a field that IS hashed ───────────────
# Add ("StackObject","was_dashed") to the allowlist. `was_dashed` is hashed, so the
# dead-entry guard must reject it — otherwise the allowlist could silence a real,
# covered field just by naming it.
perl -pi -e 's/const NOT_HASHED: &\[\(&str, &str\)\] = &\[\];/const NOT_HASHED: &[(&str, \&str)] = \&[("StackObject", "was_dashed")];/' "$GATE"
check red not_hashed_allowlist_has_no_dead_entries \
  "an allowlist entry for the already-hashed StackObject.was_dashed is a dead entry" "$GATE"
restore "$GATE"

# ── Attack 3: NOT_HASHED entry for a non-existent field ──────────────────────
# The other dead-entry arm: name a field the struct does not declare.
perl -pi -e 's/const NOT_HASHED: &\[\(&str, &str\)\] = &\[\];/const NOT_HASHED: &[(&str, \&str)] = \&[("PendingTrigger", "no_such_field")];/' "$GATE"
check red not_hashed_allowlist_has_no_dead_entries \
  "an allowlist entry for PendingTrigger.no_such_field is a dead entry" "$GATE"
restore "$GATE"

echo
if [ "$fails" -eq 0 ]; then
  echo "SR-19: all attacks caught exactly as intended."
else
  echo "SR-19: $fails attack(s) did not behave as intended."; exit 1
fi
