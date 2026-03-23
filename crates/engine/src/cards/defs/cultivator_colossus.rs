// Cultivator Colossus — {4}{G}{G}{G}, Creature — Plant Beast */*
// Trample
// Cultivator Colossus's power and toughness are each equal to the number of lands
// you control.
// When this creature enters, you may put a land card from your hand onto the
// battlefield tapped. If you do, draw a card and repeat this process.
//
// TODO: CDA P/T = land count. ETB loop (put land, draw, repeat) too complex.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cultivator-colossus"),
        name: "Cultivator Colossus".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 3, ..Default::default() }),
        types: creature_types(&["Plant", "Beast"]),
        oracle_text: "Trample\nCultivator Colossus's power and toughness are each equal to the number of lands you control.\nWhen this creature enters, you may put a land card from your hand onto the battlefield tapped. If you do, draw a card and repeat this process.".to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // TODO: CDA + ETB land-play loop not expressible.
        ],
        ..Default::default()
    }
}
