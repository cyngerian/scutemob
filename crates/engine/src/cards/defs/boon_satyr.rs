// 59. Boon Satyr — {2G}, Creature — Satyr 4/2; Flash; Bestow {4GG}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boon-satyr"),
        name: "Boon Satyr".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Satyr"]),
        oracle_text: "Flash\nBestow {4}{G}{G} (If you cast this card for its bestow cost, it's an Aura spell with enchant creature. It becomes a creature again if it's not attached to a creature.)".to_string(),
        power: Some(4),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Keyword(KeywordAbility::Bestow),
            AbilityDefinition::Bestow { cost: ManaCost { generic: 4, green: 2, ..Default::default() } },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}
