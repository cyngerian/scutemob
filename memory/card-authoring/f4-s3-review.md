# Card Review: F-4 Session 3 — Re-authored Abilities

**Reviewed**: 2026-03-22
**Cards**: 6
**Findings**: 0 HIGH, 3 MEDIUM, 2 LOW

---

## Card 1: Yavimaya Hollow

- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (none — land)
- **DSL correctness**: YES
- **Findings**: None

Regenerate ability uses `EffectTarget::DeclaredTarget { index: 0 }` with `TargetRequirement::TargetCreature` — correct for "Regenerate target creature." Cost is `{G}, {T}` via `Cost::Sequence` — correct. Mana tap ability produces colorless via `mana_pool(0, 0, 0, 0, 0, 1)` — correct (WUBRGC order).

---

## Card 2: Skithiryx, the Blight Dragon

- **Oracle match**: YES
- **Types match**: YES (Legendary Creature — Phyrexian Dragon Skeleton, supertypes correct)
- **Mana cost match**: YES ({3}{B}{B} = generic: 3, black: 2)
- **P/T match**: YES (4/4)
- **DSL correctness**: YES
- **Findings**: None

- Flying + Infect keywords: correct.
- `{B}: gains haste until EOT` — uses `ApplyContinuousEffect` with `EffectFilter::Source`, `EffectLayer::Ability`, `EffectDuration::UntilEndOfTurn`, `AddKeyword(Haste)`. All correct for self-grant.
- `{B}{B}: Regenerate` — uses `Effect::Regenerate { target: EffectTarget::Source }`. Correct for "Regenerate Skithiryx" (self, no target).

---

## Card 3: Nezumi Prowler

- **Oracle match**: YES
- **Types match**: YES (Artifact Creature — Rat Ninja)
- **Mana cost match**: YES ({1}{B} = generic: 1, black: 1)
- **P/T match**: YES (3/1)
- **DSL correctness**: YES
- **Findings**: None

- Ninjutsu: has both `Keyword(Ninjutsu)` and `Ninjutsu { cost }` — correct (KI-6 satisfied).
- ETB trigger targets `TargetCreatureWithFilter(TargetFilter { controller: You })` — matches "target creature you control."
- Two `ApplyContinuousEffect` in a `Sequence` for deathtouch + lifelink, both using `EffectFilter::DeclaredTarget { index: 0 }`, `EffectLayer::Ability`, `EffectDuration::UntilEndOfTurn` — all correct.

---

## Card 4: Vivisection Evangelist

- **Oracle match**: YES
- **Types match**: YES (Creature — Phyrexian Cleric)
- **Mana cost match**: YES ({3}{W}{B} = generic: 3, white: 1, black: 1)
- **P/T match**: YES (4/4)
- **DSL correctness**: MOSTLY
- **Findings**:
  - F1 (MEDIUM): Target filter is over-broad. Oracle says "destroy target creature or planeswalker an opponent controls" but the filter uses `non_land: true, controller: Opponent` which also includes artifacts and enchantments. The DSL has `has_card_types: Vec<CardType>` on TargetFilter which could narrow this to `has_card_types: vec![CardType::Creature, CardType::Planeswalker]` with `controller: Opponent`. The `non_land: true` should be removed. The comment on line 17-18 acknowledges this as "slightly broader" but a tighter filter IS expressible.
  - F2 (LOW): File uses explicit struct fields instead of `..Default::default()` (lines 34-41). Not a correctness issue but inconsistent with other card defs.

The `intervening_if: Some(Condition::OpponentHasPoisonCounters(3))` is correct for the Corrupted mechanic. The target is properly in the `targets` vec (not inline) — the fix described in the batch description is confirmed applied.

---

## Card 5: Shizo, Death's Storehouse

- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (none — land)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F3 (MEDIUM): Target filter allows any creature, not just legendary creatures. Oracle says "Target legendary creature gains fear until end of turn." The TODO on lines 24-25 documents this gap. Confirmed: `TargetFilter` lacks a `has_supertype` field, so there is no way to restrict to Legendary creatures currently. The TODO is legitimate — this is a true DSL gap. Not flagged as KI-3.
  - F4 (MEDIUM): W5 policy question — this over-permissive targeting allows granting Fear to non-legendary creatures, producing wrong game behavior. However, the ability still costs mana and a tap, and Fear is a relatively minor keyword. The risk of incorrect game state is low (player would have to intentionally misuse it). Documenting but not escalating to HIGH.

Mana tap produces `mana_pool(0, 0, 1, 0, 0, 0)` — {B} is correct (WUBRGC: W=0, U=0, B=1, R=0, G=0, C=0). Cost is `{B}, {T}` via `Cost::Sequence` — correct. `ApplyContinuousEffect` uses `EffectFilter::DeclaredTarget { index: 0 }`, `EffectLayer::Ability`, `EffectDuration::UntilEndOfTurn`, `AddKeyword(Fear)` — all correct.

---

## Card 6: Otawara, Soaring City

- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (none — land)
- **DSL correctness**: YES
- **Findings**:
  - F5 (LOW): Cost reduction not implemented. Oracle says "This ability costs {1} less to activate for each legendary creature you control." The TODO on line 26 documents this. The DSL has `CostReduction` (PB-8) and `CountPermanents` (PB-7), so this may be expressible. However, cost reduction on activated abilities (not spell costs) may not be wired. This needs further investigation but is not blocking — the card is still playable at full cost.

The key fix (ControllerOf -> OwnerOf) is confirmed correct:
- Line 34: `to: ZoneTarget::Hand { owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })) }`
- This correctly returns the target to "its owner's hand" per oracle text.
- `EffectTarget::DeclaredTarget { index: 0 }` refers to the bounced permanent, and `OwnerOf` extracts that permanent's owner — correct for multiplayer.

Target filter uses `non_land: true` which matches "artifact, creature, enchantment, or planeswalker" (everything except lands) — correct.

---

## Summary

- **Cards with issues**: Vivisection Evangelist (F1 MEDIUM — over-broad target filter, expressible tighter), Shizo Death's Storehouse (F3-F4 MEDIUM — missing legendary filter, true DSL gap), Otawara (F5 LOW — cost reduction TODO)
- **Clean cards**: Yavimaya Hollow, Skithiryx the Blight Dragon, Nezumi Prowler

### Action Items

| ID | Severity | Card | Action |
|----|----------|------|--------|
| F1 | MEDIUM | Vivisection Evangelist | Replace `non_land: true` with `has_card_types: vec![CardType::Creature, CardType::Planeswalker]` and `controller: Opponent` |
| F3 | MEDIUM | Shizo, Death's Storehouse | True DSL gap — TargetFilter needs `has_supertype: Option<SuperType>` field. Keep TODO. No action until DSL extended. |
| F4 | MEDIUM | Shizo, Death's Storehouse | Document W5 risk as acceptable (low impact, targeting is player-controlled). No action needed. |
| F5 | LOW | Otawara, Soaring City | Investigate whether activated-ability cost reduction is wirable via existing `self_cost_reduction` or `CostReduction`. Low priority. |
