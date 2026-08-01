# PB-DX3 Plan — two stale blocker notes: `garruks_uprising` + `inventors_fair` (OOS-DP6-3)

<!-- last_updated: 2026-08-01 -->

- **Task**: `scutemob-164`
- **Branch**: `feat/pb-dx3-two-stale-blocker-notes-garruksuprising-inventorsfair`
- **Queue**: `memory/primitives/seed-rerank-2026-07-27.md` §4 rank 3; analysis §2.4; brief at §"Dispatch briefs".
- **Class**: **CARD YIELD, ZERO ENGINE.** 2 flips (`partial` → `Complete`), 0 engine lines,
  0 wire movement (PROTOCOL **32** / HASH **69** must be unmoved, proven by an *empty*
  `git diff` over `crates/engine/src/rules/protocol.rs` and `crates/engine/src/state/hash.rs`).

---

## §1 Premise re-verification (base = this worktree's fork point off `main`)

Every claim in the dispatch brief was re-derived against source before any edit. All hold, and
two are **stronger** than the brief states.

| Brief claim | Status | Evidence on current main |
|---|---|---|
| Both notes name the **runtime** `InterveningIf` (2 variants) when the def-level field is `Option<Condition>` | **HOLDS** | Def field: `card_definition.rs:342` `intervening_if: Option<Condition>`. Runtime enum: `crates/card-types/src/state/game_object.rs:819` — and it now has **three** variants, not two: PB-DX1 added `InterveningIf::CardDef(Box<Condition>)` at `:838`. Both notes are stale twice over. |
| `Condition::YouControlNOrMoreWithFilter { count, filter }` exists | **HOLDS** | `card_definition.rs:3834`. |
| 21 shipped defs already use it | **HOLDS** | `rg -l 'intervening_if: Some' crates/card-defs/src/defs/` ∩ the variant → 21 files. Closest templates: `dragonmaster_outcast.rs:44` (upkeep + land count), `hellkite_tyrant.rs:36` / `revel_in_riches.rs:45` (upkeep + **artifact** count — the exact filter Inventors' Fair needs), `growing_rites_of_itlimoc.rs:46` (creature count). |
| The variant is queue-time evaluable | **HOLDS** (cite drifted) | `effects/mod.rs:**10151**`, not `:10139` — inside `condition_is_queue_time_evaluable`'s `true` arm. The brief's `:10139` is now `CanRevealFromHandWithSubtype`. Cite **by symbol**, not line (PB-DX2's lesson). |
| PB-DP6 wired the card-def `intervening_if` into the queue sites | **HOLDS, and both sites this batch needs are among them** | ETB: `rules/replacement.rs:2131`, inside `queue_carddef_etb_triggers`' `TriggerCondition::WhenEntersBattlefield` arm — this is Garruk's Uprising's site. Upkeep: `rules/turn_actions.rs:310`, inside the `AtBeginningOfYourUpkeep` / `AtBeginningOfEachUpkeep` CardDef sweep — this is Inventors' Fair's site. |
| (not in the brief) the **resolution-time** half of CR 603.4 is also live for these two | **HOLDS** | Both cards queue as `PendingTriggerKind::CardDefETB`; that path's re-check is `rules/resolution.rs:2337-2352` (`triggered_carddef_iif` → `condition_is_queue_time_evaluable` guard → `check_condition`). The runtime-lowered path's counterpart is `resolution.rs:2437` (`check_intervening_if`, `InterveningIfMoment::Resolution`), added by PB-DX1. Neither card needs the lowered path, but the two agree. |
| `activation_condition` is enforced | **HOLDS** | `rules/abilities.rs:260`, CR 602.5b, inside `activate_ability`'s pre-payment block, reading the **layer-resolved** ability (`expect_characteristics`, CR 613.1f). |

### §1.1 Oracle verification (MCP `lookup_card`, before any marker flip)

**Garruk's Uprising** — `{2}{G}` Enchantment.
> When this enchantment enters, if you control a creature with power 4 or greater, draw a card.
> Creatures you control have trample.
> Whenever a creature you control with power 4 or greater enters, draw a card.

Ruling 2024-11-08: *"If you don't control a creature with power 4 or greater immediately after
Garruk's Uprising enters, its first ability won't trigger. If you don't control one as the
ability resolves, you don't draw a card. They don't have to be the same creature both times."*
→ **both** halves of CR 603.4 are required, exactly what `intervening_if` now provides at both ends.
Ruling 2024-11-08 #2: *"draw just one card, no matter how many"* → `count: EffectAmount::Fixed(1)`
already correct.

**Inventors' Fair** — Legendary Land.
> At the beginning of your upkeep, if you control three or more artifacts, you gain 1 life.
> {T}: Add {C}.
> {4}, {T}, Sacrifice Inventors' Fair: Search your library for an artifact card, reveal it, put
> it into your hand, then shuffle. Activate only if you control three or more artifacts.

Ruling 2016-09-20 #1: *"No player may take actions in a turn before Inventors' Fair's triggered
ability checks to see if it should trigger. If you don't control three or more artifacts, it
won't trigger."* → queue-time half.
Ruling 2016-09-20 #2: *"If you control three artifacts as the ability resolves, you gain 1 life…
If you don't control three artifacts at that time, you won't gain life."* → resolution-time half.
Ruling 2016-09-20 #3: *"When using Inventors' Fair's activated ability, the number of artifacts
you control is checked **only as you activate it**. It's not checked again as the ability
resolves."* → `activation_condition` (CR 602.5b) is the right mechanism; a resolution-time
`Effect::Conditional` wrapper would be **wrong** and must not be added.

---

## §2 The edits

Engine: **none**. `crates/card-defs/src/defs/` only, plus one new integration-test module.

### §2.1 `crates/card-defs/src/defs/garruks_uprising.rs`

1. Delete the stale `// TODO: "If you control creature with power 4+" intervening-if on ETB.` line.
2. On the **first** ability (`TriggerCondition::WhenEntersBattlefield`), replace
   `intervening_if: None` with:

   ```rust
   // CR 603.4 (ruling 2024-11-08): checked BOTH immediately after this enchantment
   // enters (queue time, rules/replacement.rs `queue_carddef_etb_triggers`) and again
   // as the ability resolves (rules/resolution.rs' CardDefETB re-check). The two
   // creatures need not be the same one.
   intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
       count: 1,
       filter: TargetFilter {
           has_card_type: Some(CardType::Creature),
           min_power: Some(4),
           ..Default::default()
       },
   }),
   ```

   **No `controller` field on the filter** — `Condition::YouControlNOrMoreWithFilter` already
   scopes to `obj.controller == controller` in `check_static_condition`
   (`effects/mod.rs:~10208`), and none of the 21 shipped users sets it. **No `exclude_self`** —
   Garruk's Uprising is an Enchantment, never a creature, so it can never satisfy its own filter.
   Power is read through `layers::expect_characteristics`, so pumps/anthems count (ruling
   2024-11-08 #3's spirit).
3. Replace `completeness: Completeness::partial(...)` with `completeness: Completeness::Complete,`
   (explicit, matching `delver_of_secrets.rs:72` / `birthing_pod.rs:87`).

### §2.2 `crates/card-defs/src/defs/inventors_fair.rs`

1. Delete both stale header `// TODO:` lines and the two inline `// TODO:` lines.
2. **Add the missing upkeep trigger** (it is absent entirely today, not merely unconditional):

   ```rust
   // At the beginning of your upkeep, if you control three or more artifacts, you gain 1 life.
   // CR 603.4 (rulings 2016-09-20 #1/#2): checked at queue time
   // (rules/turn_actions.rs' AtBeginningOfYourUpkeep CardDef sweep) and re-checked at
   // resolution; the artifacts need not be the same ones.
   AbilityDefinition::Triggered {
       once_per_turn: false,
       trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
       effect: Effect::GainLife {
           player: PlayerTarget::Controller,
           amount: EffectAmount::Fixed(1),
       },
       intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
           count: 3,
           filter: TargetFilter {
               has_card_type: Some(CardType::Artifact),
               ..Default::default()
           },
       }),
       targets: vec![],
       modes: None,
       trigger_zone: None,
   },
   ```

   Placed **first** in `abilities`, matching the oracle-text order the def already stores.
   Index churn is safe: `ability_index` is derived per-lookup from
   `def.effective_abilities(..)` at both queue and resolution time, the mana ability is lowered
   into `mana_abilities` by `enrich_spec_from_def` rather than by position, and `rg` finds **no**
   test, script or fixture naming `inventors_fair` / "Inventors' Fair".
3. On the **search** ability, replace `activation_condition: None` with:

   ```rust
   // CR 602.5b "Activate only if …" — ruling 2016-09-20 #3: checked ONLY on
   // activation, never re-checked at resolution, so this belongs on
   // activation_condition and NOT in an Effect::Conditional wrapper.
   activation_condition: Some(Condition::YouControlNOrMoreWithFilter {
       count: 3,
       filter: TargetFilter {
           has_card_type: Some(CardType::Artifact),
           ..Default::default()
       },
   }),
   ```

   Note the source itself is a **Land**, not an artifact, so it never self-counts; the count is
   of *other* permanents by construction, matching the printed card.
4. Leave `shuffle_before_placing: false` + the trailing `Effect::Shuffle` as-is — oracle order is
   "put it into your hand, **then** shuffle", and `shuffle_before_placing: true` is the *opposite*
   pattern (Vampiric Tutor's "shuffle, then put on top"; `effects/mod.rs:3633-3638`).
5. Replace `completeness: Completeness::partial(...)` with `completeness: Completeness::Complete,`.

### §2.3 New test module

`crates/engine/tests/primitives/pb_dx3_stale_blocker_notes.rs`, with `mod
pb_dx3_stale_blocker_notes;` added to `crates/engine/tests/primitives/main.rs` in alphabetical
position (after `pb_dx2_command_gates`). **SR-9a**: never a top-level `tests/*.rs`.

Both defs are loaded from the real corpus via `all_cards()` — never re-declared inline — so the
probes test the shipped def, not a copy of it.

---

## §3 Probe roster (all must be *fail-before*)

| # | Card | Asserts | Why it fails before the edit |
|---|---|---|---|
| T1 | Garruk's Uprising | ETB with **no** power-4+ creature → **no** trigger on the stack, **no** card drawn | pre-fix `intervening_if: None` → the trigger always queued and always drew |
| T2 | Garruk's Uprising | ETB with a 4/4 on board → trigger queues, resolves, exactly **one** card drawn | pins the positive direction so T1 can't be satisfied by breaking the card |
| T3 | Garruk's Uprising | queue-time true, then the 4/4 leaves before the trigger resolves → **no** draw (CR 603.4 s2, ruling 2024-11-08) | resolution re-check reads the def's `intervening_if`, which was `None` |
| T4 | Garruk's Uprising | the *third* ability (power-4+ creature ETB draw) still fires — untouched clause regression guard | — |
| T5 | Inventors' Fair | upkeep with **2** artifacts → no trigger queued, life total unchanged | pre-fix the ability did not exist at all → also fails, for the *other* reason; T6 disambiguates |
| T6 | Inventors' Fair | upkeep with **3** artifacts → trigger queues, resolves, life +1 | pre-fix: no such ability, so 0 life gained |
| T7 | Inventors' Fair | queue with 3 artifacts, then one leaves before resolution → **no** life gained (ruling 2016-09-20 #2) | — |
| T8 | Inventors' Fair | `Command::ActivateAbility` on the search ability with **2** artifacts → `Err` | pre-fix `activation_condition: None` permitted the illegal activation |
| T9 | Inventors' Fair | with **3** artifacts the activation is accepted, the ability resolves through PB-DP9's `GameEvent::EffectChoiceRequired` / `Command::AnswerEffectChoice` channel, and the **announced** artifact (not the lowest `ObjectId`) lands in hand; the land is sacrificed and the library is shuffled | end-to-end, per the brief's explicit instruction not to probe only the trigger half |
| T10 | Inventors' Fair | ruling 2016-09-20 #3: the count is **not** re-checked at resolution — activate legally with 3, then remove artifacts before it resolves; the search still happens | guards against "fix" by an `Effect::Conditional` wrapper |

Every test cites its CR / ruling in a doc comment (Architecture Invariant 8).

**Vacuity discipline** (PB-DX2's lesson): each negative probe must be shown to fail *for the
stated reason*. T8's `is_err()` is checked against the **error message** naming the activation
condition, not bare `is_err()` — a wrong ability index or an unpayable cost would also error.

---

## §4 Gates

1. `git diff --stat -- crates/engine/src` → **empty** (only `crates/engine/tests/` moves).
2. `git diff -- crates/engine/src/rules/protocol.rs crates/engine/src/state/hash.rs` → **empty**;
   PROTOCOL **32** / HASH **69**. The `core` group's `protocol_schema::*` / `hash_schema::*`
   suites green.
3. `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo fmt --check`, **and** `tools/check-defs-fmt.sh` (SR-35 — `cargo fmt` checks **zero**
   of the 1,804 defs and still exits 0).
4. `cargo test --all` — full workspace, incl. the 211 golden scripts.
5. **SR-2**: both defs become deck-legal on flip. `validate_deck` will now admit them —
   which is the point, and is why §1.1's oracle verification is a precondition, not a formality.
6. `tools/authoring-report.py` regenerated; expect **1,140 → 1,142** `Complete` (63.2% → 63.3%).

## §5 Risks

- **Legal-but-wrong flip.** Both cards are `partial` today, so a wrong flip *ships* a broken
  card into legal decks. Mitigated by §1.1 (oracle text + all four rulings each) and by T4/T10,
  which pin the clauses this batch does **not** touch.
- **`min_power` semantics.** `matches_filter` (`effects/mod.rs`, symbol `matches_filter`) treats `power: None` as
  *failing* `min_power`, so a `*/*` CDA creature never counts. Correct here (a creature with
  undefined power is not "power 4 or greater" until layers give it one) and T2 uses a printed
  4/4, but worth stating.
- **Index churn on `inventors_fair`** — argued safe in §2.2.2; the runner must re-run `rg` to
  confirm no fixture names the card before reordering.
