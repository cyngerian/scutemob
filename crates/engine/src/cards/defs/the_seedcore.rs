// The Seedcore — Land — Sphere
// {T}: Add {C}.
// {T}: Add one mana of any color. Spend this mana only to cast Phyrexian creature spells.
// Corrupted — {T}: Target 1/1 creature gets +2/+1 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-seedcore"),
        name: "The Seedcore".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Sphere"]),
        oracle_text: "{T}: Add {C}.\n{T}: Add one mana of any color. Spend this mana only to cast Phyrexian creature spells.\nCorrupted — {T}: Target 1/1 creature gets +2/+1 until end of turn. Activate only if an opponent has three or more poison counters.".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // {T}: Add one mana of any color. Spend this mana only to cast Phyrexian creature spells.
            // Note: "Phyrexian" is a creature type in MTG. Using CreatureWithSubtype to enforce
            // both the creature-spell and Phyrexian-subtype requirements per oracle text.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColorRestricted {
                    player: PlayerTarget::Controller,
                    restriction: ManaRestriction::CreatureWithSubtype(SubType(
                        "Phyrexian".to_string(),
                    )),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // Corrupted — {T}: Target 1/1 creature gets +2/+1 until end of turn.
            // Activate only if an opponent has three or more poison counters.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: crate::state::EffectLayer::PtModify,
                            modification: crate::state::LayerModification::ModifyPower(2),
                            filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
                            duration: crate::state::EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: crate::state::EffectLayer::PtModify,
                            modification: crate::state::LayerModification::ModifyToughness(1),
                            filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
                            duration: crate::state::EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                timing_restriction: None,
                // TODO: Target should be "1/1 creature" — TargetFilter lacks exact P/T constraint.
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: Some(Condition::OpponentHasPoisonCounters(3)),
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
