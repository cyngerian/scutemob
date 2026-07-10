# Card Review: PB-AC9 Backfill (commit 52a2b6f2)

**Reviewed**: 2026-07-10
**Cards**: 13
**Findings**: 1 HIGH, 0 MEDIUM, 1 LOW

Engine primitives confirmed present via grep:
- `Effect::WheelHand` + `WheelDisposal::{Discard,ShuffleHandIntoLibrary,ShuffleHandAndGraveyardIntoLibrary}` + `WheelDraw::{ThatMany,Fixed}` — `effects/mod.rs:560`, correct snapshot-before-disposal ordering.
- `Effect::SetNoMaximumHandSize` — `effects/mod.rs:601`, sets persistent `no_max_hand_size_permanent`.
- `Effect::RollDice` / `EffectAmount::LastDiceRoll` — pre-existing, in use.
- `ReplacementModification::{DoubleTokens,DoubleCounters}`, `ReplacementTrigger::{WouldCreateTokens,WouldPlaceCounters}` — pre-existing, in use by Vorinclex/Corpsejack/etc.
- `KeywordAbility::Miracle` + `AbilityDefinition::Miracle { cost }` — **IMPLEMENTED** (`terminus.rs`, `temporal_mastery.rs`, `casting.rs` full support, `Command::ChooseMiracle`, `MiracleTrigger`).

---

## Card 1: Incendiary Command
- Oracle match: YES; Types: YES (Sorcery); Mana: YES ({3}{R}{R}); DSL: YES
- Choose two (min/max 2, no dupes) correct. Mode 0 = 4 dmg to target player/planeswalker; Mode 1 = 2 dmg to each creature (no target); Mode 2 = destroy target nonbasic land (`nonbasic: true`, `has_card_type: Land`); Mode 3 = `WheelHand{EachPlayer, Discard, ThatMany}`. Per-mode targets align with mode list.
- Findings: none. Mode 3 (previously ENGINE-BLOCKED) correctly re-authored; marker gone.

## Card 2: Shattered Perception
- Oracle match: YES; Types: YES; Mana: YES ({2}{R}); DSL: YES
- `WheelHand{Controller, Discard, ThatMany}` matches "Discard all the cards in your hand, then draw that many." Flashback dual-def present (`Keyword(Flashback)` + `AltCastAbility{Flashback, {5}{R}}`) — correct cost.
- Findings: none.

## Card 3: Winds of Change
- Oracle match: YES; Types: YES; Mana: YES ({R}); DSL: YES
- `WheelHand{EachPlayer, ShuffleHandIntoLibrary, ThatMany}` matches oracle exactly.
- Findings: none.

## Card 4: Echo of Eons
- Oracle match: YES; Types: YES; Mana: YES ({4}{U}{U}); DSL: YES
- `WheelHand{EachPlayer, ShuffleHandAndGraveyardIntoLibrary, Fixed(7)}` — verified `effects/mod.rs:578` shuffles BOTH `Hand` and `Graveyard` zones into library before drawing 7. Marker correctly deleted; hand AND graveyard both handled (the specific risk called out in the brief is clear). Flashback dual-def present ({2}{U}) — correct.
- Findings: none.

## Card 5: Ancient Silver Dragon
- Oracle match: YES; Types: YES (Creature — Elder Dragon); Mana: YES ({6}{U}{U}); P/T: 8/8 YES; DSL: YES
- Flying keyword present. Combat-damage trigger = `Sequence[RollDice(d20 -> DrawCards LastDiceRoll to Controller), SetNoMaximumHandSize{Controller}]`. "You"=Controller correct. One triggered ability, idempotent flag set. Draw uses `LastDiceRoll`.
- Findings: none.

## Card 6: Ancient Copper Dragon
- Oracle match: YES; Types: YES; Mana: YES ({4}{R}{R}); P/T: 6/5 YES; DSL: YES
- Combat-damage trigger = RollDice(d20 -> CreateToken Treasure, count=LastDiceRoll). Uses `..treasure_token_spec(1)` with count overridden to `LastDiceRoll`. "You create" = Controller (default CreateToken recipient). Token doubling composes via CreateToken chokepoint.
- Findings: none.

## Card 7: Ancient Gold Dragon
- Oracle match: YES; Types: YES; Mana: YES ({5}{W}{W}); P/T: 7/10 YES; DSL: YES
- RollDice(d20 -> CreateToken: 1/1 blue Faerie Dragon, Flying, count=LastDiceRoll). Colors/subtypes/keywords/P-T all match "1/1 blue Faerie Dragon creature tokens with flying."
- Findings: none.

## Card 8: Parallel Lives
- Oracle match: YES; Types: YES (Enchantment); Mana: YES ({3}{G}); DSL: YES
- `Replacement{WouldCreateTokens(controller), DoubleTokens}`. Matches token-doubling oracle.
- Findings: none.

## Card 9: Anointed Procession
- Oracle match: YES; Types: YES; Mana: YES ({3}{W}); DSL: YES
- Same token-doubling replacement as Parallel Lives (uses `full_types(&[],&[Enchantment],&[])` — equivalent, no supertypes needed).
- Findings: none.

## Card 10: Doubling Season
- Oracle match: YES; Types: YES; Mana: YES ({4}{G}); DSL: YES
- Token-doubling replacement + counter-doubling replacement. Counter clause scoped to `receiver_filter: ObjectFilter::ControlledBy(controller)` — correctly limited to "a permanent you control" (does NOT double counters placed on players), distinct from Vorinclex's `ObjectFilter::Any`. Both clauses present.
- Findings: none.

## Card 11: Reforge the Soul
- Oracle match: YES; Types: YES (Sorcery); Mana: YES ({3}{R}{R}); DSL: PARTIAL
- Wheel body correct: `WheelHand{EachPlayer, Discard, Fixed(7)}` = "Each player discards their hand, then draws seven cards." Fixes the old `DiscardCards{Fixed(7)}` approximation.
- **F1 (HIGH, KI-3 / KI-6): STALE MARKER.** The `// TODO: Miracle {1}{R} — KeywordAbility::Miracle not yet implemented.` claim is FALSE. `KeywordAbility::Miracle` AND `AbilityDefinition::Miracle { cost }` are fully implemented and in active use by `terminus.rs` and `temporal_mastery.rs` (full casting path in `rules/casting.rs`, `Command::ChooseMiracle`, `MiracleTrigger`). This card should be completed by adding the standard Miracle dual-def, exactly like Terminus:
  ```
  AbilityDefinition::Keyword(KeywordAbility::Miracle),
  AbilityDefinition::Miracle { cost: ManaCost { generic: 1, red: 1, ..Default::default() } },
  ```
  Without these, Reforge the Soul cannot be cast for its miracle cost — a missing alternative cast option. This is the exact PB-AC8 failure mode (a marker naming a primitive that already exists, needlessly leaving a card unfinished). Keywords list from Scryfall confirms `["Miracle"]`.

## Card 12: Adrix and Nev, Twincasters
- Oracle match: YES; Types: YES (Legendary Creature — Merfolk Wizard); Mana: YES ({2}{G}{U}); P/T: 2/2 YES; DSL: YES
- Ward {2} present (`Keyword(Ward(2))`). Token-doubling replacement genuinely present (`WouldCreateTokens(controller) -> DoubleTokens`) — NOT silently omitted. Explicit struct construction includes `color_indicator: None`, `back_face: None` per convention.
- Findings: none. (0-marker claim verified: doubling is real.)

## Card 13: Elspeth, Storm Slayer
- Oracle match: YES; Types: YES (Legendary Planeswalker — Elspeth); Mana: YES ({3}{W}{W}); Loyalty 5 YES; DSL: YES
- Token-doubling static genuinely present (`WouldCreateTokens(controller) -> DoubleTokens`) — NOT omitted. +1 makes 1/1 white Soldier. 0 = +1/+1 counter on each creature you control + flying until your next turn (ForEach AddCounter + ApplyContinuousEffect AddKeyword(Flying), UntilYourNextTurn). -3 = destroy target opponent creature MV>=3 (`controller: Opponent`, `min_cmc: Some(3)`).
- **F2 (LOW): "Those creatures gain flying" set-locking.** The 0 ability grants flying via `EffectFilter::CreaturesYouControl` for `UntilYourNextTurn`, which is a live filter — it grants flying to ANY creature you control until your next turn (including ones that enter afterward, and drops creatures that leave). Oracle "Those creatures gain flying" refers to the fixed set that received counters at resolution. Minor gameplay edge (extra creatures gaining flying); common engine approximation, pre-existing, not introduced by PB-AC9.

---

## Summary
- Cards with issues: Reforge the Soul (F1 HIGH — stale Miracle marker; wheel body correct but card left needlessly incomplete), Elspeth Storm Slayer (F2 LOW — "those creatures" set not locked).
- Clean cards: Incendiary Command, Shattered Perception, Winds of Change, Echo of Eons, Ancient Silver Dragon, Ancient Copper Dragon, Ancient Gold Dragon, Parallel Lives, Anointed Procession, Doubling Season, Adrix and Nev.
- Doubling verified real (not omitted) on all 5 token-doublers incl. Adrix/Nev and Elspeth.
- Echo of Eons hand+graveyard shuffle verified correct at the effect implementation level.
