// Aspect of Hydra — {G}, Instant
// Target creature gets +X/+X until end of turn, where X is your devotion to green.
// (Your devotion to green is the number of green mana symbols in the mana costs of permanents you control.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aspect-of-hydra"),
        name: "Aspect of Hydra".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Target creature gets +X/+X until end of turn, where X is your devotion to \
                      green. (Your devotion to green is the number of green mana symbols in the \
                      mana costs of permanents you control.)"
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 702.5c: devotion to green = number of {G} symbols in mana costs of
            // permanents you control. CR 608.2h: ModifyBothDynamic is substituted to a
            // concrete ModifyBoth(v) at resolution, locking X in.
            effect: Effect::ApplyContinuousEffect {
                effect_def: Box::new(ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBothDynamic {
                        amount: Box::new(EffectAmount::DevotionTo(Color::Green)),
                        negate: false,
                    },
                    filter: EffectFilter::DeclaredTarget { index: 0 },
                    duration: EffectDuration::UntilEndOfTurn,
                    condition: None,
                }),
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
