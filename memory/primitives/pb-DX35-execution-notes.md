# PB-DX35 — execution notes

**Task**: `scutemob-227`. v4 queue rank 12 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 12).
**Seeds**: `OOS-DX4-2` (Half A) + `OOS-DX4-5` (Half B); siblings `OOS-DP10-5`, `OOS-DX8-3`, PB-DX9's row.

---

## §0 — Stage 0: measured BEFORE any production line changed

### §0.1 Baseline

Full-workspace `cargo test --workspace --no-fail-fast` on this branch, tree clean, **before any edit**:

```
5,058 passed / 0 failed / 5 ignored — 61 result-producing targets
```

This **reproduces PB-DX51's published close pin exactly** (5,058 / 0 / 5, 61 targets). Log:
`scratchpad/pbdx35/baseline.log`.

Current fingerprints at the merge base: **HASH 82**, **PROTOCOL 41**.

### §0.2 The wire prediction — written before any production line changed

Both halves are predicted **NONE / NONE**, and the reasons are structural rather than hopeful.
The stop condition is stated with each: if the named premise turns out false, the batch takes ONE
bump per fingerprint for the whole PB (never two) and re-derives the number from the failing
gate's own output rather than inventing it.

#### Half A (`OOS-DX4-2`, modal trigger targets) — predicted **PROTOCOL none / HASH none**

The memo's cell reads *"none-if-registry / both-if-lowered (MED)"*. The reachability trace that
decides it:

* `ModeSelection` (and therefore `mode_targets`) hangs off
  `AbilityDefinition::Triggered`, i.e. off `CardDefinition`. `CardDefinition` is
  **excluded** from the PROTOCOL closure (`CLOSURE_MUST_NOT_CONTAIN`), and
  `GameState::card_registry` is `#[serde(skip)]` for the declaration digest — the same two
  facts PB-DX18 relied on when `AbilityDefinition::Splice` gained a field and PROTOCOL did
  not move.
* The runtime type `Characteristics::triggered_abilities: Vec<TriggeredAbilityDef>` **has no
  `modes` field at all** (`crates/card-types/src/state/game_object.rs:908-966`, read field by
  field). So the incumbent source of a modal trigger's `ModeSelection` is *already* the
  registry: both `rules/abilities.rs` (~:9845) and `rules/resolution.rs` (~:2350) read
  `state.card_registry` → `def.effective_abilities(..)` for it today.
* Therefore **reading `mode_targets` from the registry adds no type, no variant and no field**,
  and the prediction is NONE for both fingerprints.
* **The counterfactual is stated because it is the live alternative**: lowering
  `modes`/`mode_targets` into `TriggeredAbilityDef` would add a field to a type reachable from
  `Characteristics`, which is a PROTOCOL closure root — that is the "both" branch, and it is
  the branch this batch does **not** take.

**Stop condition**: if `protocol_schema` or `hash_schema` reddens, the registry-read premise is
false somewhere and the design is re-examined before any number is edited.

#### Half B (`OOS-DX4-5`, the inert `optional` flag) — predicted **PROTOCOL none / HASH none**

The question variant is the decision, and it is settled by CR text rather than by convenience.
Three candidates were considered:

1. **A new `EffectChoiceQuestion::Confirm`-shaped variant** — a costless yes/no.
   *Rejected.* It is a new variant on a type inside the wire closure, so it costs BOTH bumps,
   and it answers a *narrower* question than the cards ask.
2. **A zero-cost `PayOptionalCost`** — `Cost::Mana(default)`.
   *Rejected as dishonest.* `can_pay_optional_cost` is trivially true for a free cost, the
   client would render "Pay {0}?", and the task brief names this exact trap (DP-12). A question
   whose payload lies to the renderer is worse than no question.
3. **`EffectChoiceQuestion::ChooseObject { candidates, count: 1, up_to: true }`** — CHOSEN.
   This is what the five printed cards literally say. All five read *"You may put **a**
   [creature/land] card from among them into your hand / onto the battlefield"* (MCP,
   verbatim, §0.3): that is CR 608.2's *choose up to one*, which is the exact contract
   `ChooseObject` was built for by PB-DX28 (CR 115.10 untargeted resolution-time choice).
   It carries BOTH halves of the printed decision — **whether** and **which** — where a bare
   confirm carries only the first, and the v4 memo's own row 15 cell already records the
   judgement: *"**DX4-5 now none** (HIGH) — PB-DX28's `EffectChoiceQuestion::ChooseObject
   { up_to: true }` already expresses it"*.

   No new variant, no new field ⇒ **NONE for both fingerprints**.

**The one thing reuse costs, stated rather than discovered later**: `ChooseObject`'s own doc
comment claims *"this one names **public** information: every id is a permanent on the
battlefield or a card in a graveyard"*. `LookAtTopThenPlace` hands it **library** ids, so that
sentence becomes false the moment this batch ships. Redaction is unaffected —
`GameEvent::EffectChoiceRequired` is `private_to(player)`, the same channel `SearchLibrary`,
`Scry` and `Surveil` already carry hidden library ids on — but **a false comment is this
queue's most-repeated defect**, so the doc is corrected in the same commit as the code rather
than left for a later batch to trip over.

**Stop condition**: if either gate reddens, ONE bump per fingerprint is taken for the whole PB
(both halves together), read off the failing gate's own output.

#### Consequence if the prediction holds

No sentinel re-pin and no `*_SCHEMA_HISTORY` row are **owed**. `history_is_append_only` and
`frozen_prefix_is_pinned` are still executed and must be green; that they were not edited is the
claim, and the gates are the evidence.

### §0.3 The mode-choice channel decision for triggers — STATED, with its consequences

`AC 7327` requires this decision to be made and stated. **This batch does NOT give a modal
triggered ability's mode choice a human channel; the `decision_site_walk` `modal_trigger` row
stays `AutoChosen`.** What changes is that the engine's automatic choice becomes
**CR 700.2b-legal**, which it is not today.

**CR 700.2b, verbatim** (MCP):

> The controller of a modal triggered ability chooses the mode(s) as part of putting that
> ability on the stack. **If one of the modes would be illegal (due to an inability to choose
> legal targets, for example), that mode can't be chosen.** If no mode is chosen, the ability
> is removed from the stack. (See rule 603.3c.)

Today `flush_sorted` writes `stack_obj.modes_chosen = vec![0]` in **both** arms of its
`min_modes` branch, unconditionally — so the engine picks mode 0 even when mode 0 is exactly the
mode CR 700.2b forbids it to pick. **"Scope the targets to mode 0" — the option the brief names
— is therefore not available**: it would leave `retreat_to_kazandu` unable to gain 2 life with no
creature on the board, which is the defect, and it would make the two predicted `partial →
Complete` flips false claims. Legality-aware auto-choice is not scope creep here; it is the
minimum CR-correct behaviour, and it is what makes AC 7327's mandated probe (*a mode chosen
WITHOUT a target resolves when the other mode's target is absent*) pass.

**Gate consequences taken**:

* `core::decision_site_walk`'s `modal_trigger` row keeps class `AutoChosen` — the *controller*
  still does not choose — but its `site` string (*"modes_chosen = vec![0] in both the
  min_modes==0 and !=0 arms"*) becomes false and is rewritten.
* No new `EffectChoiceQuestion` and no new `Command` ⇒ the `decision_gate` ratchet and the
  CR 605.4a mana-ability needle list (`test_dp9_mana_ability_gate`) are **not** owed a change by
  Half A. (Half B's own needle situation is §0.4.)
* The human channel is the residual and is **filed**, not silently dropped.

### §0.4 Half B's gate consequences

`LookAtTopThenPlace` is **already** in `test_dp9_mana_ability_gate`'s needle list — PB-DX45 added
it as a deliberately OVER-wide needle for the `place_cost` field. After this batch the needle
stops being over-wide for the five `optional: true` defs (they now genuinely ask) and stays
over-wide only for a hypothetical `optional: false, place_cost: None` def, of which the corpus has
**zero**. **The needle does not change; its justifying comment does**, and that comment is
rewritten rather than left asserting a population that has moved.

`core::decision_site_walk`'s `look_at_top_or_route` row moves `AutoChosen` → `Served`, because its
stated reason (*"LookAtTopThenPlace's `optional` field is inert by construction (OOS-DP10-5)"*)
is exactly what this batch deletes.
