// Elderfang Ritualist — {2}{B}, Creature — Elf Cleric 3/1
// When this creature dies, return another target Elf card from your graveyard to your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elderfang-ritualist"),
        name: "Elderfang Ritualist".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Cleric"]),
        oracle_text: "When Elderfang Ritualist dies, return another target Elf card from your graveyard to your hand.".to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_subtype: Some(SubType("Elf".to_string())),
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
