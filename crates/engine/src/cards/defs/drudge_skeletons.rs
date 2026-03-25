// 107. Drudge Skeletons — {1}{B}, Creature — Skeleton 1/1; {B}: Regenerate ~.
// CR 701.19a: Regenerate — next time destroyed, remove damage, tap, remove from combat.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("drudge-skeletons"),
        name: "Drudge Skeletons".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Skeleton"]),
        oracle_text: "{B}: Regenerate Drudge Skeletons.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { black: 1, ..Default::default() }),
                effect: Effect::Regenerate { target: EffectTarget::Source },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
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
