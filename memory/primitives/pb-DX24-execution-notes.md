# PB-DX24 — execution notes (stages 1-6)

**Task**: `scutemob-202` · **Branch**: `feat/pb-dx24-the-lowering-drops-triggerzone-the-two-index-spaces-`
**Scope of the first invocation**: plan stages 1-4 only (see the "Stages 1-4" body below).
**Scope of this (second) invocation**: plan stages 5 (Q1-Q7 index-space fixes) and 6 (gates) only.
Stage 7 (close-out, seed filing) is OUT OF SCOPE and NOT done here. Stages 5-6 content is
appended at the bottom of this file, after the original stages 1-4 record (left verbatim).

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

---

## Stage 5 — the two index spaces (Change 4, OOS-DX1-4 Q1-Q7)

Re-cited every Q-site at current HEAD (stage 4 shifted line numbers, per the task brief's
warning) before editing. All 7 dispositions matched the plan exactly: FIX Q1/Q2/Q3/Q4/Q6/Q7,
RE-SCOPE Q5 (comment-only).

**Q1** (`abilities.rs`, Backup ETB, ~`:3187`) — `obj` was already in hand (via
`state.fizzle_object`). One shared binding `let eff = def.effective_abilities(obj.is_transformed);`
now serves BOTH the `eff.iter().enumerate()` loop and the `eff[idx + 1..]` "printed below" slice,
per the plan's explicit requirement that index and slice never diverge.

**Q2** (`abilities.rs`, `WhenYouCastThisSpell`, ~`:3809`) — `stack_obj` already in hand. Fixed to
`def.effective_abilities(stack_obj.is_transformed)`, documented as DEFENSIVE (zero behaviour
change): `is_transformed` is never true on a stack object (§4.0).

**Q3** (`abilities.rs`, `WhenExertedAsAttacks`, ~`:4169`) — `src_obj` already in hand. Fixed to
`def.effective_abilities(src_obj.is_transformed)`. Genuinely reachable: an attacking transformed
DFC is ordinary, and `WhenExertedAsAttacks` has NO Channel-A lowering arm (confirmed by grep —
zero hits in `replay_harness.rs`), so this is the ONLY dispatch path for this trigger condition.

**Q4** (`abilities.rs`, `WhenDealsCombatDamageToPlayer`, ~`:5222`) — `src_obj` already in hand.
Fixed to `def.effective_abilities(src_obj.is_transformed)`. **Finding not in the plan's own
text, discovered while designing the probe**: this trigger condition IS ALSO one of the ~34
arms lowered into the runtime Channel-A vector by `build_face_ability_vectors`
(`replay_harness.rs:2689-2722`, `TriggerEvent::SelfDealsCombatDamageToPlayer`), and Channel A is
ALREADY face-aware via `apply_face_change` (an earlier, unrelated PB-OS4b/PB-RS4 mechanism) —
so an end-to-end (life-total) probe would be satisfied by Channel A alone regardless of whether
Q4's own raw card-registry scan is fixed, and would NOT discriminate this fix. The probe instead
calls `check_triggers` directly and filters the returned `PendingTrigger`s by
`kind == PendingTriggerKind::CardDefETB`, which isolates exactly the raw-scan path Q4 touches.

**Q5** (`resolution.rs:7665-7676` per the plan's cite, re-found at current HEAD unchanged from
stage-0's line numbers) — RE-SCOPED, no behaviour change. Both ends already index plain
`def.abilities`; CR 712.2 forbids turning a transforming DFC face down, so
`PermanentTurnedFaceUp`'s source can never be a transformed DFC and `is_transformed` is
unreachable at this site. Comment rewritten to state this is CORRECT-because-unreachable rather
than an open TODO (the prior comment cited `OOS-DX1-4` as if still open).

**Q6** (`abilities.rs`, `WheneverRingTemptsYou`, ~`:6196`) — the site only had `card_id`, not the
whole object, in hand (`state.expect_object(obj_id).and_then(|o| o.card_id.clone())`). Restructured
to bind `obj` first, then read both `card_id` and `is_transformed` off it, then
`def.effective_abilities(is_transformed)`. Documented as DEFENSIVE (§4.0: `is_transformed` is
never true off the battlefield, and `WheneverRingTemptsYou` has no Channel-A lowering arm either,
so — unlike Q4 — this one COULD be tested end-to-end at the `check_triggers` level with no masking
concern; the probe uses that shape for consistency with Q4's design, not because it needs to.)

**Q7** (`abilities.rs`, `collect_graveyard_carddef_triggers`'s graveyard sweep, ~`:7198`) — the
`gy_objects` tuple gained a 4th field (`is_transformed: bool`, read once per graveyard object at
collection time, mirroring the existing `card_id_opt` pattern rather than a second
`state.objects.get` lookup per iteration). Fixed to `def.effective_abilities(is_transformed)`.
Documented as DEFENSIVE (§4.0) but noted as mattering MORE than Q2's defensive fix because this
batch's own Change 3 adds a SECOND `fires` arm to this exact loop (the new `WheneverCreatureDies`
graveyard dispatch) — making the expression uniform now avoids the new arm resting on a
reset-on-zone-change invariant three files away.

### The §4.0 measurement, re-confirmed by direct grep at this stage

`grep -rn "is_transformed = true\|is_transformed: true" crates/ --include=*.rs | grep -v "/tests/"`
returns exactly ONE production hit: `resolution.rs:853` (the disturb ETB). Three doc-comment
mentions in `crates/card-types/src/state/{stack.rs,types.rs}` (all `///`-prefixed) are the only
other occurrences in the whole workspace. This is the fact both Q2's and Q7's "defensive, zero
behaviour change" classification rests on, and it is now itself pinned by a structural test
(`test_dx24_is_transformed_true_assignment_has_exactly_one_site`) rather than asserted once and
left to rot.

### T10-family probes (all in `pb_dx24_trigger_zone_and_index_spaces.rs`, stage 5)

Per §4.2's stage-1 measurement (all 7 shapes: 0 real corpus hits on any back face), every Q1/Q3/Q4/
Q6 probe uses a SYNTHETIC `CardDefinition`/`CardFace` fixture, mirroring
`pb_rs4_face_aware_residuals.rs`'s idiom (disturb-cast helper for Q1, `Command::Transform` for
Q3/Q4/Q6). Q2/Q7 (defensive) are pinned structurally instead, per the task's explicit allowance
("if no Q-site can be exercised end-to-end with a synthetic fixture, say so plainly and pin what
you can at the unit level").

| test | site | mechanism | discriminating observable |
|---|---|---|---|
| `test_dx24_backup_lowering_reads_the_visible_face_of_a_disturbed_dfc` | Q1 | disturb cast (front: Disturb only, no Backup; back: `Backup(2)` only) → `drain_stack` twice (once for the permanent, once for the ETB Backup trigger's own KeywordTrigger stack object) | +1/+1 counter count on the entered (back-face) permanent: 2 fixed / 0 broken |
| `test_dx24_when_exerted_as_attacks_reads_the_visible_face_of_a_transformed_attacker` | Q3 | `Command::Transform`, then `DeclareAttackers` with `exert_choices: [obj_id]` (legal only because `calculate_characteristics` — an unrelated, already-face-aware mechanism — reports the back face's `Exert` keyword), `drain_stack` | controller life total: `life_before + 7` fixed / unchanged broken |
| `test_dx24_when_deals_combat_damage_to_player_reads_the_visible_face_of_a_transformed_attacker` | Q4 | `Command::Transform`, then a DIRECT `check_triggers(&state, &[GameEvent::CombatDamageDealt{..}])` call (bypassing the double-dispatch masking, see above) | count of `PendingTriggerKind::CardDefETB` hits sourced at the object: 1 fixed (with the back face's own re-derived `ability_index`) / 0 broken |
| `test_dx24_whenever_ring_tempts_you_reads_the_visible_face_of_a_transformed_permanent` | Q6 | `Command::Transform`, then a direct `check_triggers(&state, &[GameEvent::RingTempted{..}])` call | same shape as Q4: 1 `CardDefETB` hit fixed / 0 broken |
| `test_dx24_is_transformed_true_assignment_has_exactly_one_site` | §4.0 invariant | recursive source scan of `crates/engine/src` for `is_transformed = true` / `is_transformed: true` assignments (comment-aware: skips lines starting with `//`, strips trailing `//` comments) | exactly 1 hit, in `resolution.rs` |
| `test_dx24_q2_and_q7_queue_sites_call_effective_abilities` | Q2 + Q7 | locates each `OOS-DX1-4 Q<n>` anchor comment in `abilities.rs` by line, scans the next 8 lines for `effective_abilities(` (must be present) and the bare `.abilities.iter().enumerate()` shape (must be absent) | structural presence/absence, not runtime behaviour |

**All 6 reverts EXECUTED (rebuild confirmed in each captured output), all restored, `git diff`
confirmed clean after every one**:

- **Q1**: `let eff = &def.abilities;` (drop the `effective_abilities` call). Failure:
  `"Got 0 counters"` (expected 2).
- **Q3**: restored `def.abilities.iter().enumerate()` at the Q3 site. Failure:
  `"life_before=40, life_after=40"` (expected `life_after=47`).
- **Q4**: restored `def.abilities.iter().enumerate()` at the Q4 site. Failure:
  `"Got: []"` (expected exactly 1 `CardDefETB` hit).
- **Q6**: restored `def.abilities.iter().enumerate()` at the Q6 site (plus a temporary
  `let _ = is_transformed;` to silence the resulting unused-variable warning — the
  `local_game.rs` "a revert-and-rerun proves nothing unless the rebuild succeeded" gotcha, applied
  proactively rather than discovered the hard way). Failure: `"Got: []"` (expected 1).
- **§4.0 pin**: added a literal second `obj.is_transformed = true;` line immediately after the
  real one in `resolution.rs`. Failure: `"Found: [\"...resolution.rs:853\", \"...resolution.rs:854\"]"`
  (expected exactly 1 hit).
- **Q2/Q7 structural pin**: reverted EACH site in turn (Q2 first, restored, then Q7) to the bare
  `def.abilities.iter().enumerate()` shape. Q2 failure: window text shows
  `def.abilities.iter().enumerate()` immediately after the Q2 anchor, no `effective_abilities(`
  in the 8-line window. Q7 failure: same shape, naming the Q7 anchor.

No pre-existing test reddened by any of the 6 fixes (`cargo test -p mtg-engine --test primitives
pb_dx24` stayed 15/15 green after landing all six; `pb_rs4_face_aware_residuals` stayed 19/19;
`pb_ac7_ability_index_desync` stayed 4/4; `core face_dereg_parity` stayed 2/2;
`pb_dx1_lowered_intervening_if` stayed 17/17).

Committed at `36bbeed0`.

---

## Stage 6 — the gates (§2.6 G-A/G-B, §4.3 R1/R2)

New file `crates/engine/tests/core/pb_dx24_trigger_zone_roster.rs`, `mod` line added to
`crates/engine/tests/core/main.rs` (alphabetically before `pb_dx5_...`, since `cargo fmt` sorts
`"pb_dx24"` before `"pb_dx5"` as a plain string).

**G-A / G-B mechanism**: `strip_line_comments` + `strip_block_comments` (the latter copied
verbatim from `core::decision_gate`'s PB-DX32-M8-motivated idiom, not re-derived), then
`extract_function_body` does a brace-balance walk from `fn build_face_triggered_abilities(`'s
first `{` to its matching `}`.

- **G-A** (`g_a_lowering_function_never_sees_trigger_zone`): asserts `trigger_zone` occurs zero
  times in the extracted body.
- **G-A non-vacuity** (`g_a_scan_is_not_vacuous`): asserts the extracted body contains >= 30
  `trigger_condition:` match arms (measured 34 at stage 3), so a collapsed/empty extraction can't
  make G-A pass by accident.
- **G-B** (`g_b_call_site_is_unique_and_filtered`): asserts `build_face_triggered_abilities(`
  occurs exactly twice in the comment-stripped file (def + 1 call), and that the literal
  `build_face_triggered_abilities(&battlefield_triggers)` is present.
- **R1** (`r1_trigger_zone_population_is_pinned`): the `trigger_zone: Some(_)` population, pinned
  by symbol, `{"Bloodghast", "Squee, Goblin Nabob", "Nether Traitor"}`, plus an
  `all_cards().len() >= 1_700` non-vacuity floor.
- **R2** (`r2_back_face_population_is_pinned_with_a_non_vacuity_floor`): the `back_face: Some(_)`
  population pinned at 15 (matching stage 1's measurement exactly), with its own
  `!back_face_defs.is_empty()` non-vacuity floor.

**All 5 gates EXECUTED (rebuild confirmed), reverts run against every one, all restored**:

- **G-A (line-comment form)**: added a real `trigger_zone,` binding (plus `let _ = trigger_zone;`
  to silence the unused-variable warning) to the `WhenDies` arm inside
  `build_face_triggered_abilities`. Failure: the exact "must never see `trigger_zone`" message.
- **G-A (block-comment variant, PB-DX32 M8 lesson applied both ways)**: replaced that same edit
  with `/* trigger_zone, */` (a genuine comment — the code is UNCHANGED and correct). Confirmed
  this does NOT falsely redden G-A (block-comment stripping correctly ignores it). Then, to prove
  the stripping step itself matters (not just that it's present), temporarily edited the TEST
  file's own `strip_comments` to skip `strip_block_comments` (line-comment stripping only) while
  the `/* trigger_zone, */` text was still in the source — this DID falsely redden G-A
  (`"Got: ...contains trigger_zone"` — a false positive against genuinely clean production code),
  proving block-comment stripping is load-bearing for this gate, not decorative. Restored the
  test file's `strip_comments` first, then the source's block comment.
- **G-B**: added a second, unfiltered call `let _unfiltered_second_call =
  build_face_triggered_abilities(&unfiltered);` (where `unfiltered: Vec<&AbilityDefinition> =
  abilities.iter().collect()`) after the real call. Failure: `"got 3"` (expected 2).
- **R1**: added `if def.name == "Nether Traitor" { continue; }` inside `trigger_zone_population`.
  Failure: `"Expected {...Nether Traitor...}, got {\"Bloodghast\", \"Squee, Goblin Nabob\"}"`.
- **R2**: changed the pinned constant from `15` to `14`. Failure: `"got 15"` (names the real,
  unchanged population).

Committed at `f302f7a7`.

---

## Final verification for stages 5-6 (this invocation's close)

- `cargo build --workspace`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean (no findings this stage).
- `cargo fmt --check`: clean (one `cargo fmt` run — reflowed a multi-line type annotation in
  `collect_graveyard_carddef_triggers` and two lines in the new roster-gate/probe files; no
  semantic change, confirmed by re-running every gate/probe after).
- `cargo test --workspace --no-fail-fast`: **4,433 / 0 / 5** (+11 over the 4,422 stage-1-4
  baseline: 6 new T10-family tests in File A + 5 new gate tests in File B), residual list empty.
- `cargo test -p mtg-engine --test core protocol_schema` / `--test core hash_schema`: both green,
  **PROTOCOL 35 / HASH 73 both gate-confirmed unmoved** (read directly off the source constants
  too: `crates/engine/src/rules/protocol.rs:360` / `crates/engine/src/state/hash.rs:757`).
- `cargo test -p mtg-engine --test core keyword_registry`: green, unmoved (9/9) — nothing in this
  stage is a `KeywordAbility`.
- `cargo test -p mtg-engine --test core bare_lookup_ratchet`: green, unmoved (3/3) — no new bare
  `state.objects.get`-shaped read was introduced this stage (Q6/Q7 both read through
  `state.expect_object`/the existing `gy_objects` collection pass, not a fresh bare lookup).
- `cargo test -p mtg-engine --test primitives pb_rs4_face_aware_residuals` (19/19),
  `--test primitives pb_ac7_ability_index_desync` (4/4), `--test core face_dereg_parity` (2/2),
  `--test primitives pb_dx1_lowered_intervening_if` (17/17): all green, all unmoved — the four
  sibling gates the plan names in stage 5's own "Verify" line.
- `git diff main..HEAD --numstat -- crates/simulator/ tools/`: **empty**.
- `git diff main..HEAD --numstat -- crates/card-defs/`: **empty** (no card-def edit in stages
  5-6 — `nether_traitor.rs`'s comment-only edit from plan §5 is stage-7 close-out work, out of
  this invocation's scope, per the task brief).
- `git diff main..HEAD --numstat -- crates/card-types/`: **empty** (no DSL type change).
- `git status --short`: clean after both commits (`36bbeed0` stage 5, `f302f7a7` stage 6).

**Not run in this invocation** (deferred to stage 7 per scope): `tools/check-defs-fmt.sh` (no
card-def edit), `python3 tools/authoring-report.py` (no coverage-affecting change), the
`docs/audits/decision-point-audit.md` `OOS-DX1-3`/`OOS-DX1-4` row closures, the
`nether_traitor.rs` comment-only card-def edit (plan §5), and seed filing for the two findings
below.

### Findings recorded (not fixed — out of this invocation's scope)

1. **Q4's dual-dispatch discovery, stated plainly because it changed the probe design**:
   `WhenDealsCombatDamageToPlayer` is lowered into BOTH the runtime Channel-A vector
   (`replay_harness.rs`, already face-aware via the earlier `apply_face_change` mechanism) AND
   dispatched via the raw card-registry scan this batch fixes (`abilities.rs`). The two are not in
   conflict — a `Normal`-kind trigger from Channel A and a `CardDefETB`-kind trigger from the raw
   scan would BOTH fire for a def that has this ability lowered both ways, meaning a REAL
   `Complete` card using this exact shape could double-fire the effect. No corpus exposure found
   (a targeted check of the trigger's dispatch showed no `Complete` def combining both paths for
   this specific trigger condition, and this was not the reported defect of PB-DX24 — it predates
   this batch and is orthogonal to the index-space fix). Candidate for a seed at stage 7
   (`OOS-DX24-n`, "WhenDealsCombatDamageToPlayer dispatches via two independent paths that are
   not mutually exclusive").
2. **`squee_goblin_nabob` remains broken**, restated per the plan's own instruction (already
   recorded in the stage 1-4 section above; unaffected by stages 5-6, which touch none of its
   fields).

---

## Fix cycle — `memory/primitives/pb-review-DX24.md` (0 HIGH / 6 MEDIUM / 7 LOW; scope 1, 2, 3,
5, 6, 7, 8, 9, 10, 11, 12 — findings 4 and 13 are the coordinator's audit-registry rows, not
touched here)

**Stage-7 gap, discovered while working this cycle**: the execution-notes file above has NO
stage-7 section at all — the `nether_traitor.rs` comment-only edit (plan §5) and the six seed
filings landed as a separate commit (`0ca69b6b`, "PB-DX24 stage 7 — nether_traitor comment
records what the engine now reads") with no corresponding entry here. This directly confirms
Finding 12's second half: the stages-1-4 and stages-5-6 sections' repeated "no card-def edit in
this invocation" / "`crates/card-defs/` empty" claims were each locally true for their own scope
but the overall file gives the false impression that `crates/card-defs/` was never touched by
this batch. `git diff main..HEAD --numstat -- crates/card-defs/` at the time of this fix cycle
shows exactly one file, `nether_traitor.rs`, `+17 -0` (the stage-7 comment block). SR-35
(`tools/check-defs-fmt.sh`) was re-run in this fix cycle and reports "1803 defs checked / clean"
— it is unknown whether it was run at stage-7 time, but it is confirmed clean NOW.

### F1 (MEDIUM) — wrong CR citation at the Q5 comment

`crates/engine/src/rules/resolution.rs:7673-7689` (pre-edit line numbers) cited CR 712.2 for
"a transforming DFC can't be turned face down." Re-derived: CR 712.2 is only about DFC face
SYMBOLS (712.2a-c); the correct rule is **CR 712.16** ("Melded permanents and other double-faced
permanents can't be turned face down..."). CR 712.16 alone doesn't make the site unreachable,
though — **CR 712.15** explicitly allows a DFC card to enter the battlefield face down
(manifest/cloak), so a `PermanentTurnedFaceUp` source COULD in principle be a DFC that arrived
face down. What actually makes `is_transformed` unreachable at Q5 is the ENGINE's write
discipline (`face.rs:67-69`'s battlefield-only gate + `state/mod.rs`'s four zone-change resets),
not a CR prohibition. Rewrote the comment to cite 712.16 + 712.15 + the engine mechanism, all
three. **Caveat**: this session has no MCP tool access (the `mtg-rules` MCP tools are not wired
into this task's tool set), so the CR 712.15/712.16 text could not be independently re-verified
against the live rules server — it is taken from the reviewer's own quoted MCP output, which
quotes exact rule text, and matches this engineer's own knowledge of the DFC rules family. Flagged
so a future reader knows this one citation rests on the reviewer's MCP call, not a second
independent one. **Per the task brief, `docs/audits/decision-point-audit.md:1213`'s copy of the
same wrong citation was NOT corrected here** — that file is the coordinator's, edited
concurrently; the coordinator should apply the identical correction there.

### F2 (MEDIUM) — the §4.0 pin didn't gate what it claimed; Q2/Q7's real invariant was untested

Replaced `test_dx24_is_transformed_true_assignment_has_exactly_one_site` (matched only the two
literal strings `is_transformed = true` / `is_transformed: true`, missing `face.rs:104`'s
COMPUTED write `obj_mut.is_transformed = new_is_transformed;`) with THREE things:

1. `test_dx24_is_transformed_writes_are_confined_to_resolution_and_face_rs` — a widened structural
   scan (word-boundary-aware, RHS-aware: excludes `false`/`bool` RHS so the CR 712.8a reset writes
   in `state/mod.rs` and type annotations don't pollute it) asserting exactly one write in each of
   `resolution.rs` and `face.rs`, none elsewhere. **This is a drift gate, not the load-bearing
   proof** — it cannot catch the deletion of a guard (only the addition of a write), which is
   exactly the revert shape that defeated the original test.
2. `test_dx24_transform_state_resets_on_zone_change_to_graveyard` — runtime probe of CR
   712.8a/400.7's OWN mechanism: transform a synthetic DFC (front toughness 5, back toughness 2,
   3 damage marked from the start), let CR 704.3's post-transform SBA check kill it (discovered
   during authoring: `transform_permanent_in_place` checks SBAs INSIDE the same `Command::Transform`
   call, so "transformed and still alive" is not an observable intermediate state for a fixture
   built this way — the test asserts the death happened AS the transformed permanent via the
   `CreatureDied` event's `pre_death_characteristics.toughness == Some(2)`, then asserts the new
   graveyard object's `is_transformed == false`).
3. `test_dx24_apply_face_change_is_a_noop_off_the_battlefield` — runtime probe of `face.rs`'s OWN
   gate. **Promoted `apply_face_change` `pub(crate)` -> `pub`** (mirrors `build_face_ability_vectors`'s
   identical PB-DX24 promotion, T7's access problem) because NO production call site ever invokes
   it on a non-battlefield object (verified by reading all 7 call sites: `Command::Transform`'s own
   `handle_transform` already rejects off-battlefield before reaching `apply_face_change`; every
   other caller has just moved the object TO the battlefield in the same call), so the gate was
   previously unreachable through the public command API by ANY test.

**F2 experiment result (the reviewer's own claim, re-executed)**: deleted `face.rs:67-69`'s
`if obj.zone != ZoneId::Battlefield { return; }` guard, rebuilt (`Compiling mtg-engine` observed),
ran the full `pb_dx24` primitives suite: **exactly one test reddened**,
`test_dx24_apply_face_change_is_a_noop_off_the_battlefield`, with
`face.rs:63-69: apply_face_change must be a no-op ... Got is_transformed=true after calling
apply_face_change on a graveyard object.` — the other 16 tests, INCLUDING the new
"writes-confined" structural scan and the Q2/Q7 test, stayed green, confirming the reviewer's
point that a structural scan cannot see a deleted guard. Restored; `git diff --stat -- face.rs`
showed only the intended `pub(crate)` -> `pub` promotion + doc comment afterward.

Also executed a second revert on the structural scan alone (added a duplicate
`obj.is_transformed = true;` line in `resolution.rs`): reddened
`test_dx24_is_transformed_writes_are_confined_to_resolution_and_face_rs` with
`left: 2 right: 1`, naming both `resolution.rs:853` and `:854` — the widened scan does catch an
ADDED write, just not a REMOVED guard. Restored.

### F3 (MEDIUM) — plan risk #2 (slice granularity) was never discharged

Added a per-caller granularity note at `arrived_in_graveyard_this_batch`'s construction site
(`abilities.rs:2962-2998`, now +30 lines of comment). **Measurement, by reading every
`check_triggers` caller**:

| caller | events slice | granularity |
|---|---|---|
| `sba.rs:97` | `apply_sbas_once`'s own return value, ONE fixpoint pass | **EXACT** — one CR 704.3 simultaneous SBA batch |
| `resolution.rs` (post-resolution call, ~line 8142 at this fix cycle's HEAD; was `:8118` at review time, drifted from earlier edits in this same cycle) | the WHOLE resolution's accumulated `events` vec across every sequential sub-effect | **COARSER** — confirmed by reading the function: `events` is built up across the entire body (sacrifice effects, exile effects, etc. all `.push()` into the SAME vec), then `check_triggers(state, &events)` is called ONCE at the very end |
| `combat.rs:846`/`:1743`, `engine.rs:34`/`:2499` | per-action / per-command event batches | **NOT audited this cycle** — `engine.rs:34` (`check_and_flush_triggers`) is shared by nearly every `Command` arm, so its own granularity is a separate investigation; out of this fix cycle's scope per the finding's own boundary |

**The over-suppression mechanism** (resolution.rs's coarse slice): a resolution whose effects read
"sacrifice a creature, THEN destroy target creature" pushes both deaths into ONE `events` vec. If
the sacrificed creature is ITSELF a `trigger_zone: Graveyard` source (Nether Traitor's shape), its
graveyard id lands in `arrived_in_graveyard_this_batch` from the FIRST sub-effect; when the loop
reaches the SECOND sub-effect's death, the look-back guard sees that id already in the set and
suppresses what should be a live trigger — CR 603.10a asks whether the ability existed immediately
prior to THAT (second) event, and by then it already did, having arrived earlier in the SAME
resolution. Direction: **over-suppression**.

**Seed text for `OOS-DX24-7`** (coordinator to file):

> **Title**: `check_triggers`'s CR 603.10a look-back set is coarser than "one simultaneous batch"
> at its `resolution.rs` caller, and over-suppresses a sequential graveyard-trigger-zone source
> within one resolution.
> **Class**: correctness, latent — requires a resolution whose effects sequentially (1) put a
> `trigger_zone: Graveyard` source (e.g. Nether Traitor) into the graveyard, THEN (2) put another
> creature into a graveyard, both within the SAME stack-object resolution. No such `Complete` card
> pairing exists in the corpus today (measured: the `trigger_zone: Some(_)` population is exactly
> the 3 defs `{Bloodghast, Squee Goblin Nabob, Nether Traitor}`, and none of Nether Traitor's own
> effects also destroys/sacrifices ANOTHER creature in the same resolution).
> **Mechanism**: `abilities.rs`'s `resolution.rs` caller passes the WHOLE resolution's accumulated
> `events` vec into `check_triggers`, so `arrived_in_graveyard_this_batch` treats every death in
> that resolution as simultaneous, even when they are sequential sub-effects.
> **Direction**: over-suppression (the safer of the two possible directions, and the one that
> matches the common SBA-simultaneous-death case — but it is wrong for the sequential-within-one-
> resolution case specifically).
> **Fix sketch** (not done here, correctly out of scope): rebuild `arrived_in_graveyard_this_batch`
> per event-PREFIX rather than per whole-slice, so each event only "looks back" at deaths that
> occurred strictly before it in `events`' order, not after.
> **Filed by**: PB-DX24 fix cycle (`scutemob-202`), review Finding 3.

### F5 (LOW) — no benchmark was recorded

Ran the three named benches (`cargo bench -p mtg-engine --bench engine_perf -- "full_turn_4p|sba_check|priority_cycle_4p"`, release/opt profile):

| bench | this fix cycle | prior pin (CLAUDE.md / PB-DX6 collect) |
|---|---|---|
| `priority_cycle_4p` | 24.60–24.83 µs | 25.5–26.0 µs |
| `sba_check` | 14.92–14.99 µs | 14 µs |
| `full_turn_4p` | 221.46–223.45 µs | 220–222 µs |

**All three within noise of the prior pin** — no regression. Expected: this batch's new graveyard
dispatch arm only runs per `CreatureDied` event, and `full_turn_4p`'s baseline turn has few (if
any) creature deaths, so the marginal cost is not visible at this resolution.

### F6 (LOW) — the arm-count cells restated a number with no counting rule

Added a counting-rule paragraph directly under the lossy-lowering table in
`replay_harness.rs` (`build_face_ability_vectors`'s doc comment). Rule: count every
`for ability in abilities` loop-opening line in the file's CODE (not doc comments), subtract the
2 that sit OUTSIDE `build_face_triggered_abilities` (the mana-ability and activated-ability loops,
which run over the unfiltered slice before any `trigger_zone` filtering). **Verified by execution,
not trusted from the review**: `grep -c "for ability in abilities {" crates/engine/src/testing/replay_harness.rs`
→ 36 (after phrasing the new doc paragraph so it does NOT itself contain the literal substring
`for ability in abilities {` — an earlier draft of this exact comment DID contain it and inflated
the count to 37, which would have made the rule self-falsifying the moment a future reader ran the
`rg` command the rule describes; caught and reworded before committing). 36 − 2 = 34, matching
both the shipped table cells and stage 3's own re-measurement.

### F7 (LOW) — queue-time vs consume-time face residual at Q3/Q4

Added a residual comment at both `abilities.rs`'s Q3 (`WhenExertedAsAttacks`) and Q4
(`WhenDealsCombatDamageToPlayer`) sites: the fix reads `is_transformed` at QUEUE time, while
`resolution.rs`'s own read side documents an explicit "is_transformed at CONSUME time" contract
(verified by reading `resolution.rs:2177` and `:2209` at this fix cycle's HEAD — both still say
exactly `"is_transformed at consume time" contract`). Same expression, not the same evaluation — a
permanent that transforms between the trigger's queue point and its later resolution would desync
in either direction. Zero corpus exposure (stage 1 measured 0 real back-face Q3/Q4 shapes), and
the fix is still strictly better than pre-PB-DX24 code on every state reachable today. The durable
fix (snapshotting the face onto `PendingTrigger` itself) is a HASH bump, correctly out of scope.

**Seed text for `OOS-DX24-8`** (coordinator to file):

> **Title**: Q3/Q4's queue-time `is_transformed` read can desync from `resolution.rs`'s documented
> consume-time contract if the source permanent transforms between queue and resolution.
> **Class**: correctness, latent — 0 real corpus exposure (stage 1 measured 0 back-face
> `WhenExertedAsAttacks`/`WhenDealsCombatDamageToPlayer` abilities in the whole corpus).
> **Mechanism**: `abilities.rs`'s Q3/Q4 sites read `src_obj.is_transformed` at the moment the
> trigger is QUEUED (inside `check_triggers`); `resolution.rs:2177`/`:2209` documents its own read
> as an explicit "is_transformed at CONSUME time" contract. They are the same Rust EXPRESSION
> (`def.effective_abilities(<obj>.is_transformed)`) evaluated at two different points in time, so
> for a permanent that transforms between the trigger firing and the trigger resolving, the two
> reads can disagree.
> **Direction**: could go either way (a permanent that starts transformed and untransforms before
> resolution, or vice versa); PB-DX24 made both sites strictly no-worse than before (pre-batch code
> read `def.abilities` unconditionally at these sites, which is simply the FRONT face always — this
> fix is at least sometimes right where the old code was never right for a transformed attacker).
> **Fix sketch** (a HASH bump, correctly out of scope for a fix cycle): snapshot `is_transformed`
> onto `PendingTrigger` at queue time and have `resolution.rs` read the snapshot instead of the
> live object.
> **Filed by**: PB-DX24 fix cycle (`scutemob-202`), review Finding 7.

### F8 (MEDIUM) — R1 front-face-only; R2 pinned a count, not the seven-shape measurement

`trigger_zone_population` (R1) now also walks `def.back_face`'s abilities. `r2_back_face_population_...`
now asserts the seven per-shape counts directly (each measured 0 at stage 1), in addition to the
15-def-count pin (kept as an eighth assertion). **Both reverts executed**:

- R1: added a synthetic `Triggered { trigger_zone: Some(TriggerZone::Graveyard), .. }` ability to
  `delver_of_secrets.rs`'s BACK face. Reddened: `got {"Bloodghast", "Delver of Secrets", "Nether
  Traitor", "Squee, Goblin Nabob"}` (names the new member). Restored; `git diff --stat` empty.
- R2: added `AbilityDefinition::Keyword(KeywordAbility::Backup(1))` to the same back face. Reddened
  the Q1 shape assertion: `Got 1` (expected 0), naming the exact Q-site it would unblock. Restored;
  `git diff --stat` empty.

### F9 (MEDIUM) — T3's CR 118.12 citation asserted the opposite of what CR 118.12 says

Reworded T3's doc comment and its assertion message: CR 118.12 makes optional-cost payment a
PLAYER CHOICE ("checks whether the player CHOSE to pay"); the engine's OWN deviation is
pay-when-able, documented at its one implementation site (`effects/mod.rs:4299-4301`,
`try_pay_optional_cost` — verified by reading, the comment there literally says "CR 118.12:
beneficial optional cost. Deterministic path: the payer pays when able..."), and already named
"the DP-19 (`MayPayThenEffect`) bug class" at `engine.rs:1568` (verified). T3 now pins the ENGINE
deviation, cites CR 118.12 correctly (as the rule the engine deviates FROM, not the rule it
implements), and cross-references `OOS-DX24-9`.

**Seed text for `OOS-DX24-9`** (coordinator to file):

> **Title**: `nether_traitor`'s "you may pay {B}" is engine-chosen (pay-when-able), not a real
> player decision, contradicting CR 118.12's own text.
> **Class**: correctness deviation, LIVE on a `Complete` deck-legal card (`nether_traitor`) — every
> game where Nether Traitor's trigger fires with `{B}` available auto-pays, with no channel for a
> player to decline even when declining would be strictly better play (e.g. holding the mana up
> for something else, or deliberately keeping the Traitor in the graveyard for a later, larger
> return window).
> **Mechanism**: `effects/mod.rs:4299-4345`'s `MayPayThenEffect` handler calls
> `try_pay_optional_cost` unconditionally for every eligible payer with no suspension point — this
> is the pre-existing DP-19 bug class (`engine.rs:1568`), NOT introduced by PB-DX24, but PB-DX24 is
> the first batch to make `nether_traitor`'s specific instance of it reachable end-to-end (its
> trigger previously never fired at all, from either zone).
> **Cross-reference**: `OOS-DP10-9` (the class this instance belongs to).
> **Filed by**: PB-DX24 fix cycle (`scutemob-202`), review Finding 9, plan §10 risk #9.

### F10 (LOW) — T7's corpus differential was front-face-only

Extended `test_dx24_lowering_drops_every_zone_scoped_ability_over_the_corpus` to also
differentiate over `def.back_face`'s abilities for every def with a back face (same filter, same
`build_face_ability_vectors` comparison, pushing `"{name} (back face)"` into `divergent_defs` on a
mismatch). Currently contributes 0 to `non_identity_inputs` (stage 1: 0 back-face
`trigger_zone: Some(_)` abilities in the corpus) — a coverage completion, not a new finding.

### F11 (LOW) — the Q2/Q7 structural pin was comment-blind with a brittle hard window

Rewrote `test_dx24_q2_and_q7_queue_sites_call_effective_abilities`: (1) duplicated the roster
file's `strip_line_comments`/`strip_block_comments`/`strip_comments` idiom into the primitives
test file (no shared support crate exists between the `primitives` and `core` test binaries, so
byte-identical duplication rather than a shared helper — noted as a real limitation, not silently
glossed); (2) replaced the hard 8-line window with a statement-boundary scan (first line after the
anchor's own comment block whose comment-stripped, trimmed text ends with `;` or `{`, i.e. a `let`
binding's end or a `for`/`if` header's end).

**Reverts executed**: (a) restored `def.abilities.iter().enumerate()` at the Q2 site — reddened
naming Q2 exactly. (b) Repeated the PB-DX32 M8 experiment from this project's own precedent:
replaced the working `let eff = def.effective_abilities(...)` line with a genuine
`/* effective_abilities( */` block comment ABOVE the reverted bare-`.abilities` loop — confirmed
this correctly REDDENS (the block comment's text is stripped before the `contains()` check, so it
cannot fool the gate the way it fooled a line-comment-only scanner in the PB-DX32 precedent). Both
restored; `git diff --stat` empty before continuing each time.

### F12 (LOW) — stray blank line in `nether_traitor.rs`; the stale "untouched" claim

Removed the blank line between `targets: vec![],` and `modes: None,` in the struct literal. Ran
`tools/check-defs-fmt.sh`: **"1803 defs checked / clean"**. Corrected the execution-notes' stale
claim (see the "Stage-7 gap" section at the top of this fix-cycle entry) — the file WAS touched, at
stage 7, in a commit with no execution-notes entry of its own.

### Full verification (this fix cycle, all executed)

- `cargo test -p mtg-engine --test primitives pb_dx24`: 17/17 green.
- `cargo test -p mtg-engine --test core pb_dx24`: 5/5 green.
- `cargo test --workspace --no-fail-fast`: **4,435 / 0 / 5** (+2 over the 4,433 pre-fix-cycle
  baseline — the two new runtime probes `test_dx24_transform_state_resets_on_zone_change_to_graveyard`
  and `test_dx24_apply_face_change_is_a_noop_off_the_battlefield`; every other fix strengthened an
  EXISTING test or extended one without adding a new `#[test]` function), residual list empty.
- `cargo test -p mtg-engine --test core protocol_schema` / `--test core hash_schema`: both green.
  **PROTOCOL 35 / HASH 73 both gate-confirmed unmoved** (also read directly off the source
  constants: `protocol.rs:360` / `hash.rs:757`).
- `cargo clippy --workspace --all-targets -- -D warnings`: clean (one real finding fixed along the
  way — `clippy::manual_pattern_char_comparison` on a `.split(|c: char| c == ',' || c == ';')`,
  rewritten to `.split([',', ';'])`).
- `cargo fmt --check`: clean (one `cargo fmt` run — reflowed a `for` loop header and two call sites
  across the two touched test files; no semantic change, confirmed by re-running every gate/probe
  after).
- `tools/check-defs-fmt.sh`: 1803 defs, clean.
- `cargo build --workspace`: clean.
- Scope: `git status --short` shows exactly 7 tracked files touched (5 engine source/test files +
  1 card-def file) plus this execution-notes file and the (previously untracked) review doc.
  `git diff main..HEAD --numstat -- crates/simulator/ tools/ crates/card-types/`: empty.
  `git diff main..HEAD --numstat -- crates/card-defs/`: exactly one file, `nether_traitor.rs`,
  `+17 -0` total across the whole batch (stage 7's comment block + this fix cycle's 1-line
  deletion).
