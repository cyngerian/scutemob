// Kaito, Dancing Shadow — {2}{U}{B}, Legendary Planeswalker — Kaito
// Whenever one or more creatures you control deal combat damage to a player, you may return
// one of them to its owner's hand. If you do, you may activate loyalty abilities of Kaito
// twice this turn rather than only once.
// +1: Up to one target creature can't attack or block until your next turn.
// 0: Draw a card.
// −2: Create a 2/2 colorless Drone artifact creature token with deathtouch and "When this
//     token leaves the battlefield, each opponent loses 2 life and you gain 2 life."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kaito-dancing-shadow"),
        name: "Kaito, Dancing Shadow".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Kaito"],
        ),
        oracle_text: "Whenever one or more creatures you control deal combat damage to a player, you may return one of them to its owner's hand. If you do, you may activate loyalty abilities of Kaito twice this turn rather than only once.\n+1: Up to one target creature can't attack or block until your next turn.\n0: Draw a card.\n\u{2212}2: Create a 2/2 colorless Drone artifact creature token with deathtouch and \"When this token leaves the battlefield, each opponent loses 2 life and you gain 2 life.\"".to_string(),
        starting_loyalty: Some(3),
        abilities: vec![
            // TODO: Combat damage trigger + bounce + double loyalty activation not expressible.
            // +1: Up to one target creature can't attack or block until your next turn.
            // CR 611.2b: UntilYourNextTurn duration on the can't-block restriction.
            // Note: "can't attack" is a TODO — no CantAttack keyword restriction exists yet.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: crate::state::EffectLayer::Ability,
                        modification: crate::state::LayerModification::AddKeyword(
                            KeywordAbility::CantBlock,
                        ),
                        filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
                        duration: crate::state::EffectDuration::UntilYourNextTurn(
                            crate::state::player::PlayerId(0),
                        ),
                        condition: None,
                    }),
                },
                targets: vec![TargetRequirement::TargetCreature],
            },
            // 0: Draw a card
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Zero,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
            },
            // −2: Create Drone token
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Drone".to_string(),
                        card_types: [CardType::Artifact, CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Drone".to_string())].into_iter().collect(),
                        colors: im::OrdSet::new(),
                        power: 2,
                        toughness: 2,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Deathtouch].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                // TODO: Token LTB trigger (drain 2) not expressible in TokenSpec.
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
