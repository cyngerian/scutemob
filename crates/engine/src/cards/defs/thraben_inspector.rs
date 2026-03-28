// 105. Thraben Inspector — {W}, Creature — Human Soldier 1/2;
// When Thraben Inspector enters the battlefield, investigate.
// (Create a Clue token. CR 701.16a / CR 111.10f)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thraben-inspector"),
        name: "Thraben Inspector".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Soldier"]),
        oracle_text: "When Thraben Inspector enters the battlefield, investigate. (Create a Clue token. It's an artifact with \"{2}, Sacrifice this token: Draw a card.\")".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Investigate { count: EffectAmount::Fixed(1) },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
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
    }
}
