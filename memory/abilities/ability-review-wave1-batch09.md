# Wave 1 Batch 9 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 5 MEDIUM, 0 LOW

All 5 cards are Phase 2 skeletons with `abilities: vec![]` and TODO comments. Oracle text, types, subtypes, and card_id slugs match Scryfall for all cards. The issue across the batch is that ETB tapped + mana tap abilities are fully implementable in the current DSL but were left as TODOs.

---

## Card: Golgari Rot Farm
- card_id: OK (`golgari-rot-farm`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- all three abilities left as TODO
  - ETB tapped: implementable via `AbilityDefinition::Replacement { trigger: WouldEnterBattlefield { filter: ObjectFilter::Any }, modification: EntersTapped, is_self: true }`
  - Bounce trigger ("return a land you control to its owner's hand"): DSL gap (targeted_trigger)
  - Mana ability (`{T}: Add {B}{G}`): implementable via `AbilityDefinition::Activated` with `Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 1, 0, 1, 0) }`
- Verdict: **MEDIUM** -- ETB tapped and mana ability are implementable but left as TODO

**F1 (MEDIUM)**: ETB tapped is implementable via `ReplacementModification::EntersTapped` (see `blooming_marsh.rs` reference). Should not be left as TODO.

**F2 (MEDIUM)**: Mana ability `{T}: Add {B}{G}` is implementable via `AbilityDefinition::Activated` with `Cost::Tap` and `Effect::AddMana`. Should not be left as TODO. Note: this produces both colors simultaneously (not a choice), matching the oracle text.

---

## Card: Temple of Silence
- card_id: OK (`temple-of-silence`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- all three abilities left as TODO
  - ETB tapped: implementable (same pattern as above)
  - ETB scry 1: implementable via `AbilityDefinition::Triggered { trigger_condition: TriggerCondition::WhenEntersBattlefield, effect: Effect::Scry { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) }, intervening_if: None }`
  - Mana ability (`{T}: Add {W} or {B}`): implementable via `AbilityDefinition::Activated` with `Effect::Choose` between two `AddMana` effects (see `blooming_marsh.rs` reference)
- Verdict: **MEDIUM** -- all three abilities are fully implementable but left as TODO

**F3 (MEDIUM)**: All three abilities (ETB tapped, ETB scry 1, dual mana tap) are implementable in the current DSL. ETB scry uses `TriggerCondition::WhenEntersBattlefield` + `Effect::Scry` (see `consider.rs`, `viscera_seer.rs`). The "{W} or {B}" mana uses `Effect::Choose` (see `blooming_marsh.rs`).

---

## Card: Orzhov Basilica
- card_id: OK (`orzhov-basilica`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- all three abilities left as TODO
  - ETB tapped: implementable
  - Bounce trigger: DSL gap (targeted_trigger)
  - Mana ability (`{T}: Add {W}{B}`): implementable
- Verdict: **MEDIUM** -- ETB tapped and mana ability are implementable but left as TODO

**F4 (MEDIUM)**: Same as Golgari Rot Farm -- ETB tapped and mana ability (`{W}{B}`) are implementable but left as TODO.

---

## Card: Underground Mortuary
- card_id: OK (`underground-mortuary`)
- name: OK
- types/subtypes: OK (Land -- Swamp Forest, uses `types_sub`)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- all three abilities left as TODO
  - Intrinsic mana ability from basic land types (Swamp Forest): handled by the engine's basic land type system, not a CardDef ability -- OK to omit
  - ETB tapped: implementable
  - ETB surveil 1: implementable via `AbilityDefinition::Triggered { trigger_condition: TriggerCondition::WhenEntersBattlefield, effect: Effect::Surveil { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) }, intervening_if: None }`
- Verdict: **MEDIUM** -- ETB tapped and ETB surveil 1 are implementable but left as TODO

**F5 (MEDIUM)**: ETB tapped and ETB surveil 1 are both implementable. `Effect::Surveil` exists (see `consider.rs` reference). The intrinsic `{T}: Add {B} or {G}` from Swamp Forest types is handled by the engine automatically, so the TODO comment for it is misleading but harmless.

---

## Card: Rakdos Carnarium
- card_id: OK (`rakdos-carnarium`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: skeleton -- all three abilities left as TODO
  - ETB tapped: implementable
  - Bounce trigger: DSL gap (targeted_trigger)
  - Mana ability (`{T}: Add {B}{R}`): implementable
- Verdict: **MEDIUM** -- ETB tapped and mana ability are implementable but left as TODO

**F6 (MEDIUM)**: Same pattern as Golgari Rot Farm and Orzhov Basilica -- ETB tapped and mana ability (`{B}{R}`) are implementable but left as TODO.

---

## Summary

| Card | Verdict | Issues |
|------|---------|--------|
| Golgari Rot Farm | MEDIUM | F1, F2: ETB tapped + mana ability implementable but left as TODO |
| Temple of Silence | MEDIUM | F3: All 3 abilities implementable but left as TODO |
| Orzhov Basilica | MEDIUM | F4: ETB tapped + mana ability implementable but left as TODO |
| Underground Mortuary | MEDIUM | F5: ETB tapped + surveil ETB implementable but left as TODO |
| Rakdos Carnarium | MEDIUM | F6: ETB tapped + mana ability implementable but left as TODO |

**HIGH: 0 | MEDIUM: 5 | LOW: 0**

All 5 cards have correct metadata (card_id, name, types, subtypes, oracle_text). The consistent issue is that Phase 2 skeleton generation left all abilities as TODO comments, but ETB tapped, mana tap abilities, and ETB scry/surveil triggers are all expressible in the current DSL.

### DSL gap note
The bounce lands (Golgari Rot Farm, Orzhov Basilica, Rakdos Carnarium) have a bounce trigger ("return a land you control to its owner's hand") that requires targeted_trigger -- this is a known DSL gap and correctly left as TODO.

### Reference implementations
- ETB tapped: `blooming_marsh.rs` (lines 14-20)
- Dual mana (both colors): `blooming_marsh.rs` (lines 21-31) -- but note bounce lands add BOTH colors, not a choice, so they should use a single `AddMana` not `Effect::Choose`
- ETB scry: pattern from `viscera_seer.rs` (activated) and `consider.rs` (spell) -- triggered ETB scry would combine `WhenEntersBattlefield` + `Effect::Scry`
- ETB surveil: `consider.rs` (lines 14-17) shows `Effect::Surveil` syntax
