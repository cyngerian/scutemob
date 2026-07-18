// Simian Spirit Guide — {2}{R}, Creature — Ape Spirit 2/2
// Exile this card from your hand: Add {R}.
//
// PB-EF8: `Cost::ExileSelfFromHand` (CR 118 + CR 400.7 + CR 605.1a) lets this mana
// ability activate from hand, exiling the source card as the cost, producing mana
// stacklessly through `mana_ability_lowering` -> `handle_tap_for_mana`. The exile is
// the ability's exhaustion mechanism — it cannot be activated twice because the card
// leaves the hand (CR 400.7: it becomes a new object, a dead ObjectId afterward).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("simian-spirit-guide"),
        name: "Simian Spirit Guide".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Ape", "Spirit"]),
        oracle_text: "Exile this card from your hand: Add {R}.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::ExileSelfFromHand,
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: ManaPool {
                    red: 1,
                    ..Default::default()
                },
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: Some(ActivationZone::Hand),
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    }
}
