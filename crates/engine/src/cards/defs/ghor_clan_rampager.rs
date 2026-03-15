// Ghor-Clan Rampager — {2}{R}{G}, Creature — Beast 4/4; Trample; Bloodrush {R}{G}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ghor-clan-rampager"),
        name: "Ghor-Clan Rampager".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Beast"]),
        oracle_text: "Trample\nBloodrush — {R}{G}, Discard this card: Target attacking creature gets +4/+4 and gains trample until end of turn.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Bloodrush {
                cost: ManaCost { red: 1, green: 1, ..Default::default() },
                power_boost: 4,
                toughness_boost: 4,
                grants_keyword: Some(KeywordAbility::Trample),
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
    }
}
