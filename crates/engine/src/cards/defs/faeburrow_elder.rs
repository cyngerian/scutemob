// Faeburrow Elder — {1}{G}{W}, Creature — Treefolk Druid 0/0
// Vigilance
// This creature gets +1/+1 for each color among permanents you control.
// {T}: For each color among permanents you control, add one mana of that color.
//
// Vigilance is implemented.
//
// TODO: DSL gap — "gets +1/+1 for each color among permanents you control" is a CDA
// (characteristic-defining ability) whose value depends on counting distinct colors
// among permanents you control. No EffectAmount variant exists for this calculation.
// The P/T CDA is omitted; base P/T is 0/0 as printed.
//
// TODO: DSL gap — "{T}: for each color among permanents you control, add one mana of
// that color" requires a mana ability that dynamically generates mana of multiple colors
// based on a runtime query. AddManaAnyColor does not model this correctly (produces one
// mana of any single color). The mana activated ability is omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("faeburrow-elder"),
        name: "Faeburrow Elder".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Treefolk", "Druid"]),
        oracle_text: "Vigilance\nThis creature gets +1/+1 for each color among permanents you control.\n{T}: For each color among permanents you control, add one mana of that color.".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
        ],
        ..Default::default()
    }
}
