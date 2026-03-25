// Wrenn and Seven — {3}{G}{G}, Legendary Planeswalker — Wrenn
// +1: Reveal the top four cards of your library. Put all land cards from among them
//     into your hand and the rest into your graveyard.
// 0: Put any number of land cards from your hand onto the battlefield tapped.
// −3: Create a green Treefolk creature token with reach and "This creature's power and
//     toughness are each equal to the number of lands you control."
// −8: Return all permanent cards from your graveyard to your hand.
//     You get an emblem with "You have no maximum hand size."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wrenn-and-seven"),
        name: "Wrenn and Seven".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Wrenn"],
        ),
        oracle_text: "+1: Reveal the top four cards of your library. Put all land cards from among them into your hand and the rest into your graveyard.\n0: Put any number of land cards from your hand onto the battlefield tapped.\n\u{2212}3: Create a green Treefolk creature token with reach and \"This creature's power and toughness are each equal to the number of lands you control.\"\n\u{2212}8: Return all permanent cards from your graveyard to your hand. You get an emblem with \"You have no maximum hand size.\"".to_string(),
        abilities: vec![
            // +1: Reveal top 4, put lands in hand, rest to graveyard.
            // TODO: "Reveal top N and sort into different zones by card type" is a complex
            // effect. Known DSL gap. Partial: Scry 4 as approximation.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Scry {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(4),
                },
                targets: vec![],
            },
            // 0: Put any number of land cards from hand onto battlefield tapped.
            // TODO: Interactive multi-card "put any number" from hand is a DSL gap.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Zero,
                effect: Effect::Sequence(vec![]),
                targets: vec![],
            },
            // −3: Create a green Treefolk token with reach and a CDA (P/T = lands you control).
            // The CDA (*/*) requires power: None, toughness: None on the token spec.
            // TODO: CDA "P/T equal to number of lands you control" — this requires a
            // CDA continuous effect on the token. The token is created without the CDA
            // for now; the CDA is a known DSL gap for tokens.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(3),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Treefolk".to_string(),
                        power: 0,
                        toughness: 0,
                        colors: [Color::Green].into_iter().collect(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Treefolk".to_string())].into_iter().collect(),
                        keywords: [KeywordAbility::Reach].into_iter().collect(),
                        count: 1,
                        ..Default::default()
                    },
                },
                targets: vec![],
            },
            // −8: Return all permanent cards from your graveyard to your hand.
            //     You get an emblem with "You have no maximum hand size." (CR 114.1-114.4)
            // NOTE: "No maximum hand size" from an emblem affects the player during cleanup.
            // The engine checks NoMaxHandSize as a keyword on permanents, not on players.
            // This emblem creates the infrastructure correctly; the cleanup-step enforcement
            // requires scanning command zone emblems for NoMaxHandSize, which is a known
            // LOW gap. The emblem itself is created.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(8),
                // TODO: "Return all permanent cards from graveyard to hand" — ForEach with
                // graveyard filter. Partial: just create the emblem.
                effect: Effect::CreateEmblem {
                    triggered_abilities: vec![],
                    static_effects: vec![
                        ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            // TODO: INCORRECT APPROXIMATION — "You have no maximum hand size"
                            // is a player-level rule modification (CR 402.3), not a keyword
                            // granted to permanents. The cleanup step should check whether the
                            // active player has a no_max_hand_size flag and skip discard if so.
                            // This grants NoMaxHandSize to permanents you control as a proxy,
                            // which fails when the player controls no permanents (the emblem
                            // would have no effect). Correct fix: add player.no_max_hand_size
                            // flag to PlayerState and check it in the cleanup step discard loop.
                            // Known DSL gap — MEDIUM severity per PB-22 S6 review.
                            modification: LayerModification::AddKeyword(KeywordAbility::NoMaxHandSize),
                            filter: EffectFilter::CreaturesYouControl,
                            duration: EffectDuration::Indefinite,
                            condition: None,
                        },
                    ],
                },
                targets: vec![],
            },
        ],
        starting_loyalty: Some(5),
        adventure_face: None,
        meld_pair: None,
        activated_ability_cost_reductions: vec![],
        ..Default::default()
    }
}
