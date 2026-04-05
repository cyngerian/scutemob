// Elder Deep-Fiend — {8}, Creature — Eldrazi Octopus 5/6; Emerge {5}{U}{U}, Flash,
// cast trigger: tap up to four target permanents.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elder-deep-fiend"),
        name: "Elder Deep-Fiend".to_string(),
        mana_cost: Some(ManaCost { generic: 8, ..Default::default() }),
        types: creature_types(&["Eldrazi", "Octopus"]),
        oracle_text: "Emerge {5}{U}{U} (You may cast this spell by sacrificing a creature and paying the emerge cost reduced by that creature's mana value.)\nFlash\nWhen you cast this spell, tap up to four target permanents.".to_string(),
        power: Some(5),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Emerge),
            AbilityDefinition::Emerge {
                cost: ManaCost { generic: 5, blue: 2, ..Default::default() },
            },
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // When you cast this spell, tap up to four target permanents.
            // TODO: "up to four target permanents" — multi-target tap not in DSL.
            // Using WhenYouCastThisSpell trigger with TapTarget as partial.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenYouCastThisSpell,
                effect: Effect::TapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanent],

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
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
    }
}
