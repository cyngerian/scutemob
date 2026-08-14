# PB-DX28 — part 1 execution notes (§2 owner axis + §3 `EffectTarget::DamagedPlayer`)

Scope of this run: `pb-plan-DX28.md` §2 and §3 ONLY. §1 (untargeted-choice channel) and §4
(allowlist retirement) are separate runs — not touched here.

## Starting point

Resumed from `memory/primitives/pb-DX28-RESUME.md` at commit `e5ee1994` ("mechanical — owner:
None at all 46 WheneverCreatureDies sites"), which did not compile: 7 errors (1 `E0432` unresolved
import, 3 `E0027` missing-field patterns, 1 `E0063` missing-fields initializer, 2 `E0004`
non-exhaustive-match) exactly as the dispatch brief predicted. All 7 fixed by implementing
behaviour, not by adding wildcards — see "Engine changes" below.

## Engine changes (files touched, with line references as of this commit)

- `crates/card-types/src/cards/mod.rs`, `crates/card-types/src/cards/helpers.rs`,
  `crates/engine/src/cards/mod.rs`, `crates/engine/src/lib.rs` (already had it),
  `crates/engine/src/state/hash.rs`, `crates/engine/src/testing/replay_harness.rs`:
  `TargetOwner` was declared in `card_definition.rs` by the WIP commit but not re-exported through
  four of the five re-export chains card defs / engine internals / tests actually import it
  through. Added to each.
- `crates/engine/src/effects/mod.rs`:
  - `resolve_effect_target_list_indexed`: new `EffectTarget::DamagedPlayer` arm, resolving from
    `ctx.damaged_player` via `state.expect_player` (not a bare `.players.get`, to keep the SR-25
    bare-lookup ratchet at its pinned ceiling — see "Bare-lookup ratchet" below). Resolves to the
    EMPTY set when no damaged-player context exists, matching every other single-player
    `EffectTarget` arm in this resolver (`TriggeringCreature`/`EquippedCreature`/
    `LastCreatedPermanent`) rather than falling back to `ctx.controller` the way
    `PlayerTarget::DamagedPlayer` does — deliberate, and stated in the type's own doc comment.
  - `filter_states_a_quality`: added `qualities.owner = TargetOwner::default();` to the exclusion
    list, with the CR 701.23b rationale in-line.
- `crates/engine/src/rules/casting.rs`, `validate_object_satisfies_requirement`: added a
  `passes_owner` check (mirroring `passes_controller`) to all FOUR filter-carrying arms —
  `TargetCreatureWithFilter`, `TargetPermanentWithFilter`, `TargetCardInYourGraveyard`,
  `TargetCardInGraveyard` — matching `filter.owner` against `obj.owner`/`caster`.
- `crates/engine/src/rules/abilities.rs`:
  - `trigger_battlefield_target_matches`: `passes_owner` added to the `TargetCreatureWithFilter`
    and `TargetPermanentWithFilter` arms, matching `f.owner` against `obj.owner`/`trigger.controller`
    ("you" = the ability's controller, same convention as `passes_controller` in this function).
  - `trigger_target_candidates`: `owner_ok` added to both `TargetCardInYourGraveyard` and
    `TargetCardInGraveyard` filter closures.
  - The `AnyCreatureDies` BATTLEFIELD dispatch loop (`collect_graveyard_carddef_triggers`'s
    battlefield sibling, ~line 4960): `dying_is_token` and the new `dying_owner` now share ONE
    `state.objects.get(&dying_obj_id)` call (`dying_obj_snapshot`) rather than two separate bare
    lookups — see "Bare-lookup ratchet". `df.owner_you`/`df.owner_opponent` checked against
    `dying_owner` vs `obj.owner` (the watching permanent's own owner).
  - `collect_graveyard_carddef_triggers`'s `WheneverCreatureDies` arm (the GRAVEYARD dispatch
    site, ~line 7395): destructured the new `owner` field as `owner_scope` (avoiding a shadow of
    the outer `owner` variable, which is the trigger source's own owner from
    `ZoneId::Graveyard(owner)`). `dying_owner` read via `state.fizzle_object(*new_grave_id)` (a
    rules-correct fizzle classification, not `expect_*` — the card may have left the graveyard
    between the death event and this dispatch) and compared against `owner` (the trigger source's
    owner). The PRE-EXISTING comment above the `controller`/`death_scope` field ("do NOT read the
    dying object's owner here instead, that would give the SAME DSL field two different
    meanings") is about that OTHER field and was left untouched — it does not apply to the new,
    separate `owner` field.
- `crates/engine/tests/core/pb_dx42a_continuous_condition_roster.rs`: `TARGET_FILTER_FIELDS`
  (33rd field, `owner`) — see "Unplanned failure" below.
- Four test files fixed for the mechanical `E0063`/`E0027` fallout of the new `TargetFilter.owner`
  / `DeathTriggerFilter.owner_you`/`owner_opponent` / `TriggerCondition::WheneverCreatureDies.owner`
  fields: `crates/engine/tests/rules/creature_triggers.rs`,
  `crates/engine/tests/rules/etb_trigger_subtype_filter.rs`,
  `crates/engine/tests/primitives/pb_dx24_trigger_zone_and_index_spaces.rs`,
  `crates/engine/tests/primitives/pbn_subtype_filtered_triggers.rs`.

## Card-def repairs (exactly the three named in the dispatch brief, no others)

- `crates/card-defs/src/defs/staff_of_compleation.rs`: `TargetPermanentWithFilter` now carries
  `owner: TargetOwner::You` and DROPS `controller: TargetController::You`. In-def comment rewritten
  (it used to record the gap as unfixable — now false).
- `crates/card-defs/src/defs/nether_traitor.rs`: `controller: None, owner: Some(TargetOwner::You)`.
  In-def comment rewritten (used to claim "the DSL has no owner-scoped death trigger" — now false —
  AND corrected the note's own `fecundity` citation, which the plan's §0.2 census found wrong:
  `fecundity`'s gap is a CONTROLLER gap [`PlayerTarget::ControllerOf(TriggeringCreature)`], not an
  ownership approximation, per `fecundity.rs`'s own `partial` note. `fecundity.rs` itself was NOT
  edited — that's `nether_traitor`'s citation of it being wrong, not `fecundity`'s own content).
- `crates/card-defs/src/defs/sword_of_war_and_peace.rs`: `targets: vec![]`; both
  `DeclaredTarget { index: 0 }` (the `DealDamage.target`) and `PlayerTarget::DeclaredTarget { index:
  0 }` (the `ZoneTarget::Hand.owner` and the `EffectAmount::CardCount.player`) replaced with
  `EffectTarget::DamagedPlayer` / `PlayerTarget::DamagedPlayer`. Comment rewritten to note the old
  comment already claimed `ctx.damaged_player` resolution while the code used a declared target —
  the PB-DX27 stale-note class, live in a def this batch's own census found.

All three stay `Completeness::Complete`. No completeness marker was changed anywhere.

## Enforcement-site census actually verified (not just the brief's list)

- `casting::validate_object_satisfies_requirement`: verified by reading the whole function,
  confirmed exactly four filter-carrying `TargetRequirement` arms exist
  (`TargetCreatureWithFilter`, `TargetPermanentWithFilter`, `TargetCardInYourGraveyard`,
  `TargetCardInGraveyard`); all four now carry `passes_owner`.
- `rules::abilities`'s triggered-ability auto-target picker: TWO functions, not one —
  `trigger_battlefield_target_matches` (battlefield family, boolean predicate) AND
  `trigger_target_candidates` (candidate ENUMERATION, separate function, two graveyard-arm
  closures). The brief named the first; the second was found by reading the whole file's
  `TargetFilter`-consuming surface and is equally load-bearing (it is what populates a picker /
  bot candidate list, not just what validates a submitted answer). Both fixed.
- `rules::queries::spell_target_requirements` / `legal_targets_per_slot`: VERIFIED, not assumed —
  read `legal_targets_per_slot` (`queries.rs:214-259`) and confirmed it delegates EVERY candidate
  through `casting::validate_targets_inner` → `validate_object_satisfies_requirement`, the exact
  function already fixed. No separate owner-axis code exists in `queries.rs`; PB-DX20's "the offer
  layer and the cast path are one arithmetic" claim holds for this axis too.
- `filter_states_a_quality` (`effects/mod.rs`): confirmed as the brief specified, `owner` added to
  the exclusion list.

No enforcement site was found that the brief's list missed (unlike the four preceding DX-family
batches this brief itself warned about) — but see "Unplanned failure" below for a REACTIVE gap the
brief's list also missed: a THIRD, pre-existing structural gate (`pb_dx42a_continuous_condition_
roster.rs`) that keys on `TargetFilter`'s exact field COUNT and would have silently gone blind the
moment `owner` was added, with no compile error and no test failure pointing at the cause without
investigation.

## Unplanned failure found and fixed: `pb_dx42a_continuous_condition_roster::t6`

Reproduced red at HEAD (not predicted by the brief) after all 7 build errors were fixed. Root
cause: `TARGET_FILTER_FIELDS`, a hand-maintained `&[&str]` fingerprint used by
`object_field_set_equals` to recognize a serialized JSON node as "a `TargetFilter`" by EXACT field
SET equality (`m.len() != fields.len()` short-circuits first). Adding `owner` as `TargetFilter`'s
33rd field changed every `TargetFilter` node's serialized shape; the fingerprint (still 32 entries)
stopped matching ANYTHING corpus-wide, so axis 2 (`subtree_contains_target_filter`) silently went
from `{2 members}` to `{}` with zero card-def or condition-dispatch code touched. Confirmed by
diffing against a pristine `c5b9e459` worktree (`git worktree add`), where the same test passes.
Fixed by adding `"owner"` to the constant and updating its doc comment with the mechanism, so the
next field addition finds the explanation already written rather than re-deriving it. `t9`
(structural: the fingerprint must literally match the struct declaration, read from source) confirms
the fix rather than needing a hand check.

## SR-25 bare-lookup ratchet: kept at its pinned ceiling, not raised

Two new bare `.objects.get(&id)` calls this batch's dispatch logic needed would have raised
`src/effects/mod.rs` 108→109 and `src/rules/abilities.rs` 75→77. Both avoided without adding a
raised-ceiling exception:

- `effects/mod.rs`'s new `EffectTarget::DamagedPlayer` arm uses `state.expect_player(dp)` (an
  `engine-bug` classification — `state.players` never loses entries, CR 800.4a removes objects not
  players) instead of a bare `state.players.get(&dp)`, matching the SAME function's existing
  `AttackTarget` arm's convention two cases above it.
- `abilities.rs`'s battlefield `AnyCreatureDies` loop: the existing `dying_is_token` bare lookup and
  the new `dying_owner` read now share ONE `state.objects.get(&dying_obj_id)` call
  (`dying_obj_snapshot`) instead of two separate ones.
- `abilities.rs`'s graveyard-zone dispatch: `dying_owner` uses `state.fizzle_object(*new_grave_id)`
  (not counted by the ratchet's needle at all — `fizzle_object(` is a named helper, not a bare
  `.objects.get(`), reusing the SAME classification the two checks immediately below it
  (`nontoken_only`, `filter`) already use for the SAME id.

`bare_lookup_ratchet::bare_lookup_counts_are_pinned` passes unmoved at its pre-batch ceilings
(108 / 75 / 34) — verified by executing the gate, not by arithmetic.

## Version gates — numbers taken from the gates' own output, not predicted

Both fail, as the plan's §5 wire-impact prediction said they must (`TargetFilter` gains a field,
`TriggerCondition::WheneverCreatureDies` gains a field, `EffectTarget` gains a variant — all three
inside the `Command`/`GameEvent` closure or the `GameState` closure). **Not bumped in this run** —
left for the coordinator per the dispatch brief.

- `hash_schema::declaration_fingerprint_is_pinned`: current pinned `d73666c9...` (36/36); LIVE
  digest `e8ca5110...`.
- `protocol_schema::protocol_schema_fingerprint_is_pinned`: current pinned `bdd02df0...`
  (PROTOCOL 36); LIVE digest `686d14e4...`.

Full failure text (verbatim, from the final `--workspace --no-fail-fast` run):

```
---- hash_schema::declaration_fingerprint_is_pinned stdout ----
The serialized shape of the GameState type closure (130 types) has changed.
  left:  "d73666c948e7b3fe09934d87896585e5a514f559d373076197143461e1312818"
  right: "e8ca51103996c3094a0c6c1e1107511e2f98719e15cf0fe15f1726cc730f4ca5"

---- protocol_schema::protocol_schema_fingerprint_is_pinned stdout ----
The serialized shape of the Command/GameEvent type closure (97 types) has changed.
Currently PROTOCOL_VERSION 36.
  left:  "bdd02df0eb7f84f0a957852a7e0944affa7e0f7c8de1348990ad53d1c5e73f62"
  right: "686d14e4e028f7d1148958ae58fcc17a9f359ed46c4835a864199895077f5f04"
```

## Tests: `crates/engine/tests/primitives/pb_dx28_owner_axis.rs` (12 tests)

Registered in `crates/engine/tests/primitives/main.rs`. Four sections:

- **A** (casting-path `TargetFilter.owner`, `t1`-`t4`): synthetic `{T}: Destroy target permanent
  [owner scope].` activated ability, `Command::ActivateAbility`, `Ok`/`Err(InvalidTarget)`.
- **B** (battlefield `DeathTriggerFilter.owner_you`/`owner_opponent`, `t5`-`t8`): raw
  `TriggeredAbilityDef` attached via `ObjectSpec::with_triggered_ability` (bypasses card-def
  lowering, exercises the dispatch site directly) except `t8`, which goes through the REAL
  card-def lowering (`build_face_ability_vectors`) to exercise `Some(TargetOwner::Any)` as an
  actual enum value, not as two hand-set bools.
- **C** (graveyard-zone dispatch, `t9`-`t10`): the REAL `nether_traitor()` def from `all_cards()`,
  `check_and_apply_sbas` + `state.pending_triggers()` — mirrors
  `pb_dx24_trigger_zone_and_index_spaces.rs`'s fixture pattern exactly, varying owner vs controller
  of the dying creature independently.
- **D** (`EffectTarget::DamagedPlayer`, `t11`-`t12`): `t11` is a synthetic self-referential
  "whenever this deals combat damage to a player, deal N to that player" creature in a 4-player
  game, attacking p3 directly (turn order p1,p2,p3,p4) — isolates the primitive from Equip
  mechanics. `t12` is the card-integration test: the REAL `sword_of_war_and_peace()` def, equipped
  (Equip {2}, ability index derived from `all_cards()`, never hard-coded) and attacking p3, checking
  both `EffectTarget::DamagedPlayer` (damage) and `PlayerTarget::DamagedPlayer` (the `ZoneTarget::
  Hand.owner` / `EffectAmount::CardCount.player` reads) resolve correctly, plus the unrelated
  `PlayerTarget::Controller` life-gain amount for completeness. Required
  `register_static_continuous_effects` after direct battlefield placement (the same
  `GameStateBuilder::object()` gotcha `cards1_equip_target_repair.rs` already documents for
  Skullclamp) so the Sword's +2/+2 static actually applies.

## Revert matrix — every probe, executed red then restored

All six reverts were applied to LIVE source, run against the full `pb_dx28_owner_axis` module,
observed red, then restored verbatim and re-confirmed green with a final full-workspace run
(4,615 passed / 2 failed [the two version gates, unmoved] / 5 ignored — identical before and
after the revert exercise).

| # | Revert | File:site | Probes covered | Observed |
|---|---|---|---|---|
| R1 | `TargetOwner::You`/`Opponent` compare `obj.controller` instead of `obj.owner` (the pre-batch `TargetController` approximation, reproduced verbatim) | `casting.rs`, `TargetPermanentWithFilter` arm | t1, t2, t3 | **RED**: t1 `Err` (expected `Ok`), t2 `Ok` (expected `Err`), t3 both halves flipped. t4 unaffected (control). |
| R2 | `TargetOwner::Any => false` | `casting.rs`, same arm | t4 | **RED**: both accepts became `Err`. t1-t3 unaffected. |
| R3 | `df.owner_you`/`df.owner_opponent` compare `dying_controller`/`obj.controller` instead of `dying_owner`/`obj.owner` | `abilities.rs`, battlefield `AnyCreatureDies` loop | t5, t6, t7 | **RED**: t5 0 triggers (expected 1), t6 1 trigger (expected 0), t7 both halves flipped. t8-t12 unaffected. |
| R4 | Lowering maps `Some(TargetOwner::Any)` onto `owner_you: true` ("Any" mistaken for "You") | `testing/replay_harness.rs`, `build_face_triggered_abilities`'s `WheneverCreatureDies` arm | t8 | **RED**: the P2-owned-fodder half (0 triggers, expected 1). The P1-owned half stayed green — expected: from the watcher's own perspective P1-owned coincidentally still satisfies the incorrectly-widened "You" match, which is itself informative (proves the revert is doing exactly what it claims, not something coarser). Others unaffected. |
| R5 | `dying_owner` reads pre-death `*death_controller` instead of the post-death graveyard object's real `.owner` | `abilities.rs`, `collect_graveyard_carddef_triggers`'s `WheneverCreatureDies` arm | t9, t10 | **RED**: t9 0 triggers (expected 1), t10 1 trigger (expected 0). t1-t8, t11-t12 unaffected. |
| R6 | `EffectTarget::DamagedPlayer` resolves to `ctx.controller` unconditionally instead of `ctx.damaged_player` | `effects/mod.rs`, `resolve_effect_target_list_indexed` | t11, t12 | **RED**: t11 p3 life 38 (expected 35); t12 p3 life 36 (expected 35). All other tests unaffected. |

Zero UNDISCRIMINATED rows — every probe was individually reddened by a revert of the exact code it
exercises.

## What the plan got right, and one correction

- The plan's §2/§3 design (two-bool `DeathTriggerFilter` decomposition rather than a stored
  `TargetOwner`, module-dependency-direction rationale; `EffectTarget::DamagedPlayer` resolving to
  EMPTY rather than falling back to controller; the exact four casting.rs arms; the two
  `trigger_target_candidates`/`trigger_battlefield_target_matches` picker sites) all held exactly as
  specified — nothing needed re-deriving.
- One correction: the plan's §2.1 enforcement-site list did not separately call out that
  `rules::abilities`'s "auto-target picker" is actually TWO functions
  (`trigger_battlefield_target_matches` the predicate, `trigger_target_candidates` the enumerator),
  and only named the graveyard family generically ("`trigger_target_candidates`'s two graveyard
  arms"). Both were verified present and both needed the fix; this is a documentation gap in the
  plan, not a missed site — the plan's own "verify, do not assume" instruction is what caught it.
- The `pb_dx42a_continuous_condition_roster.rs` failure was NOT anticipated by the plan at all
  (§5's wire-impact section discusses `TargetFilter` gaining a field only from the hashing/protocol
  angle, not from this OTHER hand-maintained structural fingerprint). Filed here as a durable
  lesson for future `TargetFilter` field additions, not as a plan defect — the plan had no way to
  know this second, independent gate existed without reading it, and it now says so explicitly in
  its own doc comment.

## Definition-of-done checklist

- `cargo build --workspace`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --check`: clean. `tools/check-defs-fmt.sh`: clean (1,803 defs).
- Full `cargo test --workspace --no-fail-fast`: **4,615 passed / 2 failed / 5 ignored** — the 2
  failures are `hash_schema::declaration_fingerprint_is_pinned` and
  `protocol_schema::protocol_schema_fingerprint_is_pinned`, both EXPECTED (wire change, left for
  the coordinator's bump) and both gate-executed, numbers quoted above verbatim.
- No pre-existing test went red for any reason other than the two version gates. The
  `pb_dx42a_continuous_condition_roster::t6` failure surfaced DURING this run (reproduced at HEAD
  before this run's own tests existed) and was fixed as part of this run's engine work, not
  weakened or glossed.
- Tree left DIRTY, uncommitted, per instructions.
