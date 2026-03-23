// Return of the Wildspeaker — {4}{G}, Instant
// Choose one —
// • Draw cards equal to the greatest power among non-Human creatures you control.
// • Non-Human creatures you control get +3/+3 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("return-of-the-wildspeaker"),
        name: "Return of the Wildspeaker".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Draw cards equal to the greatest power among non-Human creatures you control.\n• Non-Human creatures you control get +3/+3 until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![]),
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        // Mode 0: Draw cards equal to greatest power among non-Human creatures you control.
                        // TODO: DSL gap — EffectAmount::GreatestPowerAmong(filter) does not exist.
                        Effect::Nothing,
                        // Mode 1: Non-Human creatures you control get +3/+3 until end of turn.
                        // TODO: DSL gap — EffectFilter for "non-Human creatures you control" does not exist.
                        // CreaturesYouControl exists but no exclusion by subtype.
                        Effect::Nothing,
                    ],
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
