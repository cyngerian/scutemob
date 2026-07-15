#!/usr/bin/env bash
# SR-20 adversarial demo — prove the two new guarantees in the registry gates
# (tests/core/{keyword_registry,ability_definition_registry}.rs) actually fire.
# Per the SR-track rule, each attack asserts it changed a file first, then the
# gate is run and must fail on exactly the intended test.
#
# Files are restored from an in-memory backup (NOT `git checkout`), so the demo is
# safe to run whether or not the SR-20 changes are committed yet.
#
# Run from the workspace root: bash crates/engine/tests/sr20_adversarial_demo.sh
set -u

CARGO="$HOME/.cargo/bin/cargo"
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT" || exit 1

COMBAT="crates/engine/src/rules/combat.rs"
KWTEST="crates/engine/tests/core/keyword_registry.rs"

pass=0
fail=0
note() { printf '\n=== %s ===\n' "$1"; }
backup() { cp "$1" "$1.sr20bak"; }
restore() { mv "$1.sr20bak" "$1"; }
# run one registry test module member; succeed iff it FAILS (attack must be caught)
expect_caught() { # module::test label
  if $CARGO test -p mtg-engine --test core "$1" -- --exact 2>&1 \
       | grep -qE "FAILED|panicked|error\["; then
    echo "  [CAUGHT] $2 -> $1 failed as intended"; pass=$((pass+1))
  else
    echo "  [SURVIVED] $2 -> $1 did NOT fail (BAD)"; fail=$((fail+1))
  fi
}
assert_changed() { # path backup
  if cmp -s "$1" "$2"; then
    echo "  ATTACK NO-OP: $1 unchanged (attack proves nothing)"; fail=$((fail+1))
  else
    echo "  attack changed a file: $1"
  fi
}

# --- Attack 1 + 2: aliased-import dispatch blinds the literal-token scanner ------
# Append real, compiling dispatch that reaches a variant through an *alias*, so the
# `KeywordAbility::`/`AbilityDefinition::` scanner never sees it. AD::Vanishing is a
# *Marker* — the exact "Marker silently gains dispatch" failure SR-5/SR-15 exist for.
note "Attacks 1 & 2: aliased-import dispatch (KA::Equip, AD::Vanishing) in combat.rs"
backup "$COMBAT"
cat >> "$COMBAT" <<'RS'

#[allow(dead_code)]
fn sr20_demo_keyword_alias(kw: &crate::state::types::KeywordAbility) -> bool {
    use crate::state::types::KeywordAbility as KA;
    matches!(kw, KA::Equip)
}

#[allow(dead_code)]
fn sr20_demo_ability_alias(a: &crate::cards::card_definition::AbilityDefinition) -> bool {
    use crate::cards::card_definition::AbilityDefinition as AD;
    matches!(a, AD::Vanishing { .. })
}
RS
assert_changed "$COMBAT" "$COMBAT.sr20bak"
expect_caught keyword_registry::use_imports_do_not_bypass_the_scanner "KA::Equip alias"
expect_caught ability_definition_registry::use_imports_do_not_bypass_the_scanner "AD::Vanishing alias"
restore "$COMBAT"

# --- Attack 3: without the simulator scan root the declared sim sites are unseen --
# The five keyword (and six ability) legal_actions.rs sites are declared in the
# registry. Remove crates/simulator/src from SCAN_ROOTS and the source tree no longer
# shows them, so the declarations become lies -> the anti-rot check fails. This is
# "simulator dispatch is invisible before the scan-root add" made concrete.
note "Attack 3: drop crates/simulator/src scan root -> declared sim sites unsupported"
backup "$KWTEST"
perl -0pi -e 's{"crates/card-types/src",\n    "crates/simulator/src",\n}{"crates/card-types/src",\n}' "$KWTEST"
assert_changed "$KWTEST" "$KWTEST.sr20bak"
expect_caught keyword_registry::registry_sites_match_the_source_tree "no simulator scan root"
restore "$KWTEST"

note "SR-20 demo result: $pass caught / $fail problems"
[ "$fail" -eq 0 ] && echo "ALL ATTACKS CAUGHT" || echo "SOME ATTACKS SURVIVED"
exit "$fail"
