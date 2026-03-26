// 108. Glistener Elf — {G}, Creature — Elf Warrior 1/1; Infect.
// CR 702.90: Infect — damage to creatures as -1/-1 counters, to players as poison counters.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("glistener-elf"),
        name: "Glistener Elf".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Warrior"]),
        oracle_text: "Infect (This creature deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Infect),
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
