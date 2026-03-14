// Blinkmoth Nexus — Land, {T}: Add {C}. {1}: animate (TODO). {1},{T}: pump Blinkmoth (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blinkmoth-nexus"),
        name: "Blinkmoth Nexus".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}: This land becomes a 1/1 Blinkmoth artifact creature with flying until end of turn. It's still a land.\n{1}, {T}: Target Blinkmoth creature gets +1/+1 until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {1}: Animate land as 1/1 Blinkmoth artifact creature with flying — land animation not in DSL
            // {1}, {T}: Target Blinkmoth creature gets +1/+1 until end of turn.
            // TODO: TargetFilter lacks subtype-only creature filter — using TargetCreature (over-permissive).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: crate::state::EffectLayer::PtModify,
                        modification: crate::state::LayerModification::ModifyBoth(1),
                        filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
                        duration: crate::state::EffectDuration::UntilEndOfTurn,
                    }),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
            },
        ],
        ..Default::default()
    }
}
