// Elvish Spirit Guide — {2}{G}, Creature — Elf Spirit 2/2
// Exile this creature from your hand: Add {G}.
//
// PB-EF8: `Cost::ExileSelfFromHand` (CR 118 + CR 400.7 + CR 605.1a) lets this mana
// ability activate from hand, exiling the source card as the cost, producing mana
// stacklessly through `mana_ability_lowering` -> `handle_tap_for_mana`. Prior def
// shipped a FREE, repeatable, battlefield-activated "Add {G}" (`Cost::Mana(default)`,
// `activation_zone: None`) = unbounded infinite mana; replaced here with a faithful
// one-shot, from-hand, stackless mana ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elvish-spirit-guide"),
        name: "Elvish Spirit Guide".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Spirit"]),
        oracle_text: "Exile this card from your hand: Add {G}.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::ExileSelfFromHand,
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: ManaPool {
                    green: 1,
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
