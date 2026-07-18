// Turn // Burn — Split card with Fuse (Dragon's Maze)
// Turn: {2}{U} Instant — Until end of turn, target creature loses all abilities and
//       becomes a red Weird with base power and toughness 0/1.
// Burn: {1}{R} Instant — Burn deals 2 damage to any target.
// Fuse (You may cast one or both halves of this card from your hand.)
//
// CR 702.102: Fuse — both halves may be cast together from hand, paying combined cost.
// CR 702.102d: Left half (Turn) resolves before right half (Burn); the right half's
//   target is index 1, following the left half's target(s).
// Ruling (2013-04-15): "Turn will cause the creature lose all other colors and creature
//   types, but it will retain any other card types (such as artifact) it may have" —
//   so no SetCardTypes here, only SetCreatureTypes (Layer 4) + SetColors (Layer 5).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("turn"),
        name: "Turn // Burn".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Turn — Until end of turn, target creature loses all abilities and becomes a \
                      red Weird with base power and toughness 0/1.\nBurn — Burn deals 2 damage to \
                      any target.\nFuse (You may cast one or both halves of this card from your \
                      hand.)"
            .to_string(),
        abilities: vec![
            // Fuse keyword marker (CR 702.102)
            AbilityDefinition::Keyword(KeywordAbility::Fuse),
            // Turn (left half): until end of turn, target creature loses all abilities
            // and becomes a red Weird with base P/T 0/1. Four layer-tagged continuous
            // effects; the engine sorts by layer at apply time, so Sequence order here
            // is documentation only, not semantics.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // Layer 4 (CR 205.1a): creature-type subtypes become exactly {Weird}.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::SetCreatureTypes(
                                [SubType("Weird".to_string())].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    // Layer 5 (CR 613.1e): colors become exactly {Red}.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::ColorChange,
                            modification: LayerModification::SetColors(
                                [Color::Red].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    // Layer 6 (CR 613.1f): loses all abilities.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::RemoveAllAbilities,
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    // Layer 7b (CR 613.4b): base power and toughness become 0/1.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtSet,
                            modification: LayerModification::SetPowerToughness {
                                power: 0,
                                toughness: 1,
                            },
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
            // Burn (right half): {1}{R}. Deals 2 damage to any target. No lifegain
            // clause on this card — only "Burn deals 2 damage to any target."
            // Target index 1 (right-half target follows left-half targets, CR 702.102d).
            AbilityDefinition::Fuse {
                name: "Burn".to_string(),
                cost: ManaCost {
                    generic: 1,
                    red: 1,
                    ..Default::default()
                },
                card_type: CardType::Instant,
                effect: Effect::DealDamage {
                    source: None,
                    target: EffectTarget::DeclaredTarget { index: 1 },
                    amount: EffectAmount::Fixed(2),
                },
                targets: vec![TargetRequirement::TargetAny],
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
