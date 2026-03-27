// Promise of Power — {2}{B}{B}{B}, Sorcery, modal (draw 5 + lose 5 / create Demon token), Entwine {4}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("promise-of-power"),
        name: "Promise of Power".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 3, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose one —\n\
            • You draw five cards and you lose 5 life.\n\
            • Create an X/X black Demon creature token with flying, where X is the number of cards in your hand.\n\
            Entwine {4} (Choose both if you pay the entwine cost.)"
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Entwine),
            AbilityDefinition::Entwine { cost: ManaCost { generic: 4, ..Default::default() } },
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![]),
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        // Mode 0: Draw 5 cards, then lose 5 life
                        Effect::Sequence(vec![
                            Effect::DrawCards {
                                player: PlayerTarget::Controller,
                                count: EffectAmount::Fixed(5),
                            },
                            Effect::LoseLife {
                                player: PlayerTarget::Controller,
                                amount: EffectAmount::Fixed(5),
                            },
                        ]),
                        // Mode 1: Create an X/X black Demon token with flying.
                        // TODO: TokenSpec uses fixed i32 for power/toughness; dynamic X/X (cards in hand)
                        // requires a new TokenSpec variant (e.g., DynamicPowerToughness). Using 5/5 as
                        // a placeholder. True behavior: X = controller's hand size at resolution time.
                        Effect::CreateToken {
                            spec: TokenSpec {
                                name: "Demon".to_string(),
                                power: 5,
                                toughness: 5,
                                colors: [Color::Black].into_iter().collect(),
                                supertypes: OrdSet::new(),
                                card_types: [CardType::Creature].into_iter().collect(),
                                subtypes: [SubType("Demon".to_string())].into_iter().collect(),
                                keywords: [KeywordAbility::Flying].into_iter().collect(),
                                count: 1,
                                tapped: false,
                                enters_attacking: false,
                                mana_color: None,
                                mana_abilities: vec![],
                                activated_abilities: vec![],
                                ..Default::default()
                            },
                        },
                    ],
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
