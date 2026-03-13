// Jagged-Scar Archers — {1}{G}{G}, Creature — Elf Archer */*
// Power and toughness each equal number of Elves you control.
// {T}: Deal damage equal to its power to target creature with flying.
// TODO: DSL gap — dynamic P/T based on creature subtype count (Elves you control) is not
// expressible; no CountCreaturesYouControlWithSubtype EffectAmount exists.
// TODO: DSL gap — {T} activated ability dealing damage equal to power to target creature
// with flying requires a TargetFilter for flying creatures and EffectAmount::SelfPower,
// neither of which exists in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("jagged-scar-archers"),
        name: "Jagged-Scar Archers".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: creature_types(&["Elf", "Archer"]),
        oracle_text: "Jagged-Scar Archers's power and toughness are each equal to the number of Elves you control.\n{T}: This creature deals damage equal to its power to target creature with flying.".to_string(),
        power: None,   // */*  CDA — engine SBA skips None toughness
        toughness: None,
        abilities: vec![
            // TODO: static P/T — power and toughness equal the number of Elves you control.
            // DSL gap: no CountCreaturesYouControlWithSubtype EffectAmount.
            // TODO: activated — {T}: deal damage equal to power to target creature with flying.
            // DSL gap: no EffectAmount::SelfPower; no TargetFilter for creatures with flying.
        ],
        ..Default::default()
    }
}
