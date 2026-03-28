// Edric, Spymaster of Trest — {1}{G}{U}, Legendary Creature — Elf Rogue 2/2
// Whenever a creature deals combat damage to one of your opponents, its controller
// may draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("edric-spymaster-of-trest"),
        name: "Edric, Spymaster of Trest".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, blue: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elf", "Rogue"]),
        oracle_text: "Whenever a creature deals combat damage to one of your opponents, its controller may draw a card.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 510.3a / CR 603.2: "Whenever a creature deals combat damage to one of your
            // opponents, its controller may draw a card." — fires when ANY creature (not just
            // Edric himself) deals combat damage to one of Edric's controller's opponents.
            // TODO: approximation — oracle says "its controller" (the creature's controller)
            // draws a card, but PlayerTarget::Controller here resolves to Edric's controller.
            // Correct resolution needs PlayerTarget::ControllerOf(TriggeringCreature), which
            // requires PB-37 DamagedPlayer/TriggeringCreature target support.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAnyCreatureDealsCombatDamageToOpponent,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
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
