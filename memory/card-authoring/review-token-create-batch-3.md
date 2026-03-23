# Card Review: Token-Create Batch 3 (S47-S49)

**Reviewed**: 2026-03-23
**Cards**: 5
**Findings**: 2 HIGH, 1 MEDIUM, 0 LOW

---

## Card 1: Bitterblossom
- **Oracle match**: YES
- **Types match**: NO
- **Mana cost match**: YES
- **DSL correctness**: YES (trigger, token spec, LoseLife all correct)
- **Findings**:
  - F1 (HIGH / KI-4): **Missing `CardType::Kindred` in type line.** Oracle type is "Kindred Enchantment -- Faerie". Def uses `types_sub(&[CardType::Enchantment], &["Faerie"])` -- missing `CardType::Kindred`. Should be `types_sub(&[CardType::Kindred, CardType::Enchantment], &["Faerie"])`. See `boggart_shenanigans.rs` for reference.
  - Token spec is correct: 1/1 black Faerie Rogue with Flying, count 1.
  - `PlayerTarget::Controller` for LoseLife is correct ("you lose 1 life").
  - `AtBeginningOfYourUpkeep` trigger is correct.

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/bitterblossom.rs`

---

## Card 2: Oko, Thief of Crowns
- **Oracle match**: YES
- **Types match**: YES (Legendary Planeswalker -- Oko)
- **Mana cost match**: YES ({1}{G}{U})
- **DSL correctness**: YES
- **Findings**: CLEAN
  - +2 Food token ability is correctly implemented with `food_token_spec(1)`.
  - +1 (Elkify) and -5 (exchange control) are correctly stubbed with `Effect::Nothing` and TODOs. Both effects (lose-all-abilities + become-creature-with-base-P/T, and exchange-control) are genuinely not expressible in the current DSL. TODOs are valid.
  - `starting_loyalty: Some(4)` is correct.
  - All three abilities use `LoyaltyAbility` with correct costs (+2, +1, -5).

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/oko_thief_of_crowns.rs`

---

## Card 3: Sengir Autocrat
- **Oracle match**: YES
- **Types match**: YES (Creature -- Human)
- **Mana cost match**: YES ({3}{B})
- **P/T match**: YES (2/2)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F2 (MEDIUM / KI-2 / W5): **Partial implementation produces wrong game state.** The ETB creates three 0/1 black Serf tokens, but the LTB ("When this creature leaves the battlefield, exile all Serf tokens") is missing due to a genuine DSL gap (`WhenLeavesBattlefield` not in `TriggerCondition`). Without the LTB, Serf tokens persist indefinitely after Sengir Autocrat is removed. This gives the controller extra 0/1 bodies that oracle says should be exiled. Per W5 policy, abilities should be `vec![]` with a TODO explaining both the ETB and LTB are a package deal, OR the current partial implementation should be documented as producing wrong game state. The TODO for the LTB gap is valid (KI-19 satisfied), but the partial-impl consequence is not called out.
  - Token spec is correct: 0/1 black Serf, count 3.
  - The `WhenLeavesBattlefield` DSL gap TODO is valid -- confirmed no such variant exists.

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/sengir_autocrat.rs`

---

## Card 4: Krenko's Command
- **Oracle match**: YES
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES ({1}{R})
- **DSL correctness**: YES
- **Findings**: CLEAN
  - Token spec is correct: 1/1 red Goblin, count 2.
  - `AbilityDefinition::Spell` is correct for a sorcery.
  - No targets needed -- correct.

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/krenkos_command.rs`

---

## Card 5: Mist-Syndicate Naga
- **Oracle match**: YES
- **Types match**: YES (Creature -- Snake Ninja)
- **Mana cost match**: YES ({2}{U})
- **P/T match**: YES (3/1)
- **DSL correctness**: YES
- **Findings**: CLEAN
  - Ninjutsu is correctly implemented with both `Keyword(Ninjutsu)` AND `Ninjutsu { cost }` (KI-6 satisfied).
  - Ninjutsu cost {2}{U} matches oracle.
  - `WhenDealsCombatDamageToPlayer` trigger is correct for "Whenever this creature deals combat damage to a player".
  - `CreateTokenCopy { source: EffectTarget::Source }` is correct for "create a token that's a copy of this creature".
  - `enters_tapped_and_attacking: false` is correct -- oracle does not specify the copy enters tapped/attacking.

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/mist_syndicate_naga.rs`

---

## Summary

- **Cards with issues**:
  - **Bitterblossom** -- 1 HIGH: missing `CardType::Kindred` in type line
  - **Sengir Autocrat** -- 1 MEDIUM: partial implementation (ETB without LTB) produces wrong game state
- **Clean cards**: Oko Thief of Crowns, Krenko's Command, Mist-Syndicate Naga
