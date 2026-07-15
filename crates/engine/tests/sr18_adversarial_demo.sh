#!/usr/bin/env bash
# SR-18 adversarial demo — prove the three new gates in no_stray_test_binaries.rs
# actually fire. Per the SR-track rule, each attack first asserts it changed a
# file (an attack that changes nothing proves nothing), then the gate is run and
# must fail on exactly the intended test.
#
# Run from the workspace root:  bash crates/engine/tests/sr18_adversarial_demo.sh
set -u

CARGO="$HOME/.cargo/bin/cargo"
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT" || exit 1
TESTS="crates/engine/tests"
REG="$TESTS/proptest-regressions"

pass=0
fail=0
note() { printf '\n=== %s ===\n' "$1"; }
# run one gate test; succeed iff it FAILS (the attack must be caught)
expect_caught() {
  local test_name="$1" label="$2"
  if $CARGO test -p mtg-engine --test no_stray_test_binaries "$test_name" -- --exact \
       2>&1 | grep -qE "FAILED|panicked"; then
    echo "  [CAUGHT] $label -> $test_name failed as intended"; pass=$((pass+1))
  else
    echo "  [SURVIVED] $label -> $test_name did NOT fail (BAD)"; fail=$((fail+1))
  fi
}
assert_changed() { # path
  if [ -e "$1" ]; then echo "  attack changed a file: $1"; else
    echo "  ATTACK NO-OP: $1 absent (attack proves nothing)"; fail=$((fail+1)); fi
}

# --- Attack 1: a stray .rs in the exempted proptest-regressions dir -----------
# Cargo never compiles it (no main.rs there); its tests silently do not exist.
note "Attack 1: stray .rs in exempted dir (proptest-regressions/)"
mkdir -p "$REG"
printf '#[test]\nfn ghost() { assert!(false, "never compiled"); }\n' > "$REG/attack_stray.rs"
assert_changed "$REG/attack_stray.rs"
expect_caught exempt_dirs_contain_no_rust_files "stray .rs in exempted dir"
rm -rf "$REG"

# --- Attack 2: a main.rs in the exempted dir -> ungoverned auto-built target ---
# The group-existence check filters NON_GROUP_DIRS out before comparing, so this
# target is invisible to it. auto_built_targets_match_expected admits no exemption.
note "Attack 2: main.rs in exempted dir becomes an ungoverned target"
mkdir -p "$REG"
printf 'fn main() {}\n' > "$REG/main.rs"
assert_changed "$REG/main.rs"
expect_caught auto_built_targets_match_expected "main.rs in exempted dir"
rm -rf "$REG"

# --- Attack 3a/b/c: module-level #![cfg] deletes a whole module's tests ---------
# Three forms, all valid Rust that compiles the module out: the plain attribute, an
# interior-whitespace form (`# ! [ cfg`), and one hidden behind a block comment. The
# comment/whitespace-aware detector must catch all three (an SR review found the
# earlier split("//") form missed the latter two).
TARGET="$TESTS/combat/additional_combat.rs"
ORIG="$(cat "$TARGET")"
for form in '#![cfg(any())]' '# ! [ cfg (any())]' '/* off */ #![cfg(any())]'; do
  note "Attack 3: module-level cfg attribute -> $form"
  printf '%s\n%s' "$form" "$ORIG" > "$TARGET"
  if git diff --quiet -- "$TARGET"; then
    echo "  ATTACK NO-OP: $TARGET unchanged (attack proves nothing)"; fail=$((fail+1))
  else
    echo "  attack changed a file: $TARGET"
  fi
  expect_caught no_module_level_cfg_in_group_files "cfg form: $form"
  git checkout -- "$TARGET"
done

note "SR-18 demo result: $pass caught / $fail problems"
[ "$fail" -eq 0 ] && echo "ALL ATTACKS CAUGHT" || echo "SOME ATTACKS SURVIVED"
exit "$fail"
