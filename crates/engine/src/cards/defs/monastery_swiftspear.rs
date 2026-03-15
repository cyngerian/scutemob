// 55. Monastery Swiftspear — {R}, Creature — Human Monk 1/2; Haste. Prowess.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("monastery-swiftspear"),
        name: "Monastery Swiftspear".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Human", "Monk"]),
        oracle_text: "Haste\nProwess (Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.)".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 702.108a: Prowess — builder.rs auto-generates the triggered ability from this keyword.
            AbilityDefinition::Keyword(KeywordAbility::Prowess),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}
