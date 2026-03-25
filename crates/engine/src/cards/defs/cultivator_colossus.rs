// Cultivator Colossus — {4}{G}{G}{G}, Creature — Plant Beast */*
// Trample
// Cultivator Colossus's power and toughness are each equal to the number of lands
// you control.
// When this creature enters, you may put a land card from your hand onto the
// battlefield tapped. If you do, draw a card and repeat this process.
//
// TODO: ETB land-play loop (put land, draw, repeat) — too complex for current DSL; deferred.
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
            // CR 604.3, 613.4a: CDA — P/T each equal to the number of lands you control.
            AbilityDefinition::CdaPowerToughness {
                power: EffectAmount::PermanentCount {
                    filter: TargetFilter { has_card_type: Some(CardType::Land), ..Default::default() },
                    controller: PlayerTarget::Controller,
                },
                toughness: EffectAmount::PermanentCount {
                    filter: TargetFilter { has_card_type: Some(CardType::Land), ..Default::default() },
                    controller: PlayerTarget::Controller,
                },
            },
        ],
        ..Default::default()
    }
}
