// Jagged-Scar Archers — {1}{G}{G}, Creature — Elf Archer */*
// Power and toughness each equal number of Elves you control.
// {T}: Deal damage equal to its power to target creature with flying.
//
// TODO: activated — {T}: deal damage equal to power to target creature with flying.
// DSL gap: no EffectAmount::PowerOf(Source); no TargetFilter for "with flying". Deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("jagged-scar-archers"),
        name: "Jagged-Scar Archers".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: creature_types(&["Elf", "Archer"]),
        oracle_text: "Jagged-Scar Archers's power and toughness are each equal to the number of Elves you control.\n{T}: This creature deals damage equal to its power to target creature with flying.".to_string(),
        power: None,   // */* CDA — P/T set dynamically by Layer 7a
        toughness: None,
        abilities: vec![
            // CR 604.3, 613.4a: CDA — P/T each equal to the number of Elves you control.
            AbilityDefinition::CdaPowerToughness {
                power: EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        has_subtype: Some(SubType("Elf".to_string())),
                        ..Default::default()
                    },
                    controller: PlayerTarget::Controller,
                },
                toughness: EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        has_subtype: Some(SubType("Elf".to_string())),
                        ..Default::default()
                    },
                    controller: PlayerTarget::Controller,
                },
            },
        ],
        ..Default::default()
    }
}
