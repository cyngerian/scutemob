// Satyr Wayfinder — {1}{G}, Creature — Satyr 1/1
// When this creature enters, reveal the top four cards of your library. You may
// put a land card from among them into your hand. Put the rest into your graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("satyr-wayfinder"),
        name: "Satyr Wayfinder".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Satyr"]),
        oracle_text: "When this creature enters, reveal the top four cards of your library. You \
                      may put a land card from among them into your hand. Put the rest into your \
                      graveyard."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            // `Effect::RevealAndRoute` routes ALL matching cards (mandatory) — printed text is
            // "You may put A land card" (at most one, optional). Use the put-≤1 sibling instead
            // (see its doc in card-types/src/cards/card_definition.rs).
            // Note: neither primitive emits a distinct "reveal" event (only zone-move events),
            // so modelling this as "look at" rather than "reveal" loses no simulated behaviour.
            effect: Effect::LookAtTopThenPlace {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(4),
                filter: TargetFilter {
                    has_card_type: Some(CardType::Land),
                    ..Default::default()
                },
                place_cost: None,
                destination: ZoneTarget::Hand {
                    owner: PlayerTarget::Controller,
                },
                rest_to: ZoneTarget::Graveyard {
                    owner: PlayerTarget::Controller,
                },
                optional: true,
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}
