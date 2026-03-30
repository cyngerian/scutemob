// Dreadhound — {4}{B}{B}, Creature — Demon Dog 6/6
// When this creature enters, mill three cards.
// Whenever a creature dies or a creature card is put into a graveyard from a library,
// each opponent loses 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dreadhound"),
        name: "Dreadhound".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 2, ..Default::default() }),
        types: creature_types(&["Demon", "Dog"]),
        oracle_text: "When Dreadhound enters, mill three cards.\nWhenever a creature dies or a creature card is put into a graveyard from a library, each opponent loses 1 life.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            // CR 603.1: "When Dreadhound enters, mill three cards."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MillCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // CR 603.2: "Whenever a creature dies or a creature card is put into a graveyard
            // from a library, each opponent loses 1 life."
            // Partial: only creature deaths fire (WheneverCreatureDies). "Creature card put
            // into GY from library" (mill trigger) is a known DSL gap.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: None,
                    exclude_self: false,
                    nontoken_only: false,
                },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::LoseLife {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(1),
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
