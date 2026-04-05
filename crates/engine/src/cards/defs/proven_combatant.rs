// Proven Combatant — {U}, Creature — Human Warrior 1/1; Eternalize {4}{U}{U}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("proven-combatant"),
        name: "Proven Combatant".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Warrior"]),
        oracle_text: "Eternalize {4}{U}{U} ({4}{U}{U}, Exile this card from your graveyard: Create a token that's a copy of it, except it's a 4/4 black Zombie Human Warrior with no mana cost. Eternalize only as a sorcery.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Eternalize),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Eternalize,
                cost: ManaCost { generic: 4, blue: 2, ..Default::default() },
                details: None,
            },
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
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
    }
}
