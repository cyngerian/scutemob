// Pyroblast — {R} Instant; modal: counter target spell if it's blue OR destroy target permanent if it's blue.
//
// Per 2016-06-08 ruling: Pyroblast can target ANY spell or permanent; the "if it's blue"
// check happens on resolution, not at cast time. Targets are therefore unrestricted.
// TODO: Resolution-time color check not implemented in DSL — the color filter should be
// applied when the effect resolves, not when the target is declared. Currently the effect
// resolves unconditionally (counters/destroys any target). Fix when Condition::TargetHasColor
// is available and mode effects can apply conditional resolution checks.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("pyroblast"),
        name: "Pyroblast".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Counter target spell if it's blue.\n• Destroy target permanent if it's blue.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            targets: vec![
                // Mode 0: any spell (color check happens on resolution, not at targeting)
                TargetRequirement::TargetSpell,
                // Mode 1: any permanent (color check happens on resolution, not at targeting)
                TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    ..Default::default()
                }),
            ],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Counter target spell if it's blue.
                    Effect::CounterSpell {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        exile_instead: false,
                    },
                    // Mode 1: Destroy target permanent if it's blue.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                    cant_be_regenerated: false,
                    },
                ],
                mode_targets: None,
            }),
            cant_be_countered: false,
        }],
        completeness: Completeness::known_wrong("both modes resolve unconditionally, so this counters any spell / destroys any permanent regardless of color. Oracle gates both on 'if it's blue', checked at resolution (2016-06-08 ruling), and no Condition variant expresses a target's color nor can mode effects apply a conditional resolution check. Needs Condition::TargetHasColor + conditional mode resolution."),
        ..Default::default()
    }
}
