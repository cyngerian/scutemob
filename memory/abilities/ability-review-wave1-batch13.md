# Wave 1 Batch 13 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 5 MEDIUM, 5 LOW

---

## Card: Temple of Deceit
- **card_id**: OK (`temple-of-deceit`)
- **name**: OK
- **types/subtypes**: OK (Land, no subtypes)
- **oracle_text**: OK -- matches Scryfall exactly
- **mana_cost**: OK (None)
- **abilities**: Skeleton (`vec![]`) with TODOs
- **Findings**:
  - F1 (MEDIUM): ETB tapped is implementable via `AbilityDefinition::Replacement { trigger: ReplacementTrigger::WouldEnterBattlefield { filter: ObjectFilter::Any }, modification: ReplacementModification::EntersTapped, is_self: true }` (see lonely_sandbar.rs, path_of_ancestry.rs). Left as TODO.
  - F2 (MEDIUM): Tap-for-mana ability is implementable via `AbilityDefinition::Activated { cost: Cost::Tap, effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(...) } }`. Left as TODO. This card produces U or B, which means two separate activated abilities (one for U, one for B).
  - F3 (MEDIUM): ETB scry 1 trigger is implementable via `AbilityDefinition::Triggered` with `SelfEntersBattlefield` condition and `Effect::Scry { count: EffectAmount::Fixed(1) }` effect (see read_the_bones.rs for Scry effect usage). Left as TODO.
- **Verdict**: MEDIUM -- all three abilities are implementable in the current DSL but left as skeleton TODOs.

---

## Card: Dimir Aqueduct
- **card_id**: OK (`dimir-aqueduct`)
- **name**: OK
- **types/subtypes**: OK (Land, no subtypes)
- **oracle_text**: OK -- matches Scryfall exactly
- **mana_cost**: OK (None)
- **abilities**: Skeleton (`vec![]`) with TODOs
- **Findings**:
  - F4 (MEDIUM): ETB tapped is implementable (same pattern as F1). Left as TODO.
  - F5 (LOW): Tap-for-UB (both at once) is implementable via `Effect::AddMana { mana: mana_pool(0, 1, 1, 0, 0, 0) }`. Left as TODO. This is a single ability producing both U and B simultaneously, unlike Temple of Deceit which offers a choice.
  - F6 (LOW): ETB bounce-a-land trigger is a DSL gap (targeted_trigger -- "return a land you control to its owner's hand" requires targeting a land you control and returning it). Correctly left as TODO.
- **Verdict**: MEDIUM -- ETB tapped is implementable but left as skeleton. Bounce trigger is a genuine DSL gap. Tap-for-mana is implementable.

---

## Card: Skemfar Elderhall
- **card_id**: OK (`skemfar-elderhall`)
- **name**: OK
- **types/subtypes**: OK (Land, no subtypes -- Scryfall confirms no Legendary supertype)
- **oracle_text**: OK -- matches Scryfall exactly
- **mana_cost**: OK (None)
- **abilities**: Skeleton (`vec![]`) with TODOs
- **Findings**:
  - F7 (MEDIUM): ETB tapped is implementable (same pattern as F1). Left as TODO.
  - F8 (MEDIUM): Tap-for-G mana ability is implementable via `AbilityDefinition::Activated { cost: Cost::Tap, effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 0, 1, 0) } }`. Left as TODO.
  - F9 (LOW): Complex activated ability ({2}{B}{B}{G}, {T}, Sacrifice: -2/-2 + create two Elf Warrior tokens, sorcery-only) is a DSL gap -- combines multiple costs (mana + tap + sacrifice), targeted debuff, and token creation in a single activated ability. Correctly left as TODO.
- **Verdict**: MEDIUM -- ETB tapped and tap-for-G are implementable but left as skeleton.

---

## Card: Arixmethes, Slumbering Isle
- **card_id**: OK (`arixmethes-slumbering-isle`)
- **name**: OK
- **types/subtypes**: OK (Legendary Creature -- Kraken; uses `full_types` correctly)
- **oracle_text**: OK -- matches Scryfall exactly
- **mana_cost**: OK ({2}{G}{U} = generic 2, green 1, blue 1)
- **power/toughness**: OK (12/12)
- **abilities**: Skeleton (`vec![]`) with TODOs
- **Findings**:
  - F10 (LOW): All four abilities are complex DSL gaps: ETB tapped with slumber counters, continuous type-changing effect based on counter presence, spell-cast trigger to remove counters, tap-for-GU mana. The type-changing static ability and counter-conditional behavior are genuine DSL gaps. Correctly left as TODO.
  - F11 (LOW): The tap-for-GU mana ability (`Effect::AddMana { mana: mana_pool(0, 1, 0, 0, 1, 0) }`) could theoretically be expressed, but it only functions when Arixmethes is a land (has slumber counters), which requires the static type-changing ability to work first. Reasonable to leave as skeleton.
- **Verdict**: LOW -- complex card with genuine DSL gaps for most abilities; skeleton is appropriate.

---

## Card: Undercity Sewers
- **card_id**: OK (`undercity-sewers`)
- **name**: OK
- **types/subtypes**: OK (Land -- Island Swamp; uses `types_sub` correctly)
- **oracle_text**: OK -- matches Scryfall exactly
- **mana_cost**: OK (None)
- **abilities**: Skeleton (`vec![]`) with TODOs
- **Findings**:
  - F12 (MEDIUM): ETB tapped is implementable (same pattern as F1). Left as TODO.
  - F13 (LOW): The `({T}: Add {U} or {B}.)` reminder text is an intrinsic ability granted by the Island and Swamp subtypes. It does not need to be explicitly defined in abilities (the engine should grant it from subtypes). The TODO is fine but could note this is intrinsic.
  - F14 (MEDIUM): ETB surveil 1 trigger is likely implementable -- Surveil is implemented as an Effect (see engine). Similar to scry ETB on temples: `AbilityDefinition::Triggered` with `SelfEntersBattlefield` and `Effect::Surveil { count: EffectAmount::Fixed(1) }`. Left as TODO.
- **Verdict**: MEDIUM -- ETB tapped and ETB surveil 1 are implementable but left as skeleton.

---

## Summary

- **Cards with issues**:
  - Temple of Deceit (MEDIUM) -- 3 implementable abilities left as TODO
  - Dimir Aqueduct (MEDIUM) -- ETB tapped + tap-for-mana implementable, left as TODO
  - Skemfar Elderhall (MEDIUM) -- ETB tapped + tap-for-G implementable, left as TODO
  - Undercity Sewers (MEDIUM) -- ETB tapped + ETB surveil implementable, left as TODO
  - Arixmethes, Slumbering Isle (LOW) -- genuine DSL gaps, skeleton appropriate

- **Clean cards**: None (all have at least implementable abilities left as TODO)

- **Common pattern**: All 4 land cards with unconditional ETB tapped have the same issue -- this ability is well-established in the DSL (lonely_sandbar.rs, path_of_ancestry.rs, fire_diamond.rs, etc.) but was left as a TODO skeleton. The Phase 1 template generator likely did not emit these patterns. Similarly, basic tap-for-mana abilities are implementable but were skipped.

**HIGH: 0 | MEDIUM: 5 | LOW: 5**
