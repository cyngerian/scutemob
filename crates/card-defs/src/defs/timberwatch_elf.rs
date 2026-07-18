// Timberwatch Elf — {2}{G}, Creature — Elf 1/2
// {T}: Target creature gets +X/+X until end of turn, where X is the number of
// Elves on the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("timberwatch-elf"),
        name: "Timberwatch Elf".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf"]),
        oracle_text: "{T}: Target creature gets +X/+X until end of turn, where X is the number of \
                      Elves on the battlefield."
            .to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            // X = number of Elves on the whole battlefield (any controller), including
            // Timberwatch Elf itself if it's still there when the ability resolves
            // (2016-06-08 ruling). PermanentCount with controller: EachPlayer sums
            // across all players; the filter carries no controller restriction.
            effect: Effect::ApplyContinuousEffect {
                effect_def: Box::new(ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBothDynamic {
                        amount: Box::new(EffectAmount::PermanentCount {
                            filter: TargetFilter {
                                has_subtype: Some(SubType("Elf".to_string())),
                                ..Default::default()
                            },
                            controller: PlayerTarget::EachPlayer,
                        }),
                        negate: false,
                    },
                    filter: EffectFilter::DeclaredTarget { index: 0 },
                    duration: EffectDuration::UntilEndOfTurn,
                    condition: None,
                }),
            },
            timing_restriction: None,
            targets: vec![TargetRequirement::TargetCreature],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    }
}
