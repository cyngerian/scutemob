// Territorial Maro — {4}{G}, Creature — Elemental */*
// Domain — Territorial Maro's power and toughness are each equal to twice the number of
// basic land types among lands you control.
//
// CR 604.3: CDAs function in all zones.
// CR 613.4a: CDA sets P/T in Layer 7a.
// CR 305.6 / ability word "Domain": domain count = distinct basic land types among lands you
// control. P/T = 2 * domain count, computed as DomainCount + DomainCount via Sum.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("territorial-maro"),
        name: "Territorial Maro".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 1, ..Default::default() }),
        types: creature_types(&["Elemental"]),
        oracle_text: "Domain \u{2014} Territorial Maro's power and toughness are each equal to twice the number of basic land types among lands you control.".to_string(),
        power: None,   // */* CDA — P/T set dynamically by Layer 7a
        toughness: None,
        abilities: vec![
            // CR 604.3, 613.4a: CDA — P/T = 2 * domain count.
            // 2 * domain_count is expressed as DomainCount + DomainCount using Sum.
            AbilityDefinition::CdaPowerToughness {
                power: EffectAmount::Sum(
                    Box::new(EffectAmount::DomainCount),
                    Box::new(EffectAmount::DomainCount),
                ),
                toughness: EffectAmount::Sum(
                    Box::new(EffectAmount::DomainCount),
                    Box::new(EffectAmount::DomainCount),
                ),
            },
        ],
        ..Default::default()
    }
}
