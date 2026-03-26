// Forerunner of Slaughter — {B}{R}, Creature — Eldrazi Drone 3/2; Devoid.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("forerunner-of-slaughter"),
        name: "Forerunner of Slaughter".to_string(),
        mana_cost: Some(ManaCost { black: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Eldrazi", "Drone"]),
        oracle_text: "Devoid (This card has no color.)\n{1}: Target colorless creature gains haste until end of turn.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Devoid),
            // {1}: Target colorless creature gains haste until end of turn.
            // TODO: TargetFilter lacks is_colorless — using TargetCreature (over-permissive).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: crate::state::EffectLayer::Ability,
                        modification: crate::state::LayerModification::AddKeyword(
                            KeywordAbility::Haste,
                        ),
                        filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
                        duration: crate::state::EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
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
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}
