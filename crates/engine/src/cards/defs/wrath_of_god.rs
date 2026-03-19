// 26. Wrath of God — {2WW}, Sorcery; destroy all creatures. They can't be
// regenerated.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wrath-of-god"),
        name: "Wrath of God".to_string(),
        mana_cost: Some(ManaCost { white: 2, generic: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Destroy all creatures. They can't be regenerated.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.8: Destroy all creatures. CR 701.19c: can't be regenerated.
            effect: Effect::DestroyAll {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                },
                cant_be_regenerated: true,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
