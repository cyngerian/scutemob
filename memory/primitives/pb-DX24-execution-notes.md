# PB-DX24 — execution notes (stages 1-4 of this invocation)

**Task**: `scutemob-202` · **Branch**: `feat/pb-dx24-the-lowering-drops-triggerzone-the-two-index-spaces-`
**Scope of this invocation**: plan stages 1-4 only. Stage 5 (Q1-Q7 index-space fixes), stage 6
(gates), stage 7 (close-out, seed filing) are OUT OF SCOPE and NOT done here.

---

## Stage 0 re-verification (before stage 1)

Every line number cited in `pb-DX24-stage0.md` re-checked at HEAD (`c15e685b`) and confirmed
unchanged (PB-DX23 merged since the stage-0 census, but this branch's own stage-0 commit already
captured the post-DX23 state, so nothing shifted from that baseline):

- Region boundaries: comment starts at `:2527` (`// CR 603.6c / CR 700.4: Convert "When ~ dies"...`),
  `for ability in abilities {` loop starts at `:2531`. Return statement at `:3862`, closing brace
  `:3863`. Matches stage 0 exactly.
- The `WheneverPermanentEntersBattlefield` guard confirmed at `:3029-3051` (comment `:3029-3032`,
  guard `:3048-3051`) — one line off from stage 0's `:3049-3051` cite (an off-by-one in the
  original census, not a real discrepancy).
- `rg -n 'ability'` over the extracted range: **34** `for ability in abilities {` loops, **34**
  matching `} = ability` scrutinee bindings, **zero** other uses of the bare identifier `ability`
  outside `if let ... = ability { ... }` scrutinees (all remaining hits are inside `//` comments).
  Confirms §2.4's compile-note claim before any edit was made.
- No `.enumerate()` / index arithmetic / `position(` in the extracted region (confirmed by grep;
  the function's only `.enumerate()` in the whole file, at `:7186` post-extraction, is inside
  `collect_graveyard_carddef_triggers`, outside the extracted range).
- `GameEvent` variants carrying `new_grave_id`: enumerated by reading `rules/events.rs` directly —
  `CreatureDied`, `PlaneswalkerDied`, `AuraFellOff`, `PermanentDestroyed`, `ObjectPutInGraveyard`
  (all field-named `new_grave_id`). `PermanentSacrificed` carries the SAME shape id but under the
  field name `new_id`, not `new_grave_id` — a real, verified name mismatch against the plan's
  §3.2 literal list (see Stage 4 below for the disposition).
- `cargo test --workspace --no-fail-fast`: **4,413 / 0 / 5**, matching the plan's pinned baseline
  exactly. `PROTOCOL_VERSION` = 35 (source + `--test core protocol_schema` both confirm),
  `HASH_SCHEMA_VERSION` = 73 (source + `--test core hash_schema` both confirm).

---

## Stage 1 — SR-36 measurement (§4.2)

Enumerated via a throwaway scratch test (`crates/engine/tests/primitives/pb_dx24_stage1_measure_scratch.rs`,
run once, then deleted — never committed; `git status` confirmed clean before continuing) that
iterates `all_cards()`, filters `def.back_face.is_some()`, and scans `face.abilities` for each of
the 7 shapes.

| measurement | value |
|---|---|
| `all_cards().len()` | **1,803** |
| defs with `back_face: Some(_)` | **15** (cross-checked: `grep -rl "back_face: Some" crates/card-defs/src/defs/ \| wc -l` also gives 15) |
| Q1 `Keyword(Backup(_))` on back face | **0** |
| Q2 `Triggered { trigger_condition: WhenYouCastThisSpell, .. }` on back face | **0** |
| Q3 `Triggered { trigger_condition: WhenExertedAsAttacks, .. }` on back face | **0** |
| Q4 `Triggered { trigger_condition: WhenDealsCombatDamageToPlayer, .. }` on back face | **0** |
| Q5 `Triggered { trigger_condition: WhenTurnedFaceUp, .. }` on back face | **0** |
| Q6 `Triggered { trigger_condition: WheneverRingTemptsYou, .. }` on back face | **0** |
| Q7 `Triggered { trigger_zone: Some(_), .. }` on back face | **0** |

**All seven counts are zero on the real corpus.** Per plan §4.4: every Q-site probe in stage 5
(NOT done in this invocation) will need a **synthetic** `CardDefinition`/`CardFace` fixture, never
a real corpus card — the plan's "decide per Q-site, real vs synthetic" question is answered
uniformly: synthetic for all seven.

The 15 `back_face: Some(_)` defs (for the record): Growing Rites of Itlimoc, Bloodline Keeper,
Hanweir the Writhing Township, Brutal Cathar, Braided Net, Fable of the Mirror-Breaker, Disciple
of Freyalise, Bridgeworks Battle, Thaumatic Compass, Revitalizing Repast, Legions' Landing,
Beloved Beggar, Sea Gate Restoration, Docent of Perfection, Delver of Secrets. None of them
carries any of the 7 shapes on its back face.

---

## Stage 2 — fail-before T1/T7

**Preparatory, behaviour-neutral edit**: `build_face_ability_vectors` promoted `pub(crate)` →
`pub` (visibility only) so an integration test can call it directly — this is plan §6's own
"preferred" resolution to T7's access problem, taken rather than the fallback (driving it through
`enrich_spec_from_def`).

New file `crates/engine/tests/primitives/pb_dx24_trigger_zone_and_index_spaces.rs`, `mod` line
added to `crates/engine/tests/primitives/main.rs`.

**T1** (`test_dx24_nether_traitor_does_not_trigger_from_the_battlefield`) — fixture: Nether
Traitor + Blood Artist (non-vacuity control) + a toughness-0 "Fodder" creature, all on the
battlefield, `check_and_apply_sbas(&mut state)` directly (no priority passing needed for a
pending-trigger-level check).

Observed failure (executed against the tree with ONLY the `pub` visibility change, i.e. before
any lowering-filter change):
```
CR 113.6 / CR 113.6m: Nether Traitor on the BATTLEFIELD must not trigger when another creature
dies -- the ability functions only from the graveyard. Got 1 trigger(s) sourced at the
battlefield object: [PendingTrigger { source: ObjectId(1), ability_index: 2, controller:
PlayerId(1), kind: Normal, triggering_event: Some(AnyCreatureDies), ... }]
```

**T7** (`test_dx24_lowering_drops_every_zone_scoped_ability_over_the_corpus`) — differential over
`all_cards()`: `build_face_ability_vectors(&def.abilities)` vs the same call with every
`trigger_zone: Some(_)` ability manually stripped from the input first.

Observed failure:
```
CR 113.6b / CR 113.6m: build_face_ability_vectors must lower a def's abilities IDENTICALLY
whether or not its trigger_zone: Some(_) abilities are present in the input -- a lowering arm is
installing a zone-scoped ability onto the battlefield object's runtime trigger vector. Divergent
defs: ["Nether Traitor"]
```

Both watched failing in the SAME `cargo test -p mtg-engine --test primitives pb_dx24` run
(2 failed / 0 passed), naming Nether Traitor in both messages. Committed at `e3b61022`.

---

## Stage 3 — the lowering filter (Change 1 + Change 2)

**Mechanism, exactly as designed**: extracted the trigger-lowering region (previously
`:2531`-`:3862` inline in `build_face_ability_vectors`) into a new private function
`build_face_triggered_abilities(abilities: &[&AbilityDefinition]) -> Vec<TriggeredAbilityDef>`,
added `lowers_onto_the_battlefield(ability: &AbilityDefinition) -> bool` (exhaustive match on
`TriggerZone`, single variant `Graveyard` today — a future variant is a compile error, not a
silent default), and a single filter+call at `build_face_ability_vectors`'s new tail:

```rust
let battlefield_triggers: Vec<&AbilityDefinition> = abilities
    .iter()
    .filter(|a| lowers_onto_the_battlefield(a))
    .collect();
let triggered_abilities = build_face_triggered_abilities(&battlefield_triggers);
```

**Compile note verified, not assumed**: `for ability in abilities` where
`abilities: &[&AbilityDefinition]` binds `ability: &&AbilityDefinition`; every one of the 34
`if let AbilityDefinition::Triggered { .. } = ability` arms compiled UNCHANGED (default binding
modes peeled both references). No arm needed editing beyond the one guard removal below. The
`pub`/no-fallback path from stage 2 was already in place.

**Guard removal**: deleted the `WheneverPermanentEntersBattlefield` arm's
`if trigger_zone.is_some() { continue; }` (former `:3048-3051`) AND removed `trigger_zone,` from
that arm's destructure pattern entirely (not just the guard) — a residual `trigger_zone,` binding
with no use would either warn (`unused variable`) or, worse, invite a future author to reintroduce
a per-arm check. Rewrote the surrounding comment to point at the new single-filter-site design.

**Doc table (Change 2)**: rewrote the `trigger_zone` row. **Arm count correction, per Change 2's
own instruction to re-derive rather than trust**: the plan/stage-0 claimed "40 arms, 1 checks
trigger_zone." Re-measured directly (`grep -c 'trigger_condition:'` inside the extracted region,
cross-checked against `for ability in abilities {` loop count): **34 arms, 1 checks
trigger_zone** — not 40. This is recorded as a real, verified correction (the PB-DX19 "three
published counts" lesson recurring): the `once_per_turn`/`intervening_if` table cells that cited
"34" were already correct; only the stage-0 census's OWN new arm-count claim ("40") was wrong.
Left the `intervening_if`/`once_per_turn` rows' existing "34" citations untouched (they were
already right) and did not introduce a second, conflicting count.

**T7's own non-vacuity design bug, found and fixed mid-stage**: T7's original assertion
"`divergent_defs` must be non-empty" was written as a PERMANENT test-body assertion, but that
claim is only ever true PRE-fix — asserting it post-fix would make T7 permanently red after the
very fix it exists to gate (exactly the class of self-defeating test this project's conventions
warn against). Removed that assertion, replaced with a code comment pointing at this file's
fail-before record; kept the OTHER non-vacuity floor (`non_identity_inputs >= 1`), which holds
regardless of fix state.

**Verification, all executed**:
- `cargo build --workspace` clean.
- T1 + T7 green (`cargo test -p mtg-engine --test primitives pb_dx24`).
- `cargo test -p mtg-engine --test core face_dereg_parity` — 2/2 green (unmoved).
- `cargo test -p mtg-engine --test primitives pb_dx1_lowered_intervening_if` — 17/17 green
  (unmoved; the other consumer of this function's contract).
- `cargo test --workspace --no-fail-fast`: **4,415 / 0 / 5** (+2 over the 4,413 baseline — exactly
  T1 + T7, no pre-existing test reddened).

Committed at `cf046000`.

---

## Stage 4 — the graveyard death dispatch (Change 3)

### Fail-before (T2-T6 + two extra sub-probes)

Wrote T2 (`test_dx24_nether_traitor_triggers_from_the_graveyard`), T3
(`test_dx24_nether_traitor_returns_itself_end_to_end`), T4
(`test_dx24_simultaneous_death_does_not_trigger`), T5
(`test_dx24_exclude_self_compares_the_graveyard_identity`), T6
(`test_dx24_graveyard_death_filters_mirror_the_battlefield_path`), plus two extra sub-probes the
plan's §6 table folds into "T6" but which are cleaner as their own functions given they need
SYNTHETIC card defs (a corpus card with `nontoken_only: true` or a subtype `filter` alongside
`trigger_zone: Graveyard` does not exist): `test_dx24_graveyard_death_filter_nontoken_only`,
`test_dx24_graveyard_death_filter_subtype_filter`.

All 6 reddened against the unmodified (post-stage-3) tree — `collect_graveyard_carddef_triggers`
had no `GameEvent::CreatureDied` dispatch arm at all. T1, T5\*, T7 stayed green in the SAME run
(\*T5 was a VACUOUS pass at this point — see the finding below; not yet meaningful).

Failure output (excerpted, full capture in `/tmp/.../stage4-failbefore.txt`):
```
test_dx24_graveyard_death_filter_nontoken_only ... FAILED
test_dx24_graveyard_death_filter_subtype_filter ... FAILED
test_dx24_nether_traitor_triggers_from_the_graveyard ... FAILED (0 triggers, expected 1)
test_dx24_nether_traitor_returns_itself_end_to_end ... FAILED
test_dx24_graveyard_death_filters_mirror_the_battlefield_path ... FAILED (non-vacuity: 0 triggers, expected 1)
test_dx24_simultaneous_death_does_not_trigger ... FAILED (non-vacuity: 0 triggers, expected 1)
```
Committed at `299f9ddf`.

### Implementation (§3.1-§3.6)

**3.1/3.2** — new parameter `arrived_in_graveyard_this_batch: &HashSet<ObjectId>` on
`collect_graveyard_carddef_triggers`; the batch-arrival set computed once at the top of
`check_triggers`, before `for event in events {`.

**Runner obligation — the real `GameEvent` variant enumeration (§3.2), done by reading
`events.rs`, not by trusting the plan's list**: the plan's literal code sample named
`CreatureDied, PermanentDestroyed, PermanentSacrificed, AuraFellOff`. Measured instead:

- **Included** (all field-named `new_grave_id`): `CreatureDied`, `PlaneswalkerDied`,
  `PermanentDestroyed`, `AuraFellOff`, `ObjectPutInGraveyard`.
- **`PermanentSacrificed` EXCLUDED** — its field is `new_id`, not `new_grave_id` (the plan's own
  sample code would not have compiled as literally written). Every creature-sacrifice call site
  that emits `PermanentSacrificed` ALSO emits `CreatureDied` with the IDENTICAL id in the SAME
  push (verified by reading `abilities.rs:966-992` and `casting.rs:4321`/`:4340-4344` — both push
  `CreatureDied { new_grave_id: new_id, .. }` immediately followed by
  `PermanentSacrificed { new_id, .. }`), so including it would only ever re-insert an id already
  present.
- **`CardMilled`/`CardDiscarded`/`CardCycled` EXCLUDED** — these move a card into a graveyard from
  hand/library, never from the battlefield, and this arm's `WheneverCreatureDies` dispatch only
  ever fires on `CreatureDied`, so a card whose own `trigger_zone` ability needed that coverage
  does not exist in the corpus today (the `trigger_zone: Some(_)` population is exactly the 3 defs
  measured at stage 0/1). Filed as a candidate seed rather than widened (see Findings below).

**3.3** — new `fired_as: Option<TriggerEvent>` computation (renamed from a bare `fires: bool`,
since the push now needs to know WHICH `TriggerEvent` variant to record — the ETB arm and the new
death arm dispatch as different `TriggerEvent`s, and the push site is shared). New
`GameEvent::CreatureDied` arm added beside the existing `PermanentEnteredBattlefield` arm, mirrors
the battlefield `AnyCreatureDies` arm (`:4866-4916` per stage 0's cite, re-confirmed at
`:4828-4947` this session) clause for clause:

| clause | implementation |
|---|---|
| `controller_you`/`controller_opponent` | `*death_controller != owner` / `== owner`, per CR 108.4a (owner stands in for a graveyard card's absent controller) |
| `exclude_self` | `*new_grave_id == obj_id \|\| *pre_death_id == obj_id` (CR 400.7 — the graveyard-id comparison is the one that can match) |
| `nontoken_only` | `state.fizzle_object(*new_grave_id).is_some_and(\|o\| o.is_token)` |
| look-back | `arrived_in_graveyard_this_batch.contains(&obj_id)` (CR 603.10a) — **this arm only**, never the ETB arm |
| `filter` | `f.is_token` check + `matches_filter` against `pre_death_characteristics` (falling back to the graveyard object's base characteristics) |

**SR-25 bare-lookup ratchet caught a real gap in the first draft**: the first draft used bare
`state.objects.get(new_grave_id)` at two sites (the `nontoken_only` check and the `filter` `Some`
branch). `bare_lookup_ratchet::bare_lookup_counts_are_pinned` reddened (75 → 77). Fixed by
converting both to `state.fizzle_object(*new_grave_id)` — the SAME idiom the surrounding
`GameEvent::CreatureDied` arm ALREADY uses at every other LKI read in this file (`SelfDies`,
`SelfLeavesBattlefield`, Recover, Champion, Haunt all read via `fizzle_object`), so this is not a
new pattern, it is bringing two sites into line with the six that already surrounded them.

**Clippy caught a real style gap in the second draft**: the `if cond1 { false } else if cond2
{ false } else if cond3 { false } else if cond4 { false } else { <real logic> }` chain (four
identical `false` blocks) tripped `clippy::if_same_then_else` under `-D warnings`. Restructured
into four named `bool`s (`controller_blocks`, `exclude_self_blocks`, `nontoken_blocks`,
`lookback_blocks`) OR'd into a single `if`, each still carrying its own CR-cited comment. Re-ran
every per-clause revert (below) against this restructured shape to confirm nothing changed
semantically.

**3.4** — confirmed, by reading the source (not assumed): the `carddef_intervening_if_holds_at_queue_time`
call sits AFTER `let Some(triggering_event) = fired_as else { continue; };`, i.e. OUTSIDE the
`fired_as` match — it is shared, unedited code, and covers the new arm automatically. Read at
`abilities.rs:7342-7349` (post-edit line numbers).

**3.5** — the push: `triggering_event: Some(triggering_event)` (was hardcoded to
`AnyPermanentEntersBattlefield`; now varies per arm), `entering_object_id: entering_object`
(unchanged — supplied by the CALLER, both call sites pass the right value), `ability_index: idx`
(card-def index space, unchanged), `PendingTriggerKind::CardDefETB` (unchanged).

**3.6** — new call site added inside the `GameEvent::CreatureDied` arm of `check_triggers`,
immediately after the `AnyCreatureDies` block's closing brace (was `:4947`, confirmed still the
same boundary this session) and before the arm's own closing brace:
```rust
collect_graveyard_carddef_triggers(
    state, &mut triggers, event, Some(*new_grave_id), &arrived_in_graveyard_this_batch,
);
```
The pre-existing ETB call site (`:2988`) updated to pass `&arrived_in_graveyard_this_batch` too
(unused by that arm, since the look-back guard lives only inside the death arm — confirmed by
reading, matches §3.6's own note).

**3.7 resolution trace — walked in the source, and independently CONFIRMED by execution (T3)**:

1. `flush_pending_triggers`'s `once_per_turn` gate: `kind != Normal` (ours is `CardDefETB`) routes
   through the card-registry fallback, reading `def.effective_abilities(obj.is_transformed)
   [ability_index]` — confirmed at `abilities.rs` (line shifted from stage-0's `:8105-8151` cite
   due to the new arm's ~90 inserted lines; re-found by symbol, body unchanged). `is_transformed`
   is `false` for a graveyard object (never set true off the battlefield, confirmed by grepping
   every `is_transformed = true` assignment site — exactly one, `resolution.rs:853`, ETB-only).
   Nether Traitor's `once_per_turn: false`, so this is inert for the corpus today but structurally
   correct for a future card.
2. **CR 603.2d doubling — investigated, and it IS live-wrong, per plan risk #3. Filed as a
   finding, NOT fixed.** `doubler_applies_to_trigger`'s `TriggerDoublerFilter::CreatureDeath` arm
   matches on `trigger.triggering_event ∈ {SelfDies, AnyCreatureDies}` ONLY — no check on the
   trigger's SOURCE zone or whether the source is even a permanent. The doubler's OWN
   `source_active` guard checks the DOUBLER's battlefield presence, not the TRIGGER's. Since the
   new arm dispatches `AnyCreatureDies` (the SAME `TriggerEvent` the battlefield arm uses), a
   controller-matching `TriggerDoublerFilter::CreatureDeath` doubler WOULD double a graveyard-
   sourced Nether Traitor trigger. **Corpus exposure confirmed real, not hypothetical**:
   `grep -rl "TriggerDoublerFilter::CreatureDeath" crates/card-defs/src/defs/` returns
   `teysa_karlov.rs` and `drivnod_carnage_dominus.rs`. Teysa Karlov's printed text is "a
   triggered ability of a **permanent you control**" — a graveyard card is not a permanent, so
   doubling here has no CR warrant. This is squarely the plan's own risk #3
   ("Almost certainly wrong if unscoped; investigate, file, do not fix here") — recorded as a
   finding for the stage-7 seed file (out of scope this invocation).
3. Stack object dispatch: `PendingTriggerKind::CardDefETB` flows through the SAME `flush_sorted`
   match as the pre-existing graveyard ETB triggers — no new arm needed, confirmed by reading (no
   `StackObjectKind` variant added; T3's debug trace showed `StackObjectKind::TriggeredAbility {
   is_carddef_etb: true, ability_index: 2, .. }` on the stack, matching Bloodghast's shape).
4. `resolution.rs`'s `is_carddef_etb` branch reads
   `def.effective_abilities(obj.is_transformed).get(ability_index)` — **directly confirmed by
   execution**: T3's positive case (1 black mana floating) drove the trigger through
   `check_and_apply_sbas` → `flush_pending_triggers` → full stack resolution, and Nether Traitor
   ended up on the battlefield; the paired negative (0 black mana) left it in the graveyard,
   proving the assertion discriminates the RETURN (the `MayPayThenEffect`'s `then` arm), not
   merely the trigger firing.

### A real T3 fixture bug, found and fixed during authoring (recorded per the "runner obligation"
to report reality, not force a green run)

T3's first draft drove the death via one `pass_all` round (4 `PassPriority` calls) before
`drain_stack`, matching the pattern used elsewhere in this file. It reddened: Nether Traitor
stayed in the graveyard even with `{B}` floating. Root-caused with a throwaway debug test (never
committed): `check_and_apply_sbas` (and therefore the death SBA) is invoked inside `enter_step`,
NOT on every bare `PassPriority` — so a fixture that starts with an EMPTY stack and nothing else
happening advances the STEP first (CR 500.4: mana pools empty at a step boundary) via
`handle_all_passed`'s stack-empty branch, and only the RESULTING `enter_step` call runs SBA. The
mana was cleared a full step before the death (and its trigger) ever got a chance to spend it.
Fixed by calling `check_and_apply_sbas(&mut state)` + `flush_pending_triggers(&mut state)`
DIRECTLY (bypassing the step-boundary path entirely, matching T2's/T4's/T5's/T6's own idiom, which
never hit this because they inspect `pending_triggers` before any stack resolution), THEN
`drain_stack` to resolve. This is a test-fixture-design finding, not an engine defect — recorded
here because the instructions require reporting fixture bugs found along the way, not just
engine bugs.

### T4/T5's fixture also had a card_id bug, found and fixed via the SAME debug methodology

T4 and T5 both build Nether Traitor starting ON THE BATTLEFIELD (dying via SBA to reach the
graveyard, rather than starting in the graveyard like T2/T3/T6). Their first drafts used
`enrich_spec_from_def(ObjectSpec::card(p1, "Nether Traitor").in_zone(Battlefield), &defs)`
WITHOUT `.with_card_id(...)`. `collect_graveyard_carddef_triggers` looks the def up via
`obj.card_id` (`Some(card_id) = card_id_opt else { continue }`), and `card_id` is apparently NOT
auto-populated by `enrich_spec_from_def` for a battlefield object (only the CHARACTERISTICS —
types, P/T, keywords, abilities lowered onto `characteristics.triggered_abilities` — are
populated; the raw `card_id` field, needed for the SEPARATE graveyard-registry lookup path, is
not). **Both T4 and T5 passed VACUOUSLY in this state** — not because the guards under test were
correct, but because NEITHER graveyard object (after SBA moved them) had a `card_id`, so
`collect_graveyard_carddef_triggers`'s `continue` on `card_id_opt.is_none()` skipped BOTH of them
entirely, and `pending_triggers` stayed `[]` regardless of any guard's correctness. **Found by a
throwaway debug trace that printed every object's `card_id` after `check_and_apply_sbas`** — both
showed `card_id: None`. Fixed by adding `.with_card_id(nether_card_id)` to both fixtures'
`ObjectSpec::card(...)` chain (mirroring the pattern Bloodghast's own graveyard test already uses,
and the one T2/T3/T6 already had right).

**This fixture bug directly falsified an earlier, WRONG conclusion drawn mid-authoring**: with the
bug still present, T4's "disable the look-back guard" revert did NOT redden T4 (both objects were
invisible to dispatch regardless of the guard), which was initially misread as "the look-back
guard is redundant with `exclude_self` for a self-death scenario." **Re-executed after fixing the
card_id bug**: the SAME revert (disabling `lookback_blocks`) NOW correctly reddens T4 —
```
CR 603.10a: Nether Traitor and another creature dying in the SAME batch must NOT trigger Nether
Traitor -- it was on the battlefield immediately prior. Got 1 trigger(s): [PendingTrigger { source:
ObjectId(3), ... triggering_event: Some(AnyCreatureDies), entering_object_id: Some(ObjectId(4)) ...}]
```
confirming the look-back guard DOES independently matter and is NOT redundant. Restored
immediately after (`git diff` confirmed clean).

### T5's genuine (not fixture-bug-caused) non-discrimination — recorded honestly

With the card_id bug fixed, T5's OWN revert (`exclude_self` compared against `pre_death_id` alone,
dropping the `new_grave_id` term) was RE-EXECUTED and still did NOT redden T5. This is a real,
proven-by-execution architectural fact, not a residual fixture bug: for a GRAVEYARD-dispatched
`WheneverCreatureDies` trigger, `new_grave_id == obj_id` can only be true when the trigger's OWN
source is the object that just died THIS batch — and since `arrived_in_graveyard_this_batch` is
built from the SAME `events` slice `collect_graveyard_carddef_triggers` is invoked per-event from,
that exact id is ALWAYS already a member of the look-back set by construction. The two guards are
therefore logically overlapping for every state reachable through the public API, for THIS
corpus's only `exclude_self: true` graveyard-scoped card (Nether Traitor). The `new_grave_id`
comparison is kept in the source anyway (still the CR 400.7-correct comparison, and
defense-in-depth against a future narrowing of the look-back guard's scope — matching the ETB
arm's pre-existing, identically "moot but kept for symmetry" `exclude_self` check). T5's docstring
was rewritten to state this plainly rather than claim a revert-discrimination that does not exist;
see the source-level comment at the `exclude_self` clause for the CR 400.7 citation, which is the
level at which this comparison's correctness is actually verified.

**T4's and T6's per-clause reverts, all executed (rebuild confirmed each time) against the FINAL
(clippy-clean, restructured) shape of the code**:

| test | revert | observed failure |
|---|---|---|
| T4 (look-back) | `lookback_blocks = false && arrived_in_graveyard_this_batch.contains(&obj_id)` | `Got 1 trigger(s)... triggering_event: Some(AnyCreatureDies)...` (see above) |
| T6 (controller_you) | `controller_blocks` computed with `false && controller_you && ...` for the first disjunct | `CR 108.4a: an opponent's creature dying must NOT trigger Nether Traitor... Got 1 trigger(s)...` |

Both restored immediately; `git diff -- crates/engine/src/rules/abilities.rs \| grep -c "false &&"`
confirmed `0` before continuing each time.

T2, T3, and the two synthetic sub-probes (nontoken_only, subtype filter) are proven by the
stage-4a fail-before capture (the whole new dispatch arm did not exist; deleting it entirely, which
is what the pre-implementation tree literally was, reddens all four identically) — not re-executed
as a narrower per-clause revert in this pass, given the effort budget; this is recorded as a
scope note rather than silently treated as equivalent.

### Findings recorded (not fixed — out of this invocation's scope)

1. **CR 603.2d doubling of a graveyard-sourced `WheneverCreatureDies` trigger is live-wrong** on
   two real corpus cards (`teysa_karlov.rs`, `drivnod_carnage_dominus.rs`), per the trace above.
   Candidate for `OOS-DX24-n` at stage 7.
2. **The batch-arrival-set scope decision** (excluding `PermanentSacrificed`,
   `CardMilled`/`CardDiscarded`/`CardCycled`) is a corpus-zero-exposure decision today but is a
   real, stated scope boundary — candidate for a seed noting the deliberate exclusion if a future
   card pairs `trigger_zone: Graveyard` with a mill/discard-reachable trigger condition.
3. **`squee_goblin_nabob` remains broken** (per plan §5's explicit instruction to say so): its
   `AtBeginningOfYourUpkeep` + `trigger_zone: Graveyard` pair has neither a lowering arm nor a
   dispatch arm; this batch does not touch it, and it is `known_wrong`/deck-illegal, so this is
   not a regression.
4. **The `nether_traitor.rs` comment-only card-def edit (plan §5) was NOT done in this
   invocation.** It is not named as an action item under any of stages 1-4 in the plan's own §9
   stage list (it falls naturally under stage 7's close-out, alongside the coverage-regeneration
   proof in plan §8), and the task brief's explicit scope is stages 1-4 only. Flagged here rather
   than silently done or silently skipped.

---

## Final verification for stages 1-4 (this invocation's close)

- `cargo build --workspace`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean (after the `if_same_then_else`
  restructure above — the only clippy finding this batch produced).
- `cargo fmt --check`: clean (after one `cargo fmt` run — auto-wrapped a handful of lines in both
  the engine source and the new test file; no semantic change, confirmed by re-running the full
  test suite after).
- `cargo test --workspace --no-fail-fast`: **4,422 / 0 / 5** (+9 over the 4,413 stage-0 baseline:
  T1, T2, T3, T4, T5, T6, the 2 synthetic sub-probes, T7 — residual list empty).
- `cargo test -p mtg-engine --test core protocol_schema` / `--test core hash_schema`: both green,
  **PROTOCOL 35 / HASH 73 both gate-confirmed unmoved** (no wire type touched — no `Command` /
  `GameEvent` / `Effect` variant added, `TriggeredAbilityDef` gained no field).
- `cargo test -p mtg-engine --test core keyword_registry`: green, unmoved (SR-5 — nothing here is
  a `KeywordAbility`).
- `cargo test -p mtg-engine --test core bare_lookup_ratchet`: green, **75 unmoved** (the two new
  bare-lookup-shaped reads were written as `fizzle_object` calls from the start of the fix, per
  the surrounding arm's own idiom — the ratchet only reddened once, transiently, during a stage-4
  intermediate draft, and was fixed before landing; see above).
- `git diff main..HEAD --numstat -- crates/card-defs/`: **empty** (no card-def edit in this
  invocation, per the finding above).
- `git diff main..HEAD --numstat -- crates/card-types/`: **empty** (no DSL type change).
- `git diff main..HEAD --numstat -- crates/simulator/ tools/`: **empty**.

**Not run in this invocation** (deferred to stage 6/7 per scope): `tools/check-defs-fmt.sh` (no
card-def edit to check), `python3 tools/authoring-report.py` (no coverage-affecting change),
`docs/audits/decision-point-audit.md` seed filing.
