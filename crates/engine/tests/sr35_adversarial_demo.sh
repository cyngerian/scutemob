#!/usr/bin/env bash
# SR-35 adversarial demonstration.
#
# `tools/check-defs-fmt.sh` is only worth anything if it actually reddens when a
# card def is misformatted — and, per the SR track rule, each attack below first
# asserts the perturbation really changed the file. An attack that changed nothing
# proves nothing; that has been the recurring failure mode on this track.
#
# What is demonstrated:
#
#   A. THE SR-35 finding — misformat a real def; `cargo fmt --all -- --check`
#      stays GREEN (it checks none of the 1,748 defs) while the new gate goes RED.
#
#   B. THE reason the naive fix is not enough — a def whose `oracle_text` is one
#      long line is INVISIBLE to rustfmt even when rustfmt is pointed straight at
#      it: the long string does not fit in `max_width`, rustfmt falls back to the
#      original source for the enclosing expression (the whole `CardDefinition`
#      literal, i.e. the whole file), and exits 0. Without `format_strings=true`
#      the gate passes a file with a blatant misindentation. With it, red.
#
#      This is not hypothetical: 1,380 of the 1,748 defs were in exactly that
#      state before this task reformatted them. The corpus is clean now, so the
#      attack authors a NEW def in the shape an author would write one — which is
#      precisely the regression the flag exists to stop as the corpus grows.
#
#   C. The skip named in the SR-35 brief — rustfmt formats a line, the result
#      still exceeds `max_width`, and rustfmt silently keeps the file's original
#      text and exits 0. Without `error_on_line_overflow=true` the gate passes it.
#      With it, red.
#
#   D. The gate refuses to pass vacuously when it finds no defs to check.
#
# Run from the workspace root:  bash crates/engine/tests/sr35_adversarial_demo.sh
# Attack A mutates tracked source and restores it with `git checkout`, so commit first.
set -u

CARGO="$HOME/.cargo/bin/cargo"
GATE=tools/check-defs-fmt.sh
VICTIM=crates/card-defs/src/defs/abrade.rs
# Fixtures live in a scratch dir, never in crates/card-defs/src/defs/: section C's
# fixture is deliberately non-compiling Rust, and build.rs discovers that directory
# by listing it — a concurrent `cargo build` would pick the fixture up and fail.
# Only attack A touches tracked source, because only attack A is about a real def.
SCRATCH=$(mktemp -d)
FIXTURE="$SCRATCH/zz_sr35_fixture.rs"
fails=0

cleanup() { git checkout -- "$VICTIM" 2>/dev/null; rm -rf "$SCRATCH"; }
trap cleanup EXIT

ok()   { echo "  PASS: $1"; }
bad()  { echo "  FAIL: $1"; fails=$((fails + 1)); }

# Stand up a throwaway corpus holding just the fixture and run the SHIPPED gate
# against it — the script derives its defs dir from its own location, so this
# exercises the real argument set rather than a copy of it. Echoes green|red.
shipped_gate_on_fixture() {
  local tmp; tmp=$(mktemp -d)
  mkdir -p "$tmp/crates/card-defs/src/defs" "$tmp/tools"
  cp "$GATE" "$tmp/tools/"
  cp "$FIXTURE" "$tmp/crates/card-defs/src/defs/zz_sr35_fixture.rs"
  local out rc
  out=$(cd "$tmp" && ./tools/check-defs-fmt.sh 2>&1); rc=$?
  rm -rf "$tmp"
  # A gate that never saw the fixture proves nothing either way.
  case "$out" in
    *"1 defs checked"*) ;;
    *) echo "vacuous"; return ;;
  esac
  [ "$rc" -eq 0 ] && echo green || echo red
}

# Run rustfmt --check over one file with an explicit config set; echo green|red.
probe() { # <file> <config-args...>
  local f="$1"; shift
  if rustfmt --edition 2021 --color=never --check "$@" "$f" >/dev/null 2>&1; then
    echo green
  else
    echo red
  fi
}

assert_changed() { # <file> <before-md5> — the SR track rule
  local now
  now=$(md5sum "$1" | cut -d' ' -f1)
  if [ "$now" = "$2" ]; then
    bad "perturbation did not change $1 — the attack never happened"
    return 1
  fi
  return 0
}

echo "=============================================================="
echo "A. cargo fmt is blind to the defs; the new gate is not"
echo "=============================================================="
# Baseline first. `cargo fmt --all -- --check` failing for some unrelated reason
# (a misformatted engine file, say) would otherwise read as "cargo fmt caught the
# def" and turn this attack into a false alarm — which it did, once, while this
# script was being written.
if ! "$CARGO" fmt --all -- --check >/dev/null 2>&1; then
  bad "cargo fmt --all -- --check is already RED before any perturbation — \
run 'cargo fmt --all' first; this attack cannot say anything until it is green"
else
  before=$(md5sum "$VICTIM" | cut -d' ' -f1)
  # Over-indent the card_id line: valid Rust, unambiguously misformatted.
  perl -0pi -e 's/\n        card_id:/\n                card_id:/' "$VICTIM"
  if assert_changed "$VICTIM" "$before"; then
  if "$CARGO" fmt --all -- --check >/dev/null 2>&1; then
    ok "cargo fmt --all -- --check is GREEN on a misformatted def (the SR-35 bug)"
  else
    bad "cargo fmt caught it — the premise of this task no longer holds"
  fi
  if ./$GATE >/dev/null 2>&1; then
    bad "the new gate is GREEN on a misformatted def — it is vacuous"
  else
    ok "the new gate is RED on the same def"
  fi
  fi
fi
git checkout -- "$VICTIM"

echo
echo "=============================================================="
echo "B. a long one-line oracle_text makes rustfmt blind (format_strings)"
echo "=============================================================="
# A def in the shape an author writes one: oracle_text as a single long line,
# plus a blatant misindentation on card_id.
cat > "$FIXTURE" <<'FIXTURE_EOF'
// SR-35 demo fixture.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
                card_id: cid("zz-sr35-fixture"),
        name: "ZZ SR35 Fixture".to_string(),
        types: types(&[CardType::Instant]),
        oracle_text: "Whenever a creature you control deals combat damage to a player, draw a card. Then if creatures you control have total toughness 20 or greater, untap each creature you control.".to_string(),
        ..Default::default()
    }
}
FIXTURE_EOF

if [ ! -s "$FIXTURE" ]; then
  bad "fixture was not written — the attack never happened"
else
  naive=$(probe "$FIXTURE")
  ok_flag=$(probe "$FIXTURE" --config format_strings=true)
  if [ "$naive" = "green" ]; then
    ok "WITHOUT format_strings: rustfmt is GREEN on a misindented def (silently unformatted)"
  else
    bad "WITHOUT format_strings: expected green (blind), got $naive"
  fi
  if [ "$ok_flag" = "red" ]; then
    ok "WITH format_strings=true: RED — the misindentation is caught"
  else
    bad "WITH format_strings=true: expected red, got $ok_flag"
  fi
  # And the gate as shipped must catch it.
  case "$(shipped_gate_on_fixture)" in
    red)     ok  "the shipped gate is RED on the fixture" ;;
    green)   bad "the shipped gate is GREEN on the fixture" ;;
    vacuous) bad "the shipped gate never saw the fixture — this check proves nothing" ;;
  esac
fi
rm -f "$FIXTURE"

echo
echo "=============================================================="
echo "C. the max_width skip named in the brief (error_on_line_overflow)"
echo "=============================================================="
# A line rustfmt cannot fit in 100 columns *and* cannot break — here a single
# over-long path, which has no break points. rustfmt gives up on the enclosing
# expression, keeps the file's original text (hiding the misindented card_id
# above it), and exits 0 unless error_on_line_overflow is on.
#
# The over-long path is synthetic: no real card def today has a single token that
# wide, so nothing in the corpus currently trips this. The mechanism is pinned
# anyway because it is the one SR-33 hit for real — a long `AddMana` line left 51
# files silently unformatted until someone split it by hand. rustfmt only parses
# here, so the fake type names are irrelevant to what is being demonstrated.
#
# Note this is NOT the same as a long *comment*: comments do not trigger the
# fallback (245 defs have >100-column comment lines and none are inert).
cat > "$FIXTURE" <<'FIXTURE_EOF'
// SR-35 demo fixture.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
                card_id: cid("zz-sr35-fixture"),
        name: SomeEnum::AVeryLongVariantNameThatCannotBeBrokenAnywhereAtAllBecauseItIsOneSingleIdentifierToken::Nested,
        ..Default::default()
    }
}
FIXTURE_EOF

if [ ! -s "$FIXTURE" ]; then
  bad "fixture was not written — the attack never happened"
else
  # Confirm the offending line really is over 100 columns.
  width=$(awk '{ if (length > m) m = length } END { print m }' "$FIXTURE")
  if [ "$width" -le 100 ]; then
    bad "fixture's longest line is $width cols — it does not exercise the skip"
  else
    ok "fixture has a ${width}-column line (max_width is 100)"
  fi
  naive=$(probe "$FIXTURE" --config format_strings=true)
  ok_flag=$(probe "$FIXTURE" --config format_strings=true --config error_on_line_overflow=true)
  if [ "$naive" = "green" ]; then
    ok "WITHOUT error_on_line_overflow: GREEN — and note the misindented card_id is hidden too"
  else
    bad "WITHOUT error_on_line_overflow: expected green (silent skip), got $naive"
  fi
  if [ "$ok_flag" = "red" ]; then
    ok "WITH error_on_line_overflow=true: RED — the skip is a failure, not silence"
  else
    bad "WITH error_on_line_overflow=true: expected red, got $ok_flag"
  fi
  case "$(shipped_gate_on_fixture)" in
    red)     ok  "the shipped gate is RED on the fixture" ;;
    green)   bad "the shipped gate is GREEN on the fixture" ;;
    vacuous) bad "the shipped gate never saw the fixture — this check proves nothing" ;;
  esac
fi
rm -f "$FIXTURE"

echo
echo "=============================================================="
echo "D. the gate will not pass vacuously with nothing to check"
echo "=============================================================="
tmp=$(mktemp -d)
mkdir -p "$tmp/crates/card-defs/src/defs" "$tmp/tools"
cp "$GATE" "$tmp/tools/"
( cd "$tmp" && ./tools/check-defs-fmt.sh >/dev/null 2>&1 )
rc=$?
if [ "$rc" -eq 2 ]; then
  ok "empty corpus exits 2 rather than reporting success"
else
  bad "empty corpus exited $rc — expected 2"
fi
rm -rf "$tmp"

echo
echo "=============================================================="
if [ "$fails" -eq 0 ]; then
  echo "ALL ATTACKS BEHAVED AS EXPECTED"
else
  echo "$fails CHECK(S) FAILED"
fi
echo "=============================================================="
exit "$fails"
