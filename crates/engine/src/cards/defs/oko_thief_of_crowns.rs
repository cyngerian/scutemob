// Oko, Thief of Crowns — {1}{G}{U}, Legendary Planeswalker — Oko, Loyalty 4
// +2: Create a Food token.
// +1: Target artifact or creature loses all abilities and becomes a green Elk creature
//     with base power and toughness 3/3.
// −5: Exchange control of target artifact or creature you control and target creature an
//     opponent controls with power 3 or less.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("oko-thief-of-crowns"),
        name: "Oko, Thief of Crowns".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, blue: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Planeswalker], &["Oko"]),
        oracle_text: "+2: Create a Food token. (It's an artifact with \"{2}, {T}, Sacrifice this token: You gain 3 life.\")\n+1: Target artifact or creature loses all abilities and becomes a green Elk creature with base power and toughness 3/3.\n\u{2212}5: Exchange control of target artifact or creature you control and target creature an opponent controls with power 3 or less.".to_string(),
        starting_loyalty: Some(4),
        abilities: vec![
            // +2: Create a Food token
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(2),
                effect: Effect::CreateToken {
                    spec: food_token_spec(1),
                },
                targets: vec![],
            },
            // CR 613.1d/e/f: +1: Target artifact or creature loses all abilities and becomes
            // a green Elk creature with base power and toughness 3/3.
            // (Approximation: "artifact or creature" → TargetPermanent)
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::RemoveAllAbilities,
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::SetTypeLine {
                                supertypes: im::OrdSet::new(),
                                card_types: [CardType::Creature].into_iter().collect(),
                                subtypes: [SubType("Elk".to_string())].into_iter().collect(),
                            },
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::ColorChange,
                            modification: LayerModification::SetColors(
                                [Color::Green].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtSet,
                            modification: LayerModification::SetPowerToughness {
                                power: 3,
                                toughness: 3,
                            },
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        }),
                    },
                ]),
                targets: vec![TargetRequirement::TargetPermanent],
            },
            // CR 701.12b: -5: Exchange control of target permanent you control and target
            // creature an opponent controls with power 3 or less.
            // (Approximation: two TargetPermanent targets — precise filtering omitted)
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(5),
                effect: Effect::ExchangeControl {
                    target_a: EffectTarget::DeclaredTarget { index: 0 },
                    target_b: EffectTarget::DeclaredTarget { index: 1 },
                    duration: EffectDuration::Indefinite,
                },
                targets: vec![
                    TargetRequirement::TargetPermanent,
                    TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                        max_power: Some(3),
                        ..Default::default()
                    }),
                ],
            },
        ],
        ..Default::default()
    }
}
