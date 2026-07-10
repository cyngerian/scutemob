#!/usr/bin/env bash
# SR-9c adversarial demonstration.
#
# Each attack perturbs the corpus (or a harness assertion) in a way that SHOULD be
# caught, then asserts that exactly the intended gate goes red. Per SR-9a/SR-9b
# convention every attack first asserts it actually changed the file — a "passing"
# attack that changed nothing is the failure mode that has bitten every prior SR task.
#
# Run from the workspace root:  bash crates/engine/tests/scripts/adversarial_demo.sh
# It mutates tracked files and restores them with `git checkout`, so commit first.
set -u
CARGO="$HOME/.cargo/bin/cargo"
SD=test-data/generated-scripts
fails=0

# Run one named accounting test. Echoes: green | red | absent.
run_test() { # <test-name>
  local out
  out=$("$CARGO" test -p mtg-engine --test scripts "run_all_scripts::$1" -- --exact 2>&1)
  if   echo "$out" | grep -q "test result: ok. 1 passed";  then echo green
  elif echo "$out" | grep -q "test result: FAILED. 0 passed; 1 failed"; then echo red
  else echo absent; fi
}
# run_all replays the whole approved corpus.
run_all() {
  local out
  out=$("$CARGO" test -p mtg-engine --test scripts run_all_scripts::run_all_approved_scripts -- --exact 2>&1)
  if echo "$out" | grep -q "test result: ok. 1 passed"; then echo green; else echo red; fi
}

expect_red() { # <description> <changed-file> <runner-fn>
  local desc="$1" file="$2" runner="$3"
  if git diff --quiet -- "$file"; then
    echo "  ✗ ATTACK NO-OP: $desc — $file unchanged; attack proves nothing"; fails=$((fails+1)); return
  fi
  local verdict; verdict=$("$runner" "${4:-}")
  git checkout -q -- "$file"
  case "$verdict" in
    red)    echo "  ✓ caught:   $desc" ;;
    green)  echo "  ✗ SURVIVED: $desc — the gate stayed green"; fails=$((fails+1)) ;;
    absent) echo "  ✗ NO TARGET: $desc — the named test did not run"; fails=$((fails+1)) ;;
  esac
}

echo "SR-9c adversarial demonstration"

# 1. A script left pending_review — the silent-skip this task exists to kill.
f="$SD/baseline/003_tap_land_for_mana.json"
python3 -c "import json;p='$f';d=json.load(open(p));d['metadata']['review_status']='pending_review';json.dump(d,open(p,'w'),indent=2)"
expect_red "a script sits at pending_review" "$f" run_test no_script_is_awaiting_triage

# 2. A JSON that no longer deserializes — the six invisible files' failure mode.
f="$SD/baseline/002_play_basic_land.json"
printf '{ this is not json ' > "$f"
expect_red "a corpus file fails to deserialize" "$f" run_test every_script_file_deserializes

# 3. A retired script with no recorded reason.
f="$SD/baseline/006_sol_ring_mana_ability.json"
python3 -c "import json;p='$f';d=json.load(open(p));d['metadata']['review_status']='retired';d['metadata'].pop('retirement_reason',None);json.dump(d,open(p,'w'),indent=2)"
expect_red "a retired script omits its reason" "$f" run_test retired_scripts_carry_a_reason

# 4. An approved script that asserts nothing.
f="$SD/baseline/007_night_whisper_draw_two.json"
python3 -c "import json;p='$f';d=json.load(open(p));[s['actions'].__setitem__(slice(None),[a for a in s['actions'] if a['type']!='assert_state']) for s in d['script']];json.dump(d,open(p,'w'),indent=2)"
expect_red "an approved script has zero assertions" "$f" run_test every_approved_script_asserts_something

# 5. An approved script using an un-allowlisted untranslatable action.
f="$SD/baseline/008_lightning_bolt_creature.json"
python3 -c "
import json;p='$f';d=json.load(open(p))
for s in d['script']:
    for a in s['actions']:
        if a['type']=='player_action': a['action']='order_replacements'; raise SystemExit(json.dump(d,open(p,'w'),indent=2) or 0)
"
expect_red "an approved script runs an untranslatable action" "$f" run_test approved_scripts_only_use_allowlisted_untranslatable_actions

# 6. An unknown assertion path — the 244-assertion vacuity hole. Must be a hard mismatch.
f="$SD/baseline/009_read_the_bones_scry_draw.json"
python3 -c "
import json;p='$f';d=json.load(open(p))
for s in d['script']:
    for a in s['actions']:
        if a['type']=='assert_state': a['assertions']['zones.nonsense.path']=1; raise SystemExit(json.dump(d,open(p,'w'),indent=2) or 0)
"
expect_red "an approved script asserts an unimplemented path" "$f" run_all

# 7. The zones.stack vacuity fix: re-assert an empty stack where a trigger sits.
#    stack/050 correctly expects the Solemn dies trigger (count 1); claim empty instead.
f="$SD/stack/050_wrath_kills_multiple_creatures.json"
python3 -c "
import json;p='$f';d=json.load(open(p))
for s in d['script']:
    for a in s['actions']:
        if a['type']=='assert_state' and 'zones.stack.count' in a.get('assertions',{}):
            del a['assertions']['zones.stack.count']; a['assertions']['zones.stack']={'is_empty':True}
            raise SystemExit(json.dump(d,open(p,'w'),indent=2) or 0)
"
expect_red "zones.stack is_empty is actually checked against the stack" "$f" run_all

echo
if [ "$fails" -eq 0 ]; then echo "all attacks caught"; else echo "$fails attack(s) not caught"; exit 1; fi
