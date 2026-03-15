# PB-10 Plan: Return From Zone Effects

## Overview

Add graveyard-targeted return effects to the DSL. 8 cards blocked on this primitive.
No new keyword abilities — this is a targeting infrastructure extension.

## CR References

- CR 115.1: Targets must be declared as part of casting/activating
- CR 115.7: Target legality re-checked at resolution; illegal = fizzle
- CR 400.7: Zone-changed object is a new object (but graveyard cards keep identity until moved)
- CR 608.2b: Spell/ability checks target legality at resolution

## Cards Covered (8)

| Card | Zone | Filter | Destination |
|------|------|--------|-------------|
| Bloodline Necromancer | your GY | Vampire OR Wizard creature | BF |
| Bladewing the Risen | your GY | Dragon permanent | BF |
| Nullpriest of Oblivion | your GY | creature (if kicked) | BF |
| Buried Ruin | your GY | artifact | Hand |
| Hall of Heliod's Generosity | your GY | enchantment | Library top |
| Emeria, the Sky Ruin | your GY | creature (7+ Plains) | BF |
| Reanimate | any GY | creature | Your BF |
| Teneb, the Harvester | any GY | creature (pay 2B) | Your BF |

## Engine Changes

### 1. TargetRequirement — new variants (card_definition.rs)

```rust
/// "target creature card from your graveyard" — card in controller's graveyard
TargetCardInYourGraveyard(TargetFilter),
/// "target creature card from a graveyard" — card in any player's graveyard
TargetCardInGraveyard(TargetFilter),
```

Two variants needed:
- `TargetCardInYourGraveyard` — Bloodline Necromancer, Bladewing, Nullpriest, Buried Ruin, Hall, Emeria
- `TargetCardInGraveyard` — Reanimate, Teneb (target creature in ANY graveyard)

### 2. TargetFilter — add `has_subtypes` (card_definition.rs)

```rust
/// Subtype constraint (OR semantics — card must have at least one).
/// Used for "Vampire or Wizard creature card" (Bloodline Necromancer).
#[serde(default)]
pub has_subtypes: Vec<SubType>,
```

Keep existing `has_subtype: Option<SubType>` for backward compat. New `has_subtypes: Vec<SubType>`
uses OR semantics: card must have at least one of the listed subtypes. Empty vec = no restriction.

### 3. matches_filter — extend for has_subtypes (effects/mod.rs)

After the existing `has_subtype` check, add:
```rust
if !filter.has_subtypes.is_empty() {
    if !filter.has_subtypes.iter().any(|st| chars.subtypes.contains(st)) {
        return false;
    }
}
```

### 4. validate_object_satisfies_requirement — graveyard zone check (casting.rs)

Add a new match block for graveyard targeting BEFORE the battlefield block:

```rust
TargetRequirement::TargetCardInYourGraveyard(filter) => {
    let in_your_gy = matches!(obj.zone, ZoneId::Graveyard(pid) if pid == caster);
    if !in_your_gy { return false; }
    matches_filter(&chars, filter) // no hexproof/shroud check — not on battlefield
}
TargetRequirement::TargetCardInGraveyard(filter) => {
    let in_any_gy = matches!(obj.zone, ZoneId::Graveyard(_));
    if !in_any_gy { return false; }
    matches_filter(&chars, filter)
}
```

Key: NO hexproof/shroud/protection checks for graveyard cards — those only apply to
permanents on the battlefield (CR 702.11b, CR 702.18a).

### 5. Resolution — no changes needed

Existing infrastructure handles this:
- Declared targets store ObjectId at cast time
- Resolution re-validates target legality (still in GY? still matches filter?)
- Effect::MoveZone already moves objects between zones
- EffectTarget::DeclaredTarget { index } resolves to the ObjectId

The card defs wire: `targets: vec![TargetCardInYourGraveyard(filter)]` +
`effect: MoveZone { target: DeclaredTarget { index: 0 }, to: Battlefield { tapped: false } }`

### 6. Exhaustive match updates

- `casting.rs`: validate_object_satisfies_requirement — add 2 arms
- `view_model.rs`: if TargetRequirement is displayed anywhere (check)
- `replay_harness.rs`: if target requirements are mapped (check)
- No new StackObjectKind, KeywordAbility, or AbilityDefinition variants needed

### 7. helpers.rs exports

No new types to export — TargetRequirement, TargetFilter already exported.

## Test Plan (8-10 tests)

1. Basic: target creature in your GY, move to BF — verify arrives on BF
2. Filter: target artifact in your GY (Buried Ruin pattern) — verify only artifacts valid
3. Subtype filter: target Vampire or Wizard (Bloodline Necromancer) — verify OR semantics
4. Any-GY: target creature in opponent's GY (Reanimate pattern) — verify cross-player
5. Rejection: target card not matching filter — verify InvalidTarget error
6. Rejection: target card not in GY (on BF) — verify InvalidTarget error
7. Resolution fizzle: target leaves GY before resolution — verify fizzle
8. Move to hand: target artifact in GY, move to hand (Buried Ruin) — verify destination
9. Move to library top: target enchantment in GY, move to library top (Hall pattern)
10. has_subtypes empty vec: verify no filtering (backward compat)

## Card Def Patterns

### ETB trigger with graveyard target (Bloodline Necromancer, Bladewing, Nullpriest)
```rust
abilities: vec![
    AbilityDefinition::Triggered(TriggeredAbilityDef {
        trigger: TriggerCondition::SelfEntersBattlefield,
        targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
            has_card_type: Some(CardType::Creature),
            has_subtypes: vec![SubType::Vampire, SubType::Wizard],
            ..Default::default()
        })],
        effect: Effect::MoveZone {
            target: EffectTarget::DeclaredTarget { index: 0 },
            to: ZoneTarget::Battlefield { tapped: false },
        },
        optional: true, // "you may"
        ..Default::default()
    }),
],
```

### Activated ability with graveyard target (Buried Ruin, Hall)
```rust
AbilityDefinition::Activated {
    cost: Cost::Sequence(vec![Cost::Mana(...), Cost::Tap, Cost::SacrificeThis]),
    targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
        has_card_type: Some(CardType::Artifact),
        ..Default::default()
    })],
    effect: Effect::MoveZone {
        target: EffectTarget::DeclaredTarget { index: 0 },
        to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
    },
    ..Default::default()
}
```

### Spell with any-graveyard target (Reanimate)
```rust
targets: vec![TargetRequirement::TargetCardInGraveyard(TargetFilter {
    has_card_type: Some(CardType::Creature),
    ..Default::default()
})],
effect: Effect::Sequence(vec![
    Effect::MoveZone {
        target: EffectTarget::DeclaredTarget { index: 0 },
        to: ZoneTarget::Battlefield { tapped: false },
    },
    // TODO: "lose life equal to its mana value" — needs EffectAmount::ManaValueOfTarget
    // Defer dynamic life loss to PB-12 or later; implement the targeting + return now
]),
```

Note: Reanimate's "lose life equal to mana value" is a secondary effect. The primary
graveyard targeting + return is the PB-10 primitive. Dynamic life loss based on target
properties is a separate gap (EffectAmount::ManaValueOfTarget) — add a TODO for now.

## Risk Assessment

- **Low risk**: No existing behavior changes. Pure additive (new enum variants + filter field).
- **Backward compat**: `has_subtypes: Vec<SubType>` defaults to empty vec via `#[serde(default)]`.
  Existing `has_subtype` preserved — both can coexist.
- **No StackObjectKind/KeywordAbility changes**: No exhaustive match cascade beyond casting.rs.
