# PB-XSE1 — OOS-XS-E-1: Dies-side `exclude_self` Audit

**Task**: scutemob-27
**Date**: 2026-05-15
**Branch**: `feat/oos-xs-e-1-verify-dies-side-cards-use-whenevercreaturediesex`
**Scope**: 3 cards named in the PB-XS-E follow-up roster — Boggart Shenanigans, Athreos God of Passage, Meren of Clan Nel Toth.
**CR**: CR 109.1 (object identity — "another"), CR 603.10a (death triggers / LKI), CR 603.2 (triggered-ability dispatch).

## Audit Method

For each card:
1. Read the current card definition file under `crates/engine/src/cards/defs/`.
2. Look up oracle text via MCP `lookup_card`.
3. Compare oracle text to current implementation of any `TriggerCondition::WheneverCreatureDies` trigger.
4. Record decision: `exclude_self = true` if oracle says "another"; `exclude_self = false` otherwise.

The structure being audited is `TriggerCondition::WheneverCreatureDies { controller, exclude_self, nontoken_only, filter }` defined at `crates/engine/src/cards/card_definition.rs:2760`. The `exclude_self` field was wired to the runtime `DeathTriggerFilter::exclude_self` in PB-23 (currently HASH 22) and is enforced in `crates/engine/src/rules/abilities.rs` graveyard dispatch path.

---

## Card 1: Boggart Shenanigans

**File**: `crates/engine/src/cards/defs/boggart_shenanigans.rs`
**MCP oracle text** (verified 2026-05-15):

> Whenever **another** Goblin you control is put into a graveyard from the battlefield, you may have this enchantment deal 1 damage to target player or planeswalker.

**Has "another"**: YES → `exclude_self` should be `true` when implemented.

**Current implementation**: `abilities: vec![]` (empty). Top-of-file TODO comment explains the DSL gap: the trigger requires `WheneverCreatureDies` plus a subtype filter (Goblin) plus a controller filter (You) plus exclude-self. The subtype-filter half is the blocker; the card is intentionally left without the trigger until a filtered variant exists.

**Decision**: **No change required**. There is no `WheneverCreatureDies` trigger in the current card def to set `exclude_self` on. When this card is eventually implemented (after the subtype-filter DSL gap closes), the implementer must set `exclude_self: true` to match the oracle's "another" qualifier.

---

## Card 2: Athreos, God of Passage

**File**: `crates/engine/src/cards/defs/athreos_god_of_passage.rs`
**MCP oracle text** (verified 2026-05-15):

> Indestructible
> As long as your devotion to white and black is less than seven, Athreos isn't a creature.
> Whenever **another** creature you own dies, return it to your hand unless target opponent pays 3 life.

**Has "another"**: YES → `exclude_self` should be `true` when implemented.

**Acceptance-criterion discrepancy** (FLAG): AC 3893 states "Athreos: 'each time a creature you own dies' → false (not 'another')". The MCP-verified oracle text — and the canonical Scryfall oracle — both contain the word "another". The AC's claimed oracle wording is incorrect. The correct disposition for Athreos is `exclude_self: true`, matching the other two cards in this roster. This was double-checked against `mcp__mtg-rules__lookup_card` for "Athreos, God of Passage" and is consistent with Athreos's 2014 Theros printing and the 2020 oracle update.

**Current implementation**: `abilities` contains `Indestructible` + a Layer-4 `RemoveCardTypes` static for the devotion-gated creature-type removal. The death trigger itself is **not implemented** — the file's bottom TODO comment notes the DSL gap: "no mechanic for opponent paying life as an alternative to an effect."

**Decision**: **No change required**. There is no `WheneverCreatureDies` trigger in the current card def to set `exclude_self` on. When the opponent-pays-life DSL gap closes, the implementer must set `exclude_self: true` (oracle says "another") — NOT `false` as AC 3893 suggests.

---

## Card 3: Meren of Clan Nel Toth

**File**: `crates/engine/src/cards/defs/meren_of_clan_nel_toth.rs`
**MCP oracle text** (verified 2026-05-15):

> Whenever **another** creature you control dies, you get an experience counter.
> At the beginning of your end step, choose target creature card in your graveyard. If that card's mana value is less than or equal to the number of experience counters you have, return it to the battlefield. Otherwise, put it into your hand.

**Has "another"**: YES → `exclude_self` should be `true` when implemented.

**Current implementation**: `abilities: vec![]` (empty). Inline TODO comment: "DSL gap — 'another creature you control dies' trigger with controller filter + experience counter grant + end step trigger with MV comparison." Experience counters and the end-step targeted-graveyard logic are the blockers, not exclude_self itself.

**Decision**: **No change required**. There is no `WheneverCreatureDies` trigger in the current card def to set `exclude_self` on. When this card is eventually implemented, the implementer must set `exclude_self: true` to match the oracle's "another" qualifier.

---

## Summary Table

| Card | Oracle "another"? | Correct `exclude_self` | Current Trigger Implemented? | Action |
|---|---|---|---|---|
| Boggart Shenanigans | YES | `true` | NO (DSL gap: subtype filter) | None — record decision for future implementer |
| Athreos, God of Passage | YES | `true` | NO (DSL gap: opponent-pays-life alt) | None — record decision; FLAG: AC 3893 wording incorrect |
| Meren of Clan Nel Toth | YES | `true` | NO (DSL gap: experience counters + targeted end-step) | None — record decision for future implementer |

## Engine Touch

- No engine files changed.
- No card-def `.rs` files changed (none have a `WheneverCreatureDies` trigger to mutate).
- No new tests added — AC 3894 is conditional on a card needing a fix; none did.
- HASH stays at 22 (no engine schema change), matching AC 3895.

## Build / Test / Lint

Pre-existing baseline expected to remain green. The audit added only this Markdown plan document under `memory/primitives/`.

## Why No Unit Test Is Added

AC 3894: "If any card needed a fix, add a unit test asserting the trigger behavior...". No card needed a fix in this PR — all three cards have the `WheneverCreatureDies` trigger marked TODO in their `abilities` array, so there is no `exclude_self` value to assert. The general PB-23 / PB-XS-E test surface already covers the `exclude_self: true` semantics in `crates/engine/tests/primitive_pb_xs_e.rs` (creature-side) and the PB-23 death-trigger tests (graveyard dispatch). Adding a Meren-shaped scenario when Meren itself does not yet have the trigger wired up would test the engine's generic behavior — already covered — without exercising the card's own definition.

When the three blocking DSL gaps close and these cards' triggers are wired up (separate future PBs), the implementing PB should include card-specific regression tests asserting:
- Source dies: trigger does NOT fire when `exclude_self: true`.
- Another matching permanent dies: trigger DOES fire.

This audit's job is to lock the correct `exclude_self` decision into the historical record so the future implementer doesn't have to re-derive it.

## Disposition

- OOS-XS-E-1: **AUDIT-COMPLETE** with finding "all three cards correctly classified `exclude_self: true` when implemented; no current trigger exists in any of the three card defs; AC 3893 oracle wording for Athreos is incorrect — actual oracle says 'another', so Athreos should also be `true`."
