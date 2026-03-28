// Armorcraft Judge — {3}{G}, Creature — Elf Artificer 3/3
// When this creature enters, draw a card for each creature you control with a
// +1/+1 counter on it.
//
// TODO: "Creatures with +1/+1 counters" count — PermanentCount lacks counter filter.
//   Using all creatures as approximation.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("armorcraft-judge"),
        name: "Armorcraft Judge".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Artificer"]),
        oracle_text: "When this creature enters, draw a card for each creature you control with a +1/+1 counter on it.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            controller: TargetController::You,
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
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
