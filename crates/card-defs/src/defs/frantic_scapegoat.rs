// Frantic Scapegoat — Haste, ETB suspect itself; can transfer suspect to another creature
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("frantic-scapegoat"),
        name: "Frantic Scapegoat".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Goat"]),
        oracle_text: "Haste\nWhen this creature enters, suspect it. (It has menace and can't block.)\nWhenever one or more other creatures you control enter, if this creature is suspected, you may suspect one of the other creatures. If you do, this creature is no longer suspected.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 603.1: ETB trigger — suspect this creature (CR 701.60a).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Suspect { target: EffectTarget::Source },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "Whenever one or more other creatures you control enter, if this creature
            // is suspected, you may suspect one of the other creatures. If you do, this
            // creature is no longer suspected."
            // Blocked by: no Condition::SourceIsSuspected variant exists, and no Effect
            // variant for "transfer suspect" (unsuspect this + suspect target). Add
            // Condition::SourceIsSuspected and Effect::Unsuspect once DSL supports it.
        ],
        power: Some(1),
        toughness: Some(1),
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
    completeness: Completeness::partial("Blocked on the suspect-transfer trigger: no Condition::SourceIsSuspected for the 'if this creature is suspected' intervening-if (Designations::Suspected exists on GameObject, game_object.rs:21, but no Condition reads it), and 'you may suspect one of the other creatures' is optional with no optional-effect wrapper. Effect::Unsuspect DOES now exist (card_definition.rs:1764) — it is not a blocker."),
    }
}
