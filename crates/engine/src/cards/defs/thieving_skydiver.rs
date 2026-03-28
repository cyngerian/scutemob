// Thieving Skydiver — {1}{U}, Creature — Merfolk Rogue 2/1
// Kicker {X} (X can't be 0); Flying
// ETB (if kicked): gain control of target artifact (mana value X or less filter approximated)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thieving-skydiver"),
        name: "Thieving Skydiver".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Merfolk", "Rogue"]),
        oracle_text: "Kicker {X}. X can't be 0. (You may pay an additional {X} as you cast this spell.)\nFlying\nWhen this creature enters, if it was kicked, gain control of target artifact with mana value X or less. If that artifact is an Equipment, attach it to this creature.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Kicker),
            // CR 613.1b: ETB (if kicked): gain control of target artifact (indefinitely).
            // Approximation: "with mana value X or less" filter and Equipment attach omitted
            // (mana-value variable filter and Equipment-attach not in DSL).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::GainControl {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    duration: EffectDuration::Indefinite,
                },
                intervening_if: Some(Condition::WasKicked),
                targets: vec![TargetRequirement::TargetArtifact],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
