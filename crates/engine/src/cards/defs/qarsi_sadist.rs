// 79. Qarsi Sadist — {1}{B}, Creature — Human Cleric 1/3; Exploit.
// CR 702.110a: When this enters, you may sacrifice a creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("qarsi-sadist"),
        name: "Qarsi Sadist".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types_sub(&[CardType::Creature], &["Human", "Cleric"]),
        oracle_text: "Exploit (When this creature enters the battlefield, you may sacrifice a creature.)".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Exploit),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}
