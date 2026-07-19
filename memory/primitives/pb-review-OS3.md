# Primitive Batch Review: PB-OS3 — WhenTappedForMana trigger target dispatch (OOS-EF6-1)

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 605.1a/605.1b (mana-ability criteria), 605.5a (targeted trigger is NOT a mana ability → stack), 603.3d (auto-target / fizzle), 106.12a (WhenTappedForMana semantics)
**Engine files reviewed**: `crates/engine/src/rules/mana.rs` (the one-line kind reclassify + comment), `crates/engine/src/rules/abilities.rs` (dispatch chain: `has_ability_targets`, target resolution, auto-picker, stack build, doubler), `crates/engine/src/rules/resolution.rs` (CardDefETB effect resolution + EffectContext build), `crates/engine/src/state/hash.rs` (arm 46 unchanged)
**Card defs reviewed**: `crates/card-defs/src/defs/forbidden_orchard.rs` (1 def, `known_wrong` → `Complete`)
**Tests reviewed**: `crates/engine/tests/rules/mana_triggers.rs` (1 updated + 3 new)

## Verdict: clean

Option B (reclassify the queued `PendingTriggerKind` from `Normal` to `CardDefETB` on the
stack-push branch of `fire_mana_triggered_abilities`) is correct end-to-end and verified against
source and CR, not merely against the runner's summary. The full dispatch chain
(queue → `flush_pending_triggers` target resolution → auto-pick `TargetOpponent` → stack-object
target → CardDefETB resolution → `CreateToken` recipient binding) resolves the declared
`TargetOpponent` to the first active opponent and routes the Spirit token there. `CardDefETB` is
confirmed to be a pure raw-`def.abilities`-index marker (already used for many non-ETB triggers via
PB-EF3 A2 — `abilities.rs:4884-4885`), and no consumer applies ETB-specific semantics
(replacement, enters-tapped, doubling) based on the kind. `compute_trigger_doubling` /
`doubler_applies_to_trigger` (`abilities.rs:8274-8349`) key strictly on `trigger.triggering_event`,
which is `None` for the blank-constructed mana trigger, so no Panharmonicon/Yarok/CreatureDeath
doubler can misfire. The immediate-mana branch (`mana.rs:698-707`) is untouched, so genuine
triggered mana abilities still resolve without the stack. `forbidden_orchard.rs` now matches oracle
text exactly (1/1 colorless Spirit to the targeted opponent), has no residual TODO/approximation,
and the `Complete` flip is justified (both halves compose, proven by the 4-player decoy test).
No new enum variant/field/DSL type — `HASH_SCHEMA_VERSION` stays 55, `PROTOCOL_VERSION` stays 18;
hash arm 46 pre-exists (`hash.rs:2785`). No PROTOCOL/HASH bump warranted. Zero findings at
HIGH/MEDIUM/LOW. One forward-looking INFORMATIONAL note recorded below.

## Engine Change Findings

None.

## Card Definition Findings

None.

## Detailed Verification (adversarial focus areas)

### 1. CardDefETB semantic safety (focus 1)
- **has_ability_targets** (`abilities.rs:6876-6900`): CardDefETB arm reads
  `def.abilities.get(trigger.ability_index)` → sees `targets = [TargetOpponent]` → `true`. Correct raw-index space (the mana loop iterates `def.abilities.iter().enumerate()`, so `ability_idx` is the raw def index).
- **Target resolution** (`abilities.rs:6979-7025`): CardDefETB branch again uses
  `def.abilities.get(trigger.ability_index)` (`:7006-7020`), returns `[TargetOpponent]`; the Normal branch (which reads the runtime `characteristics.triggered_abilities` vec — empty for WhenTappedForMana, no enrich block) is explicitly bypassed. This is the exact bug that reclassification fixes.
- **Auto-picker** (`abilities.rs:7076-7091`): `TargetOpponent` picks the first `turn_order` player `!= controller` that is not lost/conceded; contributes `None` (trigger removed per CR 603.3d) if no opponent — never falls back to controller. Correct.
- **Stack-object build** (`abilities.rs:8107-8125`): Normal and CardDefETB build the identical `StackObjectKind::TriggeredAbility`; only `is_carddef_etb` (false→true) and `embedded_effect` (clone→None) differ. The mana path's `PendingTrigger::blank` already carries `embedded_effect = None`, so nothing is lost.
- **Resolution** (`resolution.rs:1978-2003`): CardDefETB path resolves the effect via `def.abilities.get(ability_index)` → `CreateToken`. EffectContext is built with `stack_obj.targets.clone()` (`:2095-2100`), so `TokenSpec.recipient: DeclaredTarget{0}` binds to `Target::Player(p2)`. No ETB replacement semantics are applied on this path — it is a plain effect execution.
- **No ETB misfire**: grep of every `CardDefETB` / `is_carddef_etb` consumer (mana.rs, resolution.rs, abilities.rs, hash.rs, turn_actions.rs, replacement.rs) shows the kind is used only as an index-space + registry-lookup marker; no site branches into "enters the battlefield" replacement/tapped/counter logic based on it. ETB replacements are event-driven, not kind-driven. Safe.

### 2. CR compliance (focus 2)
- CR 605.5a (verified via MCP): "An ability with a target is not a mana ability … These follow the normal rules for … triggered abilities." Forbidden Orchard's trigger declares `TargetOpponent` → correctly goes on the stack (the engine already did this; only dispatch was broken). Confirmed the immediate-mana branch is untouched (`mana.rs:698-707`), so CR 605.1b/605.4a mana abilities still resolve without the stack.
- CR 603.3d (verified via MCP): auto-picker removes the trigger if no legal opponent. Matches `abilities.rs:7076-7091`.

### 3. forbidden_orchard oracle fidelity (focus 3)
- Oracle (MCP): "{T}: Add one mana of any color. / Whenever you tap this land for mana, target opponent creates a 1/1 colorless Spirit creature token." Def now: `AddManaAnyColor` mana ability + `WhenTappedForMana` trigger with `targets: [TargetOpponent]` and `TokenSpec { name: "Spirit", power: 1, toughness: 1, colors: empty (colorless), card_types: [Creature], subtypes: [Spirit], count: 1, recipient: DeclaredTarget{0} }`. Matches oracle exactly (colorless = empty `OrdSet`, 1/1, Spirit, targeted opponent).
- `Complete` flip justified: the any-colour half was made real-colour-eligible by PB-EF12 (test asserts `white == 1` after `chosen_color: White`); the token half now routes to the targeted opponent. Both halves compose in a live 4-player game (Test 13). No residual stub; the `AddManaAnyColor` half is a genuinely-resolving mana ability (not a gated/barred stub). No remaining TODO in the file; the header comment was rewritten to describe actual (resolved) behaviour, satisfying the conventions "no aspirational comments" rule.

### 4. Test quality (focus 4)
- **Test 13 (4-player decoy compose)** proves recipient == declared opponent, not just the end effect: it captures the stack object BEFORE resolution and asserts `stack_obj.targets[0].target == Target::Player(p2)` (`:1157-1162`), then asserts exactly one Spirit controlled by p2 (`:1178-1182`), then asserts p1 (controller), p3, p4 each control ZERO Spirits (`:1184-1202`). This distinguishes the declared target from (a) controller-fallback, (b) EachOpponent (would give all three opponents a token), and (c) random opponent. `four_player()` builds turn_order `[p1..p4]`, p1 active → deterministic p2 pick, matching the explicit assertion.
- **Non-vacuity**: the runner reverted Change 1 to `Normal` and confirmed both forbidden_orchard tests fail, then restored. Independently credible: under `Normal`, target resolution reads the empty runtime vec → `stack_obj.targets` empty → the p2-target and p2-controller assertions both fail (the kind assertion also fails, but the recipient assertions are the load-bearing ones).
- **Test 14 (no-regression)** is meaningful: Wild Growth (empty-target, mana-producing) routes through the untouched immediate branch; asserts green == 2 AND `pending_triggers().is_empty()` — proving the kind change does not divert an untargeted doubler onto the stack.
- **Test 10 (updated)** correctly pins `kind == CardDefETB`, `ability_index == 1`, and strengthens the recipient assertion to p2 in the 2-player case.

### 5. Roster completeness (focus 5, SR-34/36)
- **Test 15** enumerates `all_cards()` (not grep) for every `TriggerCondition::WhenTappedForMana` ability and partitions by `targets.is_empty()`. Asserts the targeted set is exactly `["Forbidden Orchard"]` and the untargeted set is exactly the 6 doublers/adders (Badgermole Cub, Crypt Ghast, Leyline of Abundance, Mirari's Wake, Wild Growth, Zendikar Resurgent). This is the authoritative proof that Forbidden Orchard is the sole card reaching the reclassified branch. The hardcoded 6-card list is brittle-by-design (a new untargeted WhenTappedForMana card would break it), which correctly forces re-review — a feature, not a defect.
- Even a hypothetical empty-target NON-mana WhenTappedForMana card would resolve identically under Normal vs CardDefETB (empty targets → `Some(vec![])` either way; effect via registry either way, since no enrich block populates the runtime vec), so the kind change is provably inert for every untargeted case regardless of the mana-producing property. No regression path exists.

### 6. Wire invariant (focus 6)
- No new enum variant, field, or DSL type. `CardDefETB` and its hash arm (46, `hash.rs:2785`) pre-exist. Sentinels `pb_ef7_modal_activated::test_ef7_hash_and_protocol_versions` and `pb_ef12_any_color_choice::test_ef12_protocol_version_sentinel` pin PROTOCOL 18 / HASH 55 and remain green. **No bump warranted.** (Confirmed NOT a HIGH finding.)

## INFORMATIONAL (forward-looking, not a current defect)

- **INFO-1** — `resolution.rs` CardDefETB path does not propagate `mana_produced` into `EffectContext` (only the immediate-mana branch at `mana.rs:703` sets `ctx.mana_produced`). Forbidden Orchard's `CreateToken` does not read it, so there is no current bug. However, a FUTURE *targeted* WhenTappedForMana card whose stacked effect references the mana that was produced (e.g. a reflexive "add one mana of any type that land produced" gated behind a target) would silently receive no `mana_produced` on this path. Test 15's roster assertion is the guard: any such new card would break the hardcoded targeted-set assertion and force re-review, at which point this propagation gap should be addressed. No action required for PB-OS3.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 605.5a (targeted trigger not a mana ability → stack) | Yes | Yes | Tests 10, 13 (kind=CardDefETB, queued not immediate) |
| 605.1b/605.4a (immediate mana ability path untouched) | Yes (unchanged) | Yes | Test 14 (Wild Growth immediate, no PendingTrigger) |
| 603.3d (auto-target opponent) | Yes | Yes | Test 13 asserts stack target == Player(p2); no-opponent fizzle covered by existing PB-EF6 auto-picker tests |
| 106.12a (WhenTappedForMana firing incl. dead source) | Yes (pre-existing SR-28) | Yes | Tests 11/12 unaffected |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| forbidden_orchard | Yes | 0 | Yes | 1/1 colorless Spirit → targeted opponent; both halves compose; `Complete` justified |
