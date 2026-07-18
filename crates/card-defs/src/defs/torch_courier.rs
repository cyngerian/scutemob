// Torch Courier — {R}, Creature — Goblin 1/1
// Haste; Sacrifice: another target creature gains haste until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("torch-courier"),
        name: "Torch Courier".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Haste\nSacrifice this creature: Another target creature gains haste until \
                      end of turn."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // Sacrifice this creature: Another target creature gains haste until end of turn.
            // PB-XS: CR 109.1 / 601.2c — "another target creature" excludes Torch Courier.
            // CR 601.2b/c: target chosen BEFORE costs paid, so the self-sacrifice cost does
            // not retroactively make self-targeting legal — exclude_self enforces this at
            // announcement time.
            AbilityDefinition::Activated {
                cost: Cost::SacrificeSelf,
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
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    exclude_self: true,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}
