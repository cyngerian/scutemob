// Voice of Many — {2}{G}{G}, Creature — Elf Druid 3/3
// When this creature enters, draw a card for each opponent who controls fewer
// creatures than you.
//
// TODO: "Each opponent with fewer creatures" count not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("voice-of-many"),
        name: "Voice of Many".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "When this creature enters, draw a card for each opponent who controls fewer creatures than you.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: Opponent comparison count not expressible. Using Fixed(1).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
