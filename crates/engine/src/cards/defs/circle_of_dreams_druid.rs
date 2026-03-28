// Circle of Dreams Druid — {G}{G}{G}, Creature — Elf Druid 2/1
// {T}: Add {G} for each creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("circle-of-dreams-druid"),
        name: "Circle of Dreams Druid".to_string(),
        mana_cost: Some(ManaCost { green: 3, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "{T}: Add {G} for each creature you control.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaScaled {
                    player: PlayerTarget::Controller,
                    color: ManaColor::Green,
                    count: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
