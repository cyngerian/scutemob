// Shriekmaw — {4}{B}, Creature — Elemental 3/2
// Fear
// When this creature enters, destroy target nonartifact, nonblack creature.
// Evoke {1}{B}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shriekmaw"),
        name: "Shriekmaw".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, ..Default::default() }),
        types: creature_types(&["Elemental"]),
        oracle_text: "Fear (This creature can't be blocked except by artifact creatures and/or black creatures.)\nWhen this creature enters, destroy target nonartifact, nonblack creature.\nEvoke {1}{B} (You may cast this spell for its evoke cost. If you do, it's sacrificed when it enters.)".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Fear),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                // "nonartifact, nonblack creature" — exclude_colors handles nonblack;
                // TODO: no non_artifact filter on TargetFilter — this targets any nonblack
                // creature including artifact creatures. When exclude_card_types is added,
                // exclude Artifact.
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    exclude_colors: Some([Color::Black].into_iter().collect()),
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Evoke {
                cost: ManaCost { generic: 1, black: 1, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
