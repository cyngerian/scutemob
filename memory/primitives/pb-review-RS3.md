# Primitive Batch Review: PB-RS3 — card-def `AtBeginningOfCombat` sweep (OOS-OS9-1)

<!-- last_updated: 2026-07-20 -->

**Date**: 2026-07-20
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-145` · commits `95f1c306`, `95626742`, `7f00f3a6`
**CR Rules**: 101.4, 114.4, 400.7, 506.1, 507.1/507.2, 508.1d, 603.2, 603.3/603.3a/603.3b, 603.4, 702.6, 712.8d/e, 903.3d
**Engine files reviewed**: `crates/engine/src/rules/turn_actions.rs` (sweep + end-step comment repair)
**Card defs reviewed**: 6 — `helm_of_the_host`, `loyal_apprentice`, `siege_gang_lieutenant`, `goblin_rabblemaster`, `legion_warboss`, `mirage_phalanx`
**Tests reviewed**: `tests/primitives/pb_rs3_at_beginning_of_combat_sweep.rs` (8), `tests/primitives/pb_rs3_rabblemaster_mustattack_probe.rs` (1), `tests/core/pb_rs3_combat_trigger_roster.rs` (1)

## Verdict: needs-fix (0 HIGH, 3 MEDIUM, 4 LOW)

**There is nothing HIGH here, and I am saying that plainly rather than inflating a LOW.**
The sweep itself is correct: I diffed it line-by-line against the S3 sibling
(`postcombat_main_actions`, now `:488-542`) and it is a faithful copy with only the
`TriggerCondition` variant and the comments substituted. The `ability_index` namespace
decision — the subtlest part of this PB and the one the plan flagged as its most likely
defect — is **independently confirmed correct**: `resolution.rs:2018-2048` reads
`def.effective_abilities(obj.is_transformed).get(ability_index)` for the `CardDefETB`
kind, so enumerating `effective_abilities()` is required, not a bug, and Test 2 is a real
discriminator for it. Emblem/card-def disjointness is **structural, not incidental** —
`collect_emblem_triggers_for_event` filters `obj.is_emblem && matches!(obj.zone,
ZoneId::Command(_))` (`abilities.rs:6781`) against the sweep's `zone == Battlefield`, so
no object can be seen by both. Wire closure holds (PROTOCOL 27, HASH 63, both files
untouched). Determinism holds (`objects` is `OrdMap`, so iteration is ObjectId-ordered).
The hazard measurement in plan §8 was performed in full and produced real information
(three seeds filed, including a previously-unknown live-wrong `Complete` card,
`emeria_the_sky_ruin`).

The three MEDIUMs all cluster on the **third `Complete` flip** (`goblin_rabblemaster`,
which was not in the plan's flip list) and on the **accuracy of the F3 seed text**. None
of them says the shipped code computes a wrong game state on the paths that are tested;
they say the `Complete` assertion on Rabblemaster is broader than the evidence backing it,
and that a filed seed misstates the engine's actual state in a way that will misdirect its
fix.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `rules/combat.rs:421-424` (inherited) | **Must-attack "able" test ignores `CantAttackYouUnlessPay`, and Rabblemaster now mass-produces forced attackers.** Deadlock is reachable. **Fix:** record in the def note + file a seed. |
| 2 | MEDIUM | `memory/primitives/rider-seed-triage-2026-07-19.md:89` | **OOS-RS3-1 seed text is factually wrong** — the emblem sweep *does* check intervening-if at queue time. **Fix:** correct the seed's scope claim. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | MEDIUM | `goblin_rabblemaster.rs` | **The third flip has no end-to-end test on the real def.** Mock-def probe + completeness pin only. **Fix:** add an end-to-end test. |
| 4 | LOW | `goblin_rabblemaster.rs` | Flip authority misattributed to plan §5c, which says the opposite. **Fix:** correct the provenance line in `primitive-wip.md`. |
| 5 | LOW | `pb_os5_relative_attacker_count.rs:13` | Stale "(partial, pump clause implemented)" comment. **Fix:** update to `Complete`. |
| 6 | LOW | `core/pb_rs3_combat_trigger_roster.rs:39-49` | Walk does not recurse into a matching key's subtree. **Fix:** recurse on mismatch too, or comment the assumption. |
| 7 | LOW | `pb_rs3_at_beginning_of_combat_sweep.rs:239-256` | Test 2 doc names only half of hazard R1. **Fix:** name the dense-namespace variant too. |

---

### Finding Details

#### Finding 1: Rabblemaster's `Complete` inherits a reachable must-attack deadlock

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/combat.rs:392-430` (pre-existing); exposed by `crates/card-defs/src/defs/goblin_rabblemaster.rs:34-46`
**CR Rule**: 508.1d — "…the declaration is illegal if… it doesn't include a creature that must attack **if able**." Ruling 2014-07-18 on this exact card: *"If there's a cost associated with having a creature attack, you're not forced to pay that cost, so it doesn't have to attack in that case either."*

**Issue**: `combat.rs` computes `cannot_attack` from exactly four inputs
(`:421-424`): tapped-without-vigilance, summoning-sick-without-haste, Defender, and the
`CantAttackOwner` no-legal-target case. It does **not** consider
`GameRestriction::CantAttackYouUnlessPay`. That restriction is not a stub — it is fully
enforced at `combat.rs:185-224`, where a declaration the player cannot pay the tax for is
**rejected**. The two checks are therefore mutually unsatisfiable in a live, format-typical
board state:

> p1 controls Goblin Rabblemaster and its 1/1 Goblin token; p2 controls Ghostly Prison;
> p1 has no untapped mana. Declaring the token → rejected by the attack-tax check
> (`:185-224`). Omitting the token → rejected by the must-attack check (`:425-430`,
> because `cannot_attack` is false). **p1 cannot legally declare attackers at all.**

Ghostly Prison / Propaganda are 4-player Commander staples, and Rabblemaster *manufactures
a new forced attacker every single combat*, so this is not a corner case for this
particular card. The gap is genuinely pre-existing (it is shared by every
`MustAttackEachCombat` card already shipped), so this is **not** a regression introduced by
PB-RS3 — but the flip to `Complete` asserts correctness in a 4-player Commander game
(`memory/project_legal_but_wrong_gap.md`), and unlike F3 — which the PB handled well by
recording it verbatim in both flipped defs' notes and filing OOS-RS3-1 — this one is
recorded **nowhere**: not in the def note, not in the probe, not in a seed.

**Fix**: Amend `goblin_rabblemaster.rs`'s completeness note to record that the granted
`MustAttackEachCombat` is enforced by an "able" test that ignores attack taxes
(`combat.rs:421-424`), citing the 2014-07-18 ruling, exactly as `loyal_apprentice.rs:21-29`
records F3. File a seed (suggest `OOS-RS3-4`, class **correctness, engine-wide**) in
`memory/primitives/rider-seed-triage-2026-07-19.md` §1 covering the `cannot_attack`
computation vs `CantAttackYouUnlessPay` (and check whether goad enforcement at
`combat.rs:340-373` has the same hole — likely yes, which widens the class). Do **not**
fix the enforcement in this PB; it is out of scope. The flip itself may stand once the
limitation is recorded.

#### Finding 2: The OOS-RS3-1 seed text misstates the engine's actual state

**Severity**: MEDIUM
**File**: `memory/primitives/rider-seed-triage-2026-07-19.md:89`
**CR Rule**: 603.4 — the condition is checked when the event occurs **and** again on resolution.

**Issue**: The seed reads *"`resolution.rs:2125-2135` is the only check; **no trigger sweep
evaluates `intervening_if` before pushing the `PendingTrigger`**."* The second clause is
false. `collect_emblem_triggers_for_event` — the very function `begin_combat` calls three
lines below the new sweep — evaluates it at queue time:

```rust
// abilities.rs:6798-6803
// CR 603.4: Check intervening-if at trigger time.
if let Some(ref cond) = trigger_def.intervening_if {
    if !check_intervening_if(state, cond, obj.controller, None) {
        continue;
    }
}
```

So CR 603.4's trigger-time half **is** implemented on the `TriggerEvent`/`Normal` path, and
the gap is specific to the `CardDefETB` card-def sweeps. This matters for disposition, not
just pedantry: the seed's "not ranked" justification is *"a fix must add a queue-time
evaluation at each sweep site and **decide the shared-helper shape first**." The shared
helper already exists and is already in use (`check_intervening_if`), so the fix is
materially cheaper than the seed claims, and an inaccurate seed will keep OOS-RS3-1
unranked longer than the evidence warrants.

**Fix**: Correct the seed to say the gap is confined to the `PendingTriggerKind::CardDefETB`
card-def sweeps (upkeep / precombat main / postcombat main / end step / begin combat), note
that `abilities.rs:6798-6803` is a working in-repo reference implementation using
`check_intervening_if`, and re-assess the "not ranked" call in light of that. Also soften
the corresponding sentence in `loyal_apprentice.rs:26-27` and `siege_gang_lieutenant.rs:21-22`
("a pre-existing, engine-wide convention") to "engine-wide across the card-def trigger
sweeps," since as written it implies the emblem path shares the defect.

**Note on the F3 disposition question in the brief**: with that correction, the disposition
is still **defensible** and the flips are **not** overclaims. The divergent case (condition
false at trigger time, true by resolution) requires an instant-speed control change /
blink / phase-in of a commander inside the beginning-of-combat step; it is narrow, it is
symmetric across every already-shipped `Complete` intervening-if card, the correct
direction (true→false) *is* handled, and the PB recorded the limitation in both card notes
rather than burying it. That is the right call. Fixing it would touch five sweeps and
belongs in its own PB.

#### Finding 3: The third flip has no end-to-end test on the real card def

**Severity**: MEDIUM
**File**: `crates/card-defs/src/defs/goblin_rabblemaster.rs:112`; `crates/engine/tests/primitives/pb_rs3_rabblemaster_mustattack_probe.rs`
**Oracle**: "Other Goblin creatures you control attack each combat if able. / At the beginning of combat on your turn, create a 1/1 red Goblin creature token with haste. / Whenever this creature attacks, it gets +1/+0 until end of turn for each other attacking Goblin."

**Issue**: I checked the mechanism the probe tests and it is sound. `EffectFilter::Other
CreaturesYouControlWithSubtype` (`layers.rs:758-775`) correctly requires battlefield +
`CardType::Creature` + subtype match + `source_id != object_id` (CR "other") +
`source_controller == obj_controller`, all evaluated dynamically. `combat.rs:378-390` reads
`expect_characteristics`, i.e. layer-resolved, so a granted keyword on a non-source object
is picked up. `EffectLayer::Ability` is the right layer (6) for `AddKeyword`. The probe is
**non-vacuous** — it asserts the positive grant, the CR-correct negative control (the source
itself is not forced), an `Err` on an under-declaration, and an `Ok` on the forced
declaration, all driven through `Command::DeclareAttackers`, and the implementer confirms it
fails when `register_static_continuous_effects` is disabled. That is good work and the
F-Rabble lead is genuinely vindicated.

**But the probe builds `mock_rabblemaster_grant_def()`, not `goblin_rabblemaster`.** Nothing
in the suite constructs the real card. Consequently:

- Nothing verifies the real def's `AbilityDefinition::Static` is actually picked up by
  `register_static_continuous_effects` on a real ETB (the probe registers it by hand).
- Nothing verifies the real def's `AtBeginningOfCombat` trigger fires and creates a Goblin
  (Tests 1-8 cover Helm, Apprentice, Siege-Gang — never Rabblemaster).
- Nothing verifies the load-bearing composition: the token created by ability [1] is a
  Goblin, so ability [0] must then force **it** to attack. That interaction is the whole
  card, it is confirmed by the 2014-07-18 rulings, and it is untested.

The roster sweep pins `Completeness::Complete` for Rabblemaster, which is a *marker* assertion,
not a *behavior* assertion — it will keep passing if the Static grant silently stops
registering. Given that this flip was earned mid-implementation and sits outside both the
plan's predicted table and AC 5125's named cards, it carries the least evidence of the four
and should carry the most.

**Fix**: Add `test_goblin_rabblemaster_end_to_end` to
`tests/primitives/pb_rs3_at_beginning_of_combat_sweep.rs` using the real def from
`all_cards()`: place Rabblemaster on the active player's battlefield, drive the real
`PreCombatMain → BeginningOfCombat` transition, drain the stack, assert exactly one 1/1 red
Goblin token with haste exists, then advance to `DeclareAttackers` and assert that a
declaration omitting that token is rejected (CR 508.1d) while one including it is accepted.
Cite CR 508.1d + 603.2 and the 2014-07-18 ruling.

#### Finding 4: Flip authority misattributed

**Severity**: LOW
**File**: `memory/primitive-wip.md:119-120`

**Issue**: The step-4 record justifies the Rabblemaster flip as *"per plan §5c authorization
for a clean composition."* Plan §5c contains no such authorization; it is titled *"The other
three roster members — do NOT flip"* and says of Rabblemaster: *"Stays `partial`; the
surviving blocker is the subtype-filtered forced-attack `GameRestriction`."* The flip was in
fact earned by the roster reviewer's F-Rabble HIGH finding plus a purpose-built probe — a
better and more honest justification than the one recorded. Citing an authority that says the
opposite is the kind of drift that makes a later reader trust a claim they shouldn't
(`memory/conventions.md` §"Aspirationally-wrong code comments are correctness hazards"
applies to plan citations too).

**Fix**: Rewrite that clause to cite the roster review's F-Rabble finding and the probe as
the authority, and state explicitly that it overrides plan §5c's prediction.

#### Finding 5: Stale completeness claim in a neighbouring test

**Severity**: LOW
**File**: `crates/engine/tests/primitives/pb_os5_relative_attacker_count.rs:13`

**Issue**: Reads *"Goblin Rabblemaster (partial, pump clause implemented)"*. The card is now
`Complete`. The PB was otherwise diligent about this class of repair (it fixed
`turn_actions.rs:689-690` and the `pb_os9_lieutenant_commander_control.rs` headers); this one
was missed.

**Fix**: Update to `(Complete as of PB-RS3)`.

#### Finding 6: Roster walk does not recurse past a matching key

**Severity**: LOW
**File**: `crates/engine/tests/core/pb_rs3_combat_trigger_roster.rs:39-49`

**Issue**: When `k == field`, the closure returns the match result for that child and never
recurses into it. If a `trigger_condition` value ever nests another `trigger_condition`
(a modal or wrapped condition), inner occurrences would be missed. The disambiguation itself
is **correct and well-reasoned** — scoping to the `trigger_condition` key genuinely separates
`TriggerCondition::AtBeginningOfCombat` from the identically-serializing
`TriggerEvent::AtBeginningOfCombat` on `TriggeredAbilityDef::trigger_on`, and the test asserts
the Basri Ket exclusion directly, which is exactly the right guard. The non-vacuity floor
(`>= 6`) plus six per-member completeness pins is a real floor, and the implementer confirms
it collapses to 0 when the field-name match is broken.

**Fix**: Recurse into `child` as well when the keyed match fails, or add a one-line comment
recording the flat-value assumption.

#### Finding 7: Test 2 doc names only half of hazard R1

**Severity**: LOW
**File**: `crates/engine/tests/primitives/pb_rs3_at_beginning_of_combat_sweep.rs:239-256`

**Issue**: The plan (§6 Test 2) asked the doc comment to name the hazard so a future reader
cannot simplify it away. The comment thoroughly describes the *filter-then-enumerate* failure
mode but never names plan §12's R1 hazard — an implementer "correcting" the sweep to use the
dense `characteristics.triggered_abilities` namespace to match `collect_triggers_for_event`.
The test catches both (the dense namespace has no lowering arm for step-based conditions, so
it yields nothing), but the doc only inoculates against one.

**Fix**: Add a sentence naming the dense-`triggered_abilities` namespace as the other
regression this test catches, pointing at `resolution.rs:2019-2020` and
`tests/primitives/pb_ac7_ability_index_desync.rs`.

---

## Answers to the brief's ten verification questions

| # | Question | Verdict |
|---|----------|---------|
| 1 | Sweep correctness / sibling fidelity / controller scope / guard placement / SR-7 | **Correct.** Verbatim S3 copy; active-only early `return None` matches the DSL variant's "on your turn" doc and all three cards' oracle text; sweep sits **outside** `if state.combat.is_none()` (`:1689-1691` closes before `:1708`); push is `..PendingTrigger::blank(..., CardDefETB)`. |
| 2 | `ability_index` namespace | **Correct, independently verified.** `resolution.rs:2023-2030` indexes `effective_abilities()`; the dense namespace has no step-based lowering arm and would find nothing. Test 2 discriminates. |
| 3 | Emblem + card-def coexistence | **Structurally disjoint.** `is_emblem && Command(_)` vs `Battlefield`; distinct enums; both push to one deque, APNAP-sorted as one CR 603.3b batch. Test 6 pins order. |
| 4 | Three flips + helm repair | Helm/Apprentice/Siege-Gang: **oracle-exact**, MCP-verified, endorsed. Rabblemaster: mechanism sound, but see Findings 1, 3, 4. |
| 5 | F3 disposition | **Defensible** — narrow divergent case, correct direction handled, recorded in both notes, symmetric across the shipped corpus. The flips are not overclaims. But the seed text is wrong (Finding 2). |
| 6 | Test discrimination | **Genuine.** Test 1 asserts an observable token count through the real step transition; 3b removes the commander mid-stack; Test 5 asserts the `AbilityTriggered` event list, not just the outcome; Test 7 asserts 2 tokens across two combats. Test 7's doc was *corrected downward* during verification when the R2 mutation proved unreproducible — that honesty is the strongest single signal in this batch. |
| 7 | Roster sweep | **SR-36 compliant** (`all_cards()`, not grep); floor real; `TriggerEvent`/`TriggerCondition` disambiguation correct via field-name scoping. Minor: Finding 6. |
| 8 | `mirage_phalanx` containment | **Sufficient.** `known_wrong` + `validate_deck` (SR-2); no `ObjectSpec` fixture constructs it; the note now records over-production in both directions, converting lucky containment into deliberate containment. |
| 9 | Wire closure | **Unchanged.** `PROTOCOL_VERSION = 27` (`protocol.rs:260`), `HASH_SCHEMA_VERSION = 63` (`hash.rs:578`); no `Command`/`Effect`/`GameEvent`/`PendingTriggerKind`/`TriggerData` shape touched. |
| 10 | Scope discipline | **Mostly held.** §8c registry correctly measured-and-filed, not built; `emeria_the_sky_ruin` correctly filed as a seed rather than fixed; `legion_warboss` correctly *not* "fixed" with a wrong-duration keyword. One overrun: the Rabblemaster flip exceeds AC 5125's named cards and plan §5c — substantively justified, but under-tested (Finding 3) and mis-cited (Finding 4). |

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 506.1 | Yes | Yes | Test 7 (extra combat, 2 tokens) |
| 507.1/507.2 | N/A | — | Sweep correctly queues before priority; not a turn-based action |
| 603.2 | Yes | Yes | Tests 1, 5, 7, 8 |
| 603.3 | Yes | Yes | Test 1 — `flush_pending_triggers` before priority |
| 603.3a | Yes | Yes | Test 2 (index namespace), Test 5 (controller) |
| 603.3b | Yes | Partial | Test 6 pins queue order; APNAP sort is a documented no-op (single-controller batch) — honestly framed |
| **603.4** | **Partial** | Partial | Resolution half only (Tests 3a/3b). Queue half absent on the CardDefETB path — OOS-RS3-1, see Finding 2 |
| 101.4 | Trivially | Yes | Test 5; batch single-controller by construction |
| 114.4 | Pre-existing | Yes | Test 6 (emblem coexistence) |
| **508.1d** | **Partial** | Partial | Probe covers the grant; "if able" ignores attack taxes — Finding 1 |
| 702.6 | Yes | Yes | Test 8 (unattached Helm → 0 tokens, no panic) |
| 712.8d/e | Yes | No | `effective_abilities(is_transformed)` carried forward from PB-OS4b; no transformed-permanent combat trigger exists in the corpus to test |
| 903.3d | Yes | Yes | Tests 2, 3a/3b |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `helm_of_the_host` | Yes (MCP-exact) | 0 | Yes | Explicit `Complete` replaces `#[default]`; Invariant #9 repair — the real value of this PB |
| `loyal_apprentice` | Yes (MCP-exact) | 0 | Yes, modulo F3 | Trigger at `abilities[1]`; permanent-haste fallback unobservable |
| `siege_gang_lieutenant` | Yes (MCP-exact) | 0 | Yes, modulo F3 | Both abilities present; intervening-if tested both directions |
| `goblin_rabblemaster` | Yes (MCP-exact, all 3 abilities) | 0 | Likely — untested end-to-end | Findings 1, 3, 4 |
| `legion_warboss` | Yes | 0 | No — stays `partial` | Note correctly names **both** gaps and correctly refuses the wrong-duration `MustAttackEachCombat` fix |
| `mirage_phalanx` | No — stays `known_wrong` | 0 | No, contained | Note records over-production; containment verified |

## Recommended disposition

Findings 4-7 are two-minute edits. Finding 2 is a documentation correction to a filed seed.
Finding 1 is a note amendment plus one seed filing. Finding 3 is the only one requiring new
test code (~60 lines, reusing this file's existing helpers).

**None of these blocks the merge of the sweep or the first three markers.** If the
coordinator wants to ship now, Findings 1 and 3 should be resolved before
`goblin_rabblemaster` is counted toward the coverage number, since that flip is the one this
review found least-evidenced.
