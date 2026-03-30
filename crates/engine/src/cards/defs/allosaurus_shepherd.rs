// Allosaurus Shepherd — {G}, Creature — Elf Shaman 1/1
// This spell can't be countered.
// Green spells you control can't be countered.
// {4}{G}{G}: Until end of turn, each Elf creature you control has base power and
// toughness 5/5 and becomes a Dinosaur in addition to its other creature types.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("allosaurus-shepherd"),
        name: "Allosaurus Shepherd".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Shaman"]),
        oracle_text: "This spell can't be countered.\nGreen spells you control can't be countered.\n{4}{G}{G}: Until end of turn, each Elf creature you control has base power and toughness 5/5 and becomes a Dinosaur in addition to its other creature types.".to_string(),
        power: Some(1),
        toughness: Some(1),
        cant_be_countered: true,
        abilities: vec![
            // TODO: "Green spells you control can't be countered" — static anti-counter
            // effect not in DSL.
            // TODO: Activated overwrite: base P/T 5/5 + add Dinosaur type to Elves.
            // Needs Layer 7b SetBasePowerToughness + Layer 4 AddSubtype on
            // CreaturesYouControlWithSubtype(Elf), both UntilEndOfTurn.
        ],
        ..Default::default()
    }
}
