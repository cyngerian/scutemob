// Delighted Halfling — {G}, Creature — Halfling Citizen 1/2.
// "{T}: Add {G}. If this mana is spent to cast a legendary spell, that spell
// can't be countered."
// TODO: DSL gap — "mana tracking" (conditional uncounterability based on mana
// source) is not expressible. Modeled as a plain {G} mana dork.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("delighted-halfling"),
        name: "Delighted Halfling".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Halfling", "Citizen"]),
        oracle_text: "{T}: Add {G}. If this mana is spent to cast a legendary spell, that spell can't be countered.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: mana_pool(0, 0, 0, 0, 1, 0),
            },
            timing_restriction: None,
            targets: vec![],
                activation_condition: None,
        }],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        activated_ability_cost_reductions: vec![],
    }
}
