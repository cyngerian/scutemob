// Goblin Motivator — {R}, Creature — Goblin Warrior 1/1
// {T}: Target creature gains haste until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-motivator"),
        name: "Goblin Motivator".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "{T}: Target creature gains haste until end of turn. (It can attack and {T} this turn.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
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
        ..Default::default()
    }
}
