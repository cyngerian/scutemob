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
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Suspect { target: EffectTarget::Source },
                intervening_if: None,
                targets: vec![],
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
    }
}
