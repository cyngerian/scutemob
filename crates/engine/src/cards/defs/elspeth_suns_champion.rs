// Elspeth, Sun's Champion — {4}{W}{W} Legendary Planeswalker — Elspeth
// +1: Create three 1/1 white Soldier creature tokens.
// −3: Destroy all creatures with power 4 or greater.
// −7: You get an emblem with "Creatures you control get +2/+2 and have flying."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elspeth-suns-champion"),
        name: "Elspeth, Sun's Champion".to_string(),
        mana_cost: Some(ManaCost { generic: 4, white: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Planeswalker], &["Elspeth"]),
        oracle_text: "+1: Create three 1/1 white Soldier creature tokens.\n\u{2212}3: Destroy all creatures with power 4 or greater.\n\u{2212}7: You get an emblem with \"Creatures you control get +2/+2 and have flying.\"".to_string(),
        starting_loyalty: Some(4),
        abilities: vec![
            // +1: Create three 1/1 white Soldier creature tokens.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Soldier".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Soldier".to_string())].into_iter().collect(),
                        colors: [Color::White].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        count: 3,
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                targets: vec![],
            },
            // −3: Destroy all creatures with power 4 or greater.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(3),
                effect: Effect::DestroyAll {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        min_power: Some(4),
                        ..Default::default()
                    },
                    cant_be_regenerated: false,
                },
                targets: vec![],
            },
            // −7: You get an emblem with "Creatures you control get +2/+2 and have flying."
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(7),
                effect: Effect::CreateEmblem {
                    triggered_abilities: vec![],
                    static_effects: vec![
                        // +2/+2 to creatures you control
                        ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyBoth(2),
                            filter: EffectFilter::CreaturesYouControl,
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        },
                        // Grant flying to creatures you control
                        ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Flying),
                            filter: EffectFilter::CreaturesYouControl,
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        },
                    ],
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
