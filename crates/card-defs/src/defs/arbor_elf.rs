// Arbor Elf — {G}, Creature — Elf Druid 1/1
// {T}: Untap target Forest.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("arbor-elf"),
        name: "Arbor Elf".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "{T}: Untap target Forest.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // {T}: Untap target Forest. (An activated ability, not a mana ability —
            // it doesn't add mana itself.)
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_subtype: Some(SubType("Forest".to_string())),
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}
