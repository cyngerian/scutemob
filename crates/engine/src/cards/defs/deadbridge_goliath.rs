// Deadbridge Goliath — {2}{G}{G}, Creature — Insect 5/5; Scavenge {4}{G}{G}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("deadbridge-goliath"),
        name: "Deadbridge Goliath".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: creature_types(&["Insect"]),
        oracle_text: "Scavenge {4}{G}{G} ({4}{G}{G}, Exile this card from your graveyard: Put a number of +1/+1 counters equal to this card's power on target creature. Scavenge only as a sorcery.)".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Scavenge),
            AbilityDefinition::Scavenge {
                cost: ManaCost { generic: 4, green: 2, ..Default::default() },
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
    }
}
