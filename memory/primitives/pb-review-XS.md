# Primitive Batch Review: PB-XS — `TargetFilter.exclude_self` for "another target X" spell/ability target selection

**Date**: 2026-05-14
**Reviewer**: primitive-impl-reviewer (Opus)
**Branch**: feat/pb-xs-targetfilterexcludeself-for-another-target-x-spellabil
**Commit**: 9b679fb8 (worktree scutemob-21)
**CR Rules**: 109.1, 601.2c, 608.2b, 400.7, 603.10a
**Engine files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (field + doc)
- `crates/engine/src/state/hash.rs` (HASH_SCHEMA_VERSION 18→19, hash arm)
- `crates/engine/src/rules/casting.rs` (`validate_object_satisfies_requirement` 4 arms + `validate_targets` dead-code retention)
- `crates/engine/src/rules/abilities.rs` (activated-ability call site at ~344; trigger auto-target picker — 6 hot sites)

**Card defs reviewed** (9 total): roalesk_apex_hybrid, samut_voice_of_dissent, torch_courier, brash_taunter, ezuri_renegade_leader, oath_of_teferi, elderfang_ritualist, dour_port_mage, thousand_faced_shadow.

**Tests reviewed**: `crates/engine/tests/primitive_pb_xs.rs` (9 tests).

---

## Verdict: needs-fix

The engine change is mechanically correct and well-scoped. The new `exclude_self: bool`
field is dispatched at all 4 declarative target-validation arms in casting.rs and at all 6
trigger auto-target picker arms in abilities.rs. Hash is bumped, sentinel-swept, and arm
is added. Card defs match oracle text for the 9 cards in scope. CR research is sound.

The verdict is **needs-fix** rather than clean because of **one test-validity HIGH**
(`test_pbxs_etb_auto_target_picker_skips_source` is a tautology — its body does not
exercise the trigger auto-target picker it claims to test). Per `memory/conventions.md`
the "test-validity MEDIUMs are fix-phase HIGHs" rule, this surfaces as HIGH on the
initial review pass.

There are no engine-correctness HIGHs, no oracle-text mismatches, and no
silent-drop-exclude_self holes at target legality sites. The "trigger source resolves
to post-death graveyard ObjectId" concern from the dispatch brief is verified correct.
The "re-validation at resolution doesn't check exclude_self" observation is also
verified correct per CR 608.2b — only zone identity needs re-checking on resolution,
and that is unchanged by PB-XS.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | **HIGH** | `crates/engine/tests/primitive_pb_xs.rs:484` | **`test_pbxs_etb_auto_target_picker_skips_source` is a tautology.** Its body never fires an ETB trigger; it asserts that a "synthetic placement" left zero counters on MiniRoalesk. That assertion is true regardless of `exclude_self`. The 6 trigger auto-target sites in abilities.rs (6627, 6641, 6756, 6780, 6833, 6846) have **zero direct test coverage**. Per `conventions.md` "test-validity MEDIUMs are fix-phase HIGHs." **Fix:** rewrite this test to actually fire an ETB trigger and verify the auto-target picker rejects the entering object as a candidate (e.g. by enqueuing a CreateToken or AddCounter command sequence that triggers the WhenEntersBattlefield path, then asserting the resulting StackObject for the trigger has zero targets or no counters land on MiniRoalesk). Alternatively, split into two tests: one for the battlefield-scan trigger arm (TargetCreatureWithFilter / TargetPermanentWithFilter) and one for the graveyard-scan arm (TargetCardInYourGraveyard / TargetCardInGraveyard via Elderfang Ritualist's death trigger). |
| E2 | LOW | `crates/engine/src/rules/casting.rs:5319` | **`validate_targets` retained behind `#[allow(dead_code)]` with no callers.** The doc comment says "retained for callers that genuinely cannot supply a source ObjectId (none exist today)." Standing rule for dead code retention is "delete or document why retention is load-bearing." The function's only purpose is forwarding to `validate_targets_inner` with `self_id = None`. Removing it loses no functionality (callers can use `validate_targets_with_source` and pass a sentinel — but that breaks the type-system guarantee that self_id is `Option<ObjectId>`). **Fix:** either (a) delete `validate_targets` (no callers, easy to add back), or (b) replace `#[allow(dead_code)]` with `#[cfg(test)]` if test-only future use is envisioned, or (c) keep it but add a test that calls it. Recommendation: delete; if a future caller needs `self_id = None`, it can call `validate_targets_inner` directly via re-export. |
| E3 | LOW | `crates/engine/tests/{primitive_pb_ts,pbt_up_to_n_targets,primitive_pb_lki_cc,primitive_pb_cc_a,pbd_damaged_player_filter,primitive_pb_cc_c_followup,pbn_subtype_filtered_triggers,primitive_pb_lki_power,pbp_power_of_sacrificed_creature,primitive_pb_ewc}.rs` (~10 sites) | **Stale sentinel error messages reference PB-LKI-CC bump 14→15 (or similar), not the current PB-XS bump 18→19.** Examples: `pbt_up_to_n_targets.rs:412` says `"PB-LKI-CC: HASH_SCHEMA_VERSION must be 15"` while the sentinel value is now `19u8`. Per `conventions.md` "Aspirationally-wrong code comments are correctness hazards" — the assertion value is right but the message string is wrong. A future failure here will print a message pointing at the wrong PB. PB-XS updated `primitive_pb_ts.rs:370` correctly (cites PB-XS) but missed the other ~9 sites. **Fix:** sweep all sentinel assertion error messages and update to cite the current PB (PB-XS) and value (18→19). PB-EWC's message at `primitive_pb_ewc.rs:396` still says "must be 18". |
| E4 | LOW | `crates/engine/src/rules/resolution.rs:7461` | **`is_target_legal` at resolution only checks zone identity, not filter/controller/exclude_self.** Per CR 608.2b: "Other changes to the game state may cause a target to no longer be legal; for example, its characteristics may have changed..." A creature targeted by Brash Taunter's "{2}{R}, {T}: This creature fights another target creature" that gains hexproof (or somehow becomes the source via control-change) between cast and resolution would not be re-rejected. **This is a pre-existing gap, not a PB-XS regression** — but PB-XS is the first primitive whose declarative legality is filter-driven and would benefit from a full re-check. PB-XS does NOT need to fix this (per the dispatch brief), but the gap should be tracked. **Fix:** file as OOS seed in `pb-retriage-CC.md` documenting the re-validation gap for filter/controller/exclude_self. Reference CR 608.2b. Yield: probably zero in practice because hexproof gain mid-resolution is rare and Brash Taunter's source-vs-target identity doesn't change. Mark as deferred. |
| E5 | LOW | `crates/engine/src/cards/card_definition.rs:2649` (field placement) | **`exclude_self` is the 26th field on `TargetFilter` and lives at the end of the struct, but the doc comment block is the longest in the file (~20 lines).** Future maintainers may not see the "NOT checked inside matches_filter" warning unless they scroll past the entire field list. **Fix:** consider grouping the four "runtime-relationship" fields (`is_token`, `is_attacking`, `has_chosen_subtype`, `exclude_chosen_subtype`, `has_counter_type`, `exclude_self`) into a separate substruct or at minimum add a `// === RUNTIME-RELATIONSHIP FIELDS (matches_filter ignores these) ===` comment banner above the group. Pure style — defer unless reviewer/runner has bandwidth. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | LOW | `thousand_faced_shadow.rs` | **TODO comment cites OOS-XS-2 (is_attacking enforcement gap), but the card's auto-target picker will silently pick non-attacking creatures.** The card def correctly sets `is_attacking: true` and `exclude_self: true` on the TargetFilter, but per OOS-XS-2 the `is_attacking` field is silently ignored by `validate_object_satisfies_requirement` and the trigger auto-target picker. Net effect: the card ships with a known semantic gap — the ETB trigger may target a non-attacking creature instead of the intended attacking one. PB-XS does NOT need to fix this (OOS-XS-2 is filed), but the card def's game state is partially wrong even after PB-XS. **Fix:** consider reverting to TODO until OOS-XS-2 lands, or accept the partial fix with a stronger inline comment block flagging the wrong-game-state risk. Recommendation: keep as-is since this is the closest the card can get without an additional primitive, but elevate the TODO note in the source comment. The header TODO comment (lines 6-9) does NOT call out the is_attacking gap — only the ETB-trigger-from-hand gap. Add a second TODO line. |
| C2 | LOW | `dour_port_mage.rs` | **First ability ("Whenever one or more other creatures you control leave the battlefield without dying") is TODO'd, second ability ships.** Per CR 603.2 the "without dying" qualifier is a known DSL gap (no compound LBA-not-Dies trigger). The header comment at line 6 explicitly cites this. **Fix:** no action — this is consistent with established workstream policy ("No card is authored until its required primitives exist. No TODOs..."). PB-XS only widens the second-ability portion. If the strict "no TODO" policy applies, revert the card to TODO until both abilities can ship. Recommendation: keep as-is — the existing TODO is from before PB-XS and is not the responsibility of this PB. Just verify the second ability is correctly authored (it is). |
| C3 | LOW | `roalesk_apex_hybrid.rs` | **Second ability ("When Roalesk dies, proliferate, then proliferate again") TODO'd.** Comment cites "WhenThisDies trigger + Effect::Proliferate (twice)". **Fix:** no action — pre-existing TODO. PB-XS scope is the ETB half. |
| C4 | LOW | `oath_of_teferi.rs` | **Second ability ("activate loyalty abilities twice each turn") TODO'd.** Comment cites "no Permission for this." **Fix:** no action — pre-existing TODO. PB-XS scope is the ETB half. |
| C5 | LOW | `brash_taunter.rs` | **First ability ("Whenever this creature is dealt damage, it deals that much damage to target opponent") TODO'd.** Comment cites "targeted_trigger with a damage-amount variable — not in DSL." **Fix:** no action — pre-existing TODO. PB-XS scope is the activated fight half. |

### Finding Details

#### Finding E1: `test_pbxs_etb_auto_target_picker_skips_source` is a tautology

**Severity**: HIGH (per `memory/conventions.md` test-validity rule)
**File**: `crates/engine/tests/primitive_pb_xs.rs:484-562`
**CR Rule**: 109.1 / 601.2c / 603.3d (auto-target selection for triggered abilities)
**Issue**: The test name and module-level docstring promise that this test exercises the
trigger auto-target picker (`abilities.rs` lines 6627, 6641, 6756, 6780, 6833, 6846 — six
sites added by PB-XS). The test body, however, does not enqueue or fire any ETB trigger.
It synthetically places "MiniRoalesk" in `ZoneId::Battlefield` via `ObjectSpec::card(...).in_zone(ZoneId::Battlefield)`,
which the test's own comment block (lines 522-528) acknowledges does NOT trigger
`emit_etb_triggered_effects`. The final assertion is `counters == 0` on MiniRoalesk —
which is **necessarily true** because the trigger never fired, regardless of whether
`exclude_self` is wired correctly or not.

Concretely: this test would PASS against an engine where the entire `exclude_self`
auto-target picker code was deleted. That makes it the exact silent-skip failure mode
described in `conventions.md`:

> "if the test title says 'pre-death LKI' and the setup can't discriminate pre- vs
> post-death evaluation, that is a test-validity bug with the same urgency as a
> wrong-game-state bug."

The substantive impact: the 6 trigger auto-target picker sites in abilities.rs have zero
direct test coverage. The 4 declarative-validation sites in casting.rs are well-covered
by tests C-1, C-2, D-1, E-1. The two halves of the primitive are unbalanced.

**Fix**: rewrite this test to actually exercise the trigger auto-target picker.
Two recommended approaches:

1. **Direct ETB trigger fire**: have MiniRoalesk enter via the cast/resolution pipeline
   (or use `cards/registry`-driven enrichment + a `Command::CastSpell` path to land it),
   then verify the resulting PendingTrigger has no legal target candidate when MiniRoalesk
   is the only creature on P1's side. Assert the trigger resolves with no effect (no
   counters land anywhere) and produces a "skipped (no legal target)" log/event if any.

2. **Death-trigger graveyard auto-target test**: build an Elderfang-Ritualist-shaped def,
   put it on the battlefield, kill it (via Command::Sacrifice or assigned damage), and
   verify the WhenDies trigger auto-targets a SECOND Elf card in the graveyard and NOT
   the post-death Ritualist itself. This directly exercises lines 6627/6641 (the
   graveyard arm) which are the most subtle CR 400.7 / CR 603.10a interaction.

Either approach is acceptable; #2 is the better stress test because it discriminates
PRE-death (battlefield) vs POST-death (graveyard) ObjectId for `trigger.source`.

#### Finding E2: `validate_targets` retained as dead code

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:5310-5327`
**Issue**: After PB-XS migrated all in-tree callers to `validate_targets_with_source`,
the older `validate_targets` wrapper has zero callers. It is kept behind
`#[allow(dead_code)]` with a doc comment claiming "retained for callers that genuinely
cannot supply a source ObjectId (none exist today)." This is speculative — no future
caller is named, and the alternative (calling `validate_targets_inner` with
`self_id = None`) is one line longer.

The standing rule in `conventions.md` is to delete dead code or document why retention
is load-bearing. "Maybe someone will need it" doesn't qualify.

**Fix**: pick one:
- **Delete**: simplest. Re-add if a real need surfaces.
- **Make `pub(crate)` only with test usage**: requires writing a test that calls it.
- **Document a concrete future caller**: replace the speculative doc with a TODO citing
  the specific case (e.g. "future SBA-driven target re-validation at resolution will
  need a sourceless variant — see TODO X").

Recommendation: delete.

#### Finding E3: Stale sentinel error messages

**Severity**: LOW
**Files**: ~10 sentinel test files; only `primitive_pb_xs.rs:67-70` and `primitive_pb_ts.rs:370` are current.
**Issue**: The sentinel test pattern asserts `HASH_SCHEMA_VERSION == 19u8` (correct) but
the failure message text in most files still cites old PB references and old version
numbers. Examples:

- `pbt_up_to_n_targets.rs:412`: `"PB-LKI-CC: HASH_SCHEMA_VERSION must be 15 (bump from PB-TS's 14)..."`
- `pbp_power_of_sacrificed_creature.rs:783`: `"PB-LKI-CC bumped HASH_SCHEMA_VERSION 14→15..."`
- `primitive_pb_lki_cc.rs:441`: `"PB-LKI-CC bumped HASH_SCHEMA_VERSION 14→15..."`
- `primitive_pb_cc_a.rs:102`: `"PB-LKI-CC bumped HASH_SCHEMA_VERSION 14→15..."`
- `pbd_damaged_player_filter.rs:598`: `"PB-LKI-CC bumped HASH_SCHEMA_VERSION 14→15..."`
- `primitive_pb_cc_c_followup.rs:401`: `"PB-LKI-CC bumped HASH_SCHEMA_VERSION 14→15..."`
- `pbn_subtype_filtered_triggers.rs:559`: `"PB-LKI-CC bumped HASH_SCHEMA_VERSION 14→15..."`
- `primitive_pb_lki_power.rs:386`: `"PB-LKI-Power bumped HASH_SCHEMA_VERSION 16→17..."`
- `primitive_pb_ewc.rs:396`: `"PB-EWC: HASH_SCHEMA_VERSION must be 18..."`
- `effect_sacrifice_permanents_filter.rs:137`: PB-XS updated this one correctly.

Per `conventions.md` "Aspirationally-wrong code comments are correctness hazards" —
when this assertion fires on a future bump, the failure message will mislead the
reader about which PB last touched the version. Minor but cumulative.

**Fix**: sweep all sentinel-test error messages and update to cite "PB-XS bumped
HASH_SCHEMA_VERSION 18→19 (TargetFilter.exclude_self, CR 109.1 / 601.2c)". OR establish
a convention that the failure message points at the **current** PB (not the PB that
last edited that test file). Either is fine; current state is neither.

#### Finding E4: `is_target_legal` at resolution doesn't re-check filter/controller/exclude_self

**Severity**: LOW (pre-existing, not introduced by PB-XS)
**File**: `crates/engine/src/rules/resolution.rs:7461-7477`
**CR Rule**: 608.2b
**Issue**: `is_target_legal` checks only that the target object is in the same zone it
was when targeted. CR 608.2b also requires re-checking that characteristics still
satisfy the target requirement (filter), and by extension `exclude_self`. The dispatch
brief's question 3 acknowledges this is correct per CR 608.2b's primary clause ("A
target that's no longer in the zone it was in when it was targeted is illegal") but
asks if the reviewer disagrees.

I partially disagree. CR 608.2b says:
> "Other changes to the game state may cause a target to no longer be legal; for
> example, its characteristics may have changed or an effect may have changed the text
> of the spell."

A creature could legally be targeted by Brash Taunter's fight ability and then, between
target declaration and resolution, gain hexproof or become the source of the activating
spell (Cytoshape-style control change). The engine wouldn't catch this. **However**,
this is not a PB-XS regression — the pre-existing zone-only check is the same as before
PB-XS, and the exclude_self filter is a target-DECLARATION constraint (CR 601.2c),
which is correctly enforced at announcement time. CR 608.2b re-checks ZONE most rigorously
because that's what changes most often.

The reviewer's verdict: **PB-XS is correct as shipped**. Re-validation of full filter
at resolution is a pre-existing engine gap that should be tracked separately.

**Fix**: file an OOS seed in `pb-retriage-CC.md` documenting the resolution-time
re-validation gap. Cite CR 608.2b. Estimated yield: 0–1 corner-case cards. Defer.

#### Finding C1: Thousand-Faced Shadow ships with is_attacking gap

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/thousand_faced_shadow.rs:43-52`
**Oracle**: "When this creature enters from your hand, if it's attacking, create a
token that's a copy of another target **attacking** creature."
**Issue**: The card def correctly sets `is_attacking: true` AND `exclude_self: true` on
the TargetFilter, but per OOS-XS-2 (filed by PB-XS itself), `TargetFilter.is_attacking`
is silently ignored by `validate_object_satisfies_requirement` and the trigger
auto-target picker. The card's auto-target picker may pick a non-attacking creature.
Net game state: partially wrong.

**Fix**: choose one:
- **Revert to TODO** until OOS-XS-2 ships. Document why.
- **Ship as partial fix** with a stronger inline comment block flagging the
  wrong-game-state risk. The current TODO comment (lines 44-46) does call out the gap,
  but it's adjacent to the `exclude_self` PB-XS comment and could be missed.

Recommendation: ship as partial (the closest the card can get today), but elevate
the OOS-XS-2 callout to the file-level header comment block at lines 6-9 (currently
only documents the "enters from your hand" intervening-if gap, not the is_attacking
gap).

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 109.1 (object identity) | Yes | Yes (test C-1, D-1, E-1) | Source object identity vs candidate identity at declaration time. |
| 601.2c (target announcement) | Yes | Yes (test C-1, C-2, D-1) | Target rejected at announcement BEFORE costs paid. |
| 608.2b (target re-check at resolution) | Partial (zone only — pre-existing) | N/A | Gap is pre-existing; not in PB-XS scope per dispatch brief. Filed as E4. |
| 400.7 (zone-change new identity) | Yes (correct trigger.source binding for SelfDies) | **No direct test** | The graveyard arm (E-1) exercises a battlefield→graveyard scenario, but the source is on the battlefield. No test fires a real Elderfang Ritualist WhenDies trigger to verify trigger.source resolves to the post-death graveyard ObjectId. E1 HIGH covers this. |
| 603.10a (LKI / "look back in time") | Yes (relies on existing infrastructure) | Indirect | Same gap as 400.7 — E1 fix would close. |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|--------------------|-------|
| roalesk_apex_hybrid | Yes (ETB half) | 1 (death-trigger proliferate proliferate) | Yes for ETB half | C3 — pre-existing TODO out of PB-XS scope. |
| samut_voice_of_dissent | Yes | 0 | Yes | Clean. |
| torch_courier | Yes | 0 | Yes | Clean. |
| brash_taunter | Yes (activated half) | 1 (when-dealt-damage trigger) | Yes for activated half | C5 — pre-existing TODO out of PB-XS scope. |
| ezuri_renegade_leader | Yes | 0 | Yes | "another target Elf" with controller: Any — correct (oracle says "Elf", not "Elf you control"). |
| oath_of_teferi | Yes (ETB half) | 1 (loyalty-twice-per-turn) | Yes for ETB half | C4 — pre-existing TODO out of PB-XS scope. |
| elderfang_ritualist | Yes | 0 | Yes | Clean. Death-trigger graveyard arm exercised by test E-1 (indirectly — see E1 HIGH). |
| dour_port_mage | Yes (activated half) | 1 (leave-without-dying trigger) | Yes for activated half | C2 — pre-existing TODO out of PB-XS scope. |
| thousand_faced_shadow | Partial (exclude_self correct, is_attacking silently dropped) | 2 (intervening-if + is_attacking via OOS-XS-2) | Partial — auto-target may pick non-attacking creature | C1 LOW. |

## Specific Audit Responses (from dispatch brief)

### Q1: Missed call sites where matches_filter is invoked on a TargetFilter but exclude_self is silently dropped

I swept all `matches_filter(...)` callsites (~40 total across casting.rs, abilities.rs,
effects/mod.rs, layers.rs, replacement.rs). All non-target-validation sites correctly
ignore exclude_self by design (the field is documented as silently ignored by
matches_filter). Specifically verified:

- `effects/mod.rs:6230, 6232, 6256` — EffectAmount::CardCount and PermanentCount
  (counter-counting contexts, NOT target validation). exclude_self correctly ignored.
- `effects/mod.rs:7054, 7066` — Condition::YouControlPermanent / OpponentControlsPermanent
  (boolean condition evaluation, NOT target validation). Correctly ignored.
- `casting.rs:6946, 7026, 7069` — cost-reduction filter counting (Cavern-Hoard Dragon
  pattern). NOT target validation. Correctly ignored.
- `abilities.rs:4358, 5048, 6085, 6103, 6348` — trigger-fire eligibility filters
  (triggering_creature_filter, etb_filter, dealing-creature filter). These are
  TRIGGER-side filters, NOT target-side. Correctly ignored.
- `layers.rs:1490, 1511` — CDA resolution. Correctly ignored.
- `effects/mod.rs:926, 1128, 1251, 2270, 2674, 4157, 5079, 5745, 7441, 7636` — various
  effect-resolution scans (sacrifice filters, destroy filters, attach filters). NOT
  target validation. Correctly ignored.

**Target validation sites** (where exclude_self matters):
- `casting.rs:5645` — TargetSpellWithFilter. **exclude_self NOT checked here** because
  TargetSpellWithFilter targets a stack object, and the source spell's id can already
  be checked via TargetSpellOrAbilityWithSingleTarget's self-targeting prevention path
  (line 5666-5673). PB-XS's `exclude_self` field is theoretically applicable to TargetSpellWithFilter,
  but no in-scope card uses it. **Verdict**: not a finding; gap is documented behaviorally
  but doesn't affect any current card. Could be added in a future PB if needed.
- `casting.rs:5711, 5731, 5755, 5762` — TargetCreatureWithFilter, TargetPermanentWithFilter,
  TargetCardInYourGraveyard, TargetCardInGraveyard. **All four wired correctly.**
- `abilities.rs:6620, 6639, 6737, 6761, 6825, 6838` — trigger auto-target picker for the
  same 4 TargetRequirement variants (plus UpToN-inner variants). **All six wired correctly.**

**Verdict**: no silent-drop holes at target legality sites. The implementation is
complete for the 4 declared in-scope TargetRequirement variants.

### Q2: Elderfang Ritualist's post-death object identity (CR 400.7) — trigger.source resolution

Verified correct. In abilities.rs:4080-4095, the SelfDies trigger arm creates a
`PendingTrigger::blank(*new_grave_id, ...)`, where `new_grave_id` is the post-move-to-zone
graveyard ObjectId returned by `state.move_object_to_zone`. The `check_triggers` for
`CreatureDied` runs AFTER move_object_to_zone (line 4027 starts the arm with
`new_grave_id` already destructured from the event).

The auto-target picker's graveyard arm at abilities.rs:6627 then checks
`obj.id != trigger.source` while scanning graveyard objects. Since the dying Ritualist's
post-death ObjectId is now in the graveyard (with id = `new_grave_id`), and
`trigger.source == new_grave_id`, the comparison correctly excludes the post-death
Ritualist from being its own target.

**Confidence**: high. No test directly fires this path (the test E-1 places the
Necromancer on the battlefield, not in the graveyard, so it doesn't exercise the
"trigger.source IS in the graveyard" condition). E1 HIGH recommends adding a test
that does fire this scenario via the actual death pipeline.

### Q3: Re-validation at resolution — does exclude_self need re-checking?

Per CR 608.2b, the primary re-check is zone identity ("A target that's no longer in
the zone it was in when it was targeted is illegal"). The CR also mentions "Other
changes to the game state may cause a target to no longer be legal..." as a more
general statement.

**For exclude_self specifically**, the only way a target could become "newly self"
between announcement and resolution is via a control-change effect that transforms
the target into the source — extremely rare, and would also typically violate other
constraints. **The dispatch brief's claim that re-validation is unnecessary per CR
608.2b is correct in practice.**

There is a pre-existing gap (E4 LOW) for the broader CR 608.2b re-check of filter
properties (e.g. hexproof gained mid-resolution), but this is not a PB-XS regression
and PB-XS does not need to fix it.

**Verdict**: agree with the dispatch brief. No fix required.

### Q4: `validate_targets` dead-code retention — appropriate?

Marginally appropriate. The `#[allow(dead_code)]` annotation is honest about the
state of the code. The doc comment explains the speculative reason for retention
(callers without a source ObjectId). However, no concrete future caller is named,
and the alternative is one line longer.

Recommendation: **delete** `validate_targets` (E2 LOW). Re-add if a real need surfaces.
The cost of re-adding is trivial.

### Q5: Missed cards with "another target" pattern

Sweep done. Grep `another target` in `crates/engine/src/cards/defs` returns 11 files:

- **9 in scope, in this PB**: roalesk_apex_hybrid, samut_voice_of_dissent, torch_courier,
  brash_taunter, ezuri_renegade_leader, oath_of_teferi, elderfang_ritualist,
  dour_port_mage, thousand_faced_shadow. **All have `exclude_self: true` set on their
  PB-XS-relevant TargetFilter.** Verified.
- **2 documented OOS**:
  - `olivia_voldaren.rs` — needs `exclude_self` PLUS LayerModification::AddSubtype (OOS-XS-3 filed).
  - `hidden_strings.rs` — needs `TargetRequirement::TargetPermanentDistinctFrom(usize)`,
    not `exclude_self` (OOS-XS-1 filed).

**No missed cards.**

Additional sweep: I checked for the related pattern "other target" (which might
indicate a non-self exclusion):

- `return_to_dust.rs` — "Exile target artifact or enchantment. If you cast this spell
  during your main phase, you may exile up to **one other target** artifact or
  enchantment." This is **inter-target distinctness**, not exclude_self. Properly OOS
  via OOS-XS-1 (Hidden Strings family).
- `skrelv_defector_mite.rs` — "Another target creature you control" — documented as
  OOS-XS-4 (needs ChooseColor + protection-from-color + can't-block-by-color
  primitives).

**No missed cards in the "other target" sweep either.**

## Previous Findings

This is the initial review; no prior findings to re-review.

---

## Summary

PB-XS is a tight, well-bounded primitive. The engine change is mechanically correct
at all dispatch sites, the hash bump is complete with full sentinel sweep, the card
defs match oracle text, and the OOS seeds are well-filed. The one HIGH (E1) is a
test-validity issue: `test_pbxs_etb_auto_target_picker_skips_source` is a tautology
and the 6 trigger auto-target picker sites have zero direct test coverage. This
must be fixed in a brief fix-phase before signal-ready.

The four LOWs (E2-E5, C1) are low-impact cleanup items. E3 (stale sentinel error
messages) is the most worthwhile of them — fixing it improves future debuggability
across all sentinel files.

**Coordinator action**: dispatch a fix-phase runner to (a) rewrite
test_pbxs_etb_auto_target_picker_skips_source to actually exercise the trigger
auto-target picker (preferably via Elderfang Ritualist's death-trigger graveyard
arm, which doubles as the CR 400.7 / 603.10a discriminator), and (b) sweep stale
sentinel error messages. E2, E4, E5, C1 can ship-as-is or be batched into a
future LOW-cleanup pass.
