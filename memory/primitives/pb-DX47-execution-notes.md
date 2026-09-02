# PB-DX47 — execution notes

**Task**: `scutemob-218`. v4 queue rank 5 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 5).
**Seed**: `OOS-DX24-4` — "is the `WhenDealsCombatDamageToPlayer` double-push real?"
**Branch**: `feat/pb-dx47-probe-first-is-the-whendealscombatdamagetoplayer-dou`

---

## §0 — Wire prediction, written BEFORE any code changed

**PROTOCOL 39 / HASH 78, both UNMOVED.** Confidence HIGH, and the reason is stated rather
than asserted: whichever way the probe decides, the repair is a *suppression* inside
`rules/abilities.rs`'s `GameEvent::CombatDamageDealt` arm — it adds no type, no variant and
no field to the `Command`/`GameEvent`/`Effect`/`Characteristics` closure, and it changes no
hashed field's *declaration*. `PendingTrigger` is hashed, but the fix changes how many are
*produced*, not what one *is*.

Gate-computed result: recorded in §6.

---

## §1 — The experiment ran FIRST, and it is decisive

`crates/simulator/tests/pb_dx47_double_push_probe.rs`, committed **before any fix**.

### Fixture (why it is what it is)

* **Production pregame path.** The state is built by `mtg_simulator::setup::build_initial_state`
  — the same function `tools/play-server`'s `session::new_game` and `tools/tui` build through
  — not by `GameStateBuilder`. This is not fussiness: the in-source comment the seed is filed
  against claims the runtime lowering "only happens in `enrich_spec_from_def` for tests", so a
  hand-built fixture is exactly the shape the false claim says is special. Proving anything on
  one would prove nothing.
* **Subject `drana_liberator_of_malakir`** — `Complete` by derive, deck-legal, and
  **legendary**, so CR 903.6 puts it in the command zone by construction rather than leaving
  the probe dependent on a shuffle. Its trigger puts a `+1/+1` counter on each attacking
  creature you control, so a double dispatch is visible as **two counters on a lone attacker**,
  not merely as two stack entries.
* **Both seats human** (`human_seats = {p1, p2}`), so no bot RNG enters: every decision in the
  game is made by the probe's own `choose()`.
* Deck: the subject as commander + 99 `Swamp`. Basic lands are exempt from CR 903.5b's
  singleton rule and mono-black satisfies CR 903.4, so the real `validate_deck` gate
  (Architecture Invariant 9, run inside `build_initial_state`) admits it.

### Result — **the double-push is REAL**

```
PB-DX47 P1: subject=Drana, Liberator of Malakir lowered(A)=1 registry(B)=1
PB-DX47 P2: PendingTrigger census by kind = {"CardDefETB": 1, "Normal": 1} (total 2);
            +1/+1 counters on the lone attacker = 2; commands = 126
```

* **P1** — on the object `setup.rs` actually built, **both** dispatch preconditions hold:
  the runtime lowering produced exactly one `TriggeredAbilityDef` with
  `trigger_on == TriggerEvent::SelfDealsCombatDamageToPlayer` (path A), and the card-registry
  def carries exactly one `AbilityDefinition::Triggered { WhenDealsCombatDamageToPlayer }`
  (path B). If the justifying comment were true, `lowered(A)` would be `0`. It is `1`.
* **P2** — the engine's own `check_triggers`, called on the REAL driven state at the moment
  the subject was a declared attacker, pushes **two** `PendingTrigger`s for one event: one
  `PendingTriggerKind::Normal` (the runtime lowering, via `collect_triggers_for_event`) and one
  `PendingTriggerKind::CardDefETB` (the registry scan in the same arm). **No dedup exists.**
  End to end, a card printing ONE `+1/+1` counter put **TWO** on its lone attacker.

### A measurement that returned 0 for the wrong reason — recorded, not dropped

The probe's first draft censused `state.pending_triggers()` after every `advance()`/`submit()`
and measured **0 at every command boundary**. Not because nothing was pushed: because
`check_and_flush_triggers` drains the queue onto the stack inside the *same* `process_command`
call, so `pending_triggers` is never non-empty at any point a test can observe.

**A census that returns 0 because it never got to look is indistinguishable from one that
returns 0 because nothing happened.** What caught it was the end-to-end assertion running
beside it (`counters == 2`) — the census said "nothing" while the board said "twice". The
shipped census therefore calls the engine's own dispatcher directly. Filed as `OOS-DX47-1`.

---

## §2 — Why nobody noticed (the comment is false in TWO ways)

`crates/engine/src/rules/abilities.rs`, the `GameEvent::CombatDamageDealt` arm:

> "CardDef-level `WhenDealsCombatDamageToPlayer` triggers from `AbilityDefinition::Triggered`.
> These are not converted to runtime `TriggeredAbilityDef` (that only happens in
> `enrich_spec_from_def` for tests), so we collect them here from the card registry."

Both halves are false at HEAD:

1. **"not converted to runtime `TriggeredAbilityDef`"** — `build_face_ability_vectors`
   (`testing/replay_harness.rs`) has a dedicated loop that converts exactly this
   `TriggerCondition` into a `TriggeredAbilityDef { trigger_on:
   TriggerEvent::SelfDealsCombatDamageToPlayer, .. }`. PB-DX1 even *extended* that loop
   (`intervening_if` propagation) without anyone reconciling it with this comment.
2. **"only happens in `enrich_spec_from_def` for tests"** — `enrich_spec_from_def` is the
   **production pregame path**: `setup.rs:419/433/440` (commander, opening hand, library) and
   `fuzz_setup.rs:119/130`. Every object in every real game goes through it.

The same false sentence is copy-pasted onto the `WhenExertedAsAttacks` arm
(`abilities.rs:4290`), which cites this arm as its precedent — so the claim propagated.


---

## §3 — The fix: one dispatcher, chosen on CR grounds

`rules/abilities.rs`, `GameEvent::CombatDamageDealt` arm: the card-registry scan is **deleted**.
The layer-resolved runtime lowering is the single authoritative dispatcher.

**Why the lowering is the survivor rather than merely the incumbent.** It is the CR-correct one of
the two on three independent axes, each of which is a place the deleted scan was *also* wrong:

| axis | runtime lowering (kept) | registry scan (deleted) |
|---|---|---|
| CR 613.1f ability removal | `collect_triggers_for_event` reads `expect_characteristics` — Humility / Dress Down / any `RemoveAllAbilities` suppresses it | reads the printed def; **bypasses layers entirely**, so a blanked permanent still triggered |
| granted / copied abilities | sees them (they are in `characteristics.triggered_abilities`) | invisible — they are not on the card |
| tokens | sees them | invisible — a token has no `card_id` |

**The scan's own historical justification is DISCHARGED, and by execution rather than argument.**
PB-EF3 A2's comment said `CardDefETB` "keeps the raw-index/card-registry lookup authoritative for
both effect and target selection (Throat Slitter's *'destroy target nonblack creature that player
controls'* needs its declared `targets` to survive auto-target selection — EF-W-MISS-10)". The
lowering copies `targets` verbatim and `flush_sorted` reads `ab.targets` for a `Normal` trigger
through the same code that reads the registry for a `CardDefETB` one — and
`pbd_damaged_player_filter::test_throat_slitter_end_to_end_precision_fix` **passes** through the
surviving path. Revert row **V3** (drop `targets: targets.clone()` from the lowering) reddens both
that end-to-end probe and roster row `r5`, so this is measured, not asserted.

### What the scan really was load-bearing for, and it is not what the comment said

`test_throat_slitter_end_to_end_precision_fix` failed on the first run after the deletion, and the
cause was **its fixture, not the fix**: `ObjectSpec::creature(p1, "Throat Slitter", 2, 2)
.with_card_id(..)` is a **NAKED object** — `characteristics.triggered_abilities` is empty — so only
a registry scan could ever have fired it. No object in a real game is shaped like that:
`setup.rs:419/433/440` and `fuzz_setup.rs:119/130` route every card through `enrich_spec_from_def`.
The fixture was repaired (enriched) rather than the path preserved, which is the PB-DX25b lesson
one fixture over. Filed as **`OOS-DX47-4`**, *with its unmeasured half stated*: how many other
tests in the tree are green against that unreachable shape is UNKNOWN.

---

## §4 — Census (AC 7248), PRINTED not transcribed

Read off `core::pb_dx47_dispatch_path_roster::t_census_report`:

```
axis 1 (structural, AbilityDefinition::Triggered over every face):
  defs declaring WhenDealsCombatDamageToPlayer : 26
  of which deck-legal `Complete`               : 18
  total declarations (a def may declare >1)    : 26
axis 2 (inverse, printed oracle text of every face):
  Complete defs printing the trigger but NOT declaring it : 20
class sweep:
  TriggerConditions lowered by build_face_ability_vectors : 34
  TriggerConditions registry-scanned in abilities.rs      : 6
  intersection (post-filter allowlist applies)            : ["WheneverYouSacrifice"]
```

**The v4 memo's conditional "18 deck-legal `Complete` defs if real" reproduces EXACTLY.** That is
worth saying plainly rather than glossing: it was re-derived (dispatch hygiene 6) precisely because
six batches in a row found a filed member list short and PB-DX45 found the first over-count, and a
memo figure agreeing with a re-derivation is the outcome the discipline is FOR. The agreement is
kept from being self-congratulatory by axis 2, where the two do **not** agree.

**The registry row's "no corpus exposure" is replaced by the measured figure.** The row itself
flagged the claim as resting on a targeted check rather than an `all_cards()` enumeration; the
enumeration refutes it, and the derivation is stated in the row: *the set of `CardDefinition`s
whose front, back or adventure face declares `AbilityDefinition::Triggered { trigger_condition:
WhenDealsCombatDamageToPlayer }`, restricted to `completeness == Complete`.*

### The batch's roster was wrong on its first run, and its own gate caught it

`DECLARING_MEMBERS` was typed from `grep -l WhenDealsCombatDamageToPlayer
crates/card-defs/src/defs/*.rs`, which returns **30 files**. The `all_cards()` walk returns **26
defs**. The four extras — `bident_of_thassa`, `exalted_angel`, `moria_marauder`,
`parapet_thrasher` — name the variant only inside a `// TODO` or a `Completeness` note explaining
why they *cannot* use it. That is **SR-36's rule verbatim** (*enumerate `all_cards()` for rosters,
never grep source*) broken inside the batch whose subject is a false comment, and `OOS-CARDS2-7`'s
shape a second time. Filed as **`OOS-DX47-2`**; the durable half is that **a pinned roster and its
derivation must not share a source**.

---

## §5 — The CLASS (AC 7247), swept mechanically

The defect is not "this event" — it is "two dispatchers". `r3_no_trigger_condition_has_two_dispatchers`
intersects the `TriggerCondition` names `build_face_ability_vectors` converts (**34**, parsed from
its source) with those the `rules/abilities.rs` queue sites match while walking
`def.effective_abilities(..)` (**6**), and fails on any member. Both inputs are source-derived
rather than hand-listed, for `OOS-DX24-4`'s own reason: **a hand-listed set is a claim, and this
defect survived five months behind exactly such a claim written as a comment.**

Post-fix intersection: `{WheneverYouSacrifice}` alone, allowlisted with the reason stated rather
than by loosening the gate — its `abilities.rs` occurrence is inside a `triggers.retain(..)`
**post-filter** that refines the `Normal` triggers the lowering produced, never a second
`triggers.push`. Revert row **V4** (empty the allowlist) reddens `r3`, so the allowlist is proven
load-bearing rather than decorative, and `r3` carries a self-check asserting the allowlisted member
is genuinely in the intersection so the allowlist cannot hide a broken parser.

The other four registry-scanned conditions are single-path by construction and now proven so:
`WhenYouCastThisSpell`, `WhenExertedAsAttacks`, `WhenTurnedFaceUp`, `WheneverRingTemptsYou` have no
lowering loop; `mana.rs`'s `WhenTappedForMana` push and the `TriggerZone::Graveyard` collector are
mutually exclusive with the lowering by construction (`lowers_onto_the_battlefield`, PB-DX24).

---

## §6 — Gates

**Wire, predicted in §0 before any code and gate-computed after: PROTOCOL 39 / HASH 78, both
UNMOVED.** `hash_schema` 36/36, `protocol_schema` 17/17, `HASH_SCHEMA_VERSION = 78` at HEAD.
`history_is_append_only` 2/2 and `frozen_prefix_is_pinned` 2/2 green. No pin edited and no history
row appended — none was owed.

**Tests: 4,872 / 0 / 5** full-workspace, `--workspace --no-fail-fast` to a file, **56**
result-producing targets (55 → 56: one new simulator test binary), residual list empty. The
pre-edit baseline was measured on this branch BEFORE any edit at **4,861 / 0 / 5** across **55**
targets and reproduces PB-DX45's close pin exactly.

**Delta itemised by test NAME (set-diff of the two run logs): 12 additions, 1 leaver, 0 removals.**

Additions — 9 in the new `crates/engine/tests/core/pb_dx47_dispatch_path_roster.rs`
(`r1`, `r1b`, `r2`, `r3`, `r3b`, `r4`, `r5`, `r5b`, `t_census_report`), 2 in the new
`crates/simulator/tests/pb_dx47_double_push_probe.rs` (`p1`, `p2`), 1 the inversion's successor
in `crates/engine/tests/primitives/pb_dx24_trigger_zone_and_index_spaces.rs`.

The single leaver is **disclosed rather than netted out**, because it is not a removal:
`test_dx24_when_deals_combat_damage_to_player_reads_the_visible_face_of_a_transformed_attacker`
became `test_dx47_transformed_attacker_queues_exactly_one_trigger_off_the_visible_face` — same
file, same underlying Q4 property, subject **inverted**, because what it pinned is what this batch
deleted. See `OOS-DX47-5`: that test was a pin ON this defect and its own docstring said so.

**Coverage 1,137 / 1,803 = 63.1%, 0 flips**, proven by regeneration (`clean 1,137 / todo 519 /
empty 147` all identical); self-dating churn reverted. **Zero card-def edits** —
`git diff --stat -- crates/card-defs/` is empty, so the shortcut is available and was checked.

`clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
`tools/check-defs-fmt.sh` clean (1,803 defs) — all against the FINAL tree (PB-DX15a's lesson).

### Revert matrix — 8 rows, 8 RED, 0 UNDISCRIMINATED

| # | revert | expected | observed |
|---|---|---|---|
| V1 | restore the deleted registry scan in the `CombatDamageDealt` arm | `r3`, `r4`, the Q4 inversion and the `p2` probe all red | **RED (4 rows)** |
| V2 | un-enrich the Throat Slitter fixture (back to a naked object) | `test_throat_slitter_end_to_end_precision_fix` red | **RED** |
| V3 | drop `targets: targets.clone()` from the lowering | `r5` red, Throat Slitter end-to-end red | **RED (2 rows)** |
| V4 | empty `r3`'s `POST_FILTER_ONLY` allowlist | `r3` red (allowlist load-bearing) | **RED** |
| V5 | make `glissa_sunslayer` `Complete` | `r5b`'s deck-legal half red | **RED** |
| V6 | break `r2`'s oracle needle | `r2` non-vacuity floor red | **RED** |
| V7 | drop `"Scroll Thief"` from `DECLARING_MEMBERS` | `r1` red | **RED** |
| V8 | disable `condition_names_in`'s comment stripping | `r3b` red | **RED** (`r3` stayed GREEN — see below) |

**Two honest notes on rows that did NOT move, disclosed rather than left implicit:**

* Under **V1**, `p1_production_pregame_satisfies_both_dispatch_preconditions` stays green, and
  correctly so — it measures the two dispatch *preconditions* on the production-built object, both
  of which hold whether or not the scan exists. It is a fact-recording probe, not a fix pin, and
  it is the row that refutes the false comment directly.
* Under **V1**, the Throat Slitter probe also stays green: the enriched fixture works under both
  engines, which is the point. It is discriminated by V2 and V3 instead.
* Under **V8**, `r3` stayed **GREEN** while `r3b` reddened. Neither source file currently contains
  a commented-out `trigger_condition: TriggerCondition::X` pair, so the stripping is *defensive*
  for `r3`'s verdict today and load-bearing for `r3b`'s own guarantee. Stated as a residual in
  `r3b` itself and filed as **`OOS-DX47-7`**, rather than left to read as full coverage.

---

## §7 — Seeds filed

`OOS-DX47-1` (a boundary census of `pending_triggers` measures 0 for a reason unrelated to
triggers), `OOS-DX47-2` (a roster typed from `grep -l` counts TODO comments — SR-36 broken inside
this batch), `OOS-DX47-3` (the lowering drops `modes`: one member, `glissa_sunslayer`, `partial`,
**zero** deck-legal exposure — and the first draft of that claim said zero members, refuted by this
batch's own `r5b` on its first run), `OOS-DX47-4` (naked-object fixtures test an unreachable shape;
one repaired, population unmeasured), `OOS-DX47-5` (PB-DX24's Q4 probe was a pin ON this defect and
its docstring said so — the durable rule is that isolating a path you are changing becomes a pin
the moment the isolation is the only thing asserting the count), `OOS-DX47-6` (the false comment
propagated by being cited as precedent; a true conclusion on a false general premise reads as
confirmation), `OOS-DX47-7` (`r3`'s parser strips `//` only, bound stated).

**`OOS-DX24-4` is CLOSED**, with four corrections to its own claims recorded in the row itself.
