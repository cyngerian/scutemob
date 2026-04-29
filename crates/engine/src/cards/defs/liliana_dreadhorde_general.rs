// Liliana, Dreadhorde General — {4}{B}{B}, Legendary Planeswalker — Liliana
// Whenever a creature you control dies, draw a card.
// +1: Create a 2/2 black Zombie creature token.
// −4: Each player sacrifices two creatures.
// −9: Each opponent chooses a permanent they control of each permanent type and
// sacrifices the rest.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("liliana-dreadhorde-general"),
        name: "Liliana, Dreadhorde General".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Planeswalker], &["Liliana"]),
        oracle_text: "Whenever a creature you control dies, draw a card.\n+1: Create a 2/2 black Zombie creature token.\n\u{2212}4: Each player sacrifices two creatures.\n\u{2212}9: Each opponent chooses a permanent they control of each permanent type and sacrifices the rest.".to_string(),
        starting_loyalty: Some(6),
        abilities: vec![
            // CR 603.10a: "Whenever a creature you control dies, draw a card."
            // PB-23: controller_you filter applied via DeathTriggerFilter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(TargetController::You), exclude_self: false, nontoken_only: false, filter: None,
},
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Zombie".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Zombie".to_string())].into_iter().collect(),
                        colors: [Color::Black].into_iter().collect(),
                        power: 2,
                        toughness: 2,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
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
            // −4: Each player sacrifices two creatures.
            // PB-SFT (CR 701.21a + CR 109.1): creature filter, count = 2.
            // Deterministic fallback: sacrifices the 2 lowest-ObjectId creatures the
            // player controls. If a player controls fewer than 2 creatures, they sacrifice all.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(4),
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::EachPlayer,
                    count: EffectAmount::Fixed(2),
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                },
                targets: vec![],
            },
            // TODO: −9: Each opponent chooses a permanent they control of each permanent type
            // and sacrifices the rest. Requires "choose one of each type" selection rule
            // not yet expressible in DSL.
        ],
        ..Default::default()
    }
}
