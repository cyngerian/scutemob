// Lazotep Plating — {1}{U}, Instant
// Amass Zombies 1 + you and permanents you control gain hexproof until end of turn.
//
// TODO: "You and permanents you control gain hexproof until end of turn" — grant
//   hexproof to controller AND all permanents they control (mass hexproof grant)
//   not expressible in DSL (ApplyContinuousEffect only targets one permanent or
//   CreaturesYouControl, not all permanents + controller simultaneously).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lazotep-plating"),
        name: "Lazotep Plating".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Amass Zombies 1. (Put a +1/+1 counter on an Army you control. It's also a Zombie. If you don't control an Army, create a 0/0 black Zombie Army creature token first.)\nYou and permanents you control gain hexproof until end of turn. (You and they can't be the targets of spells or abilities your opponents control.)".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Amass {
                    subtype: "Zombie".to_string(),
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
            // TODO: grant hexproof to controller + all permanents until EOT
        ],
        ..Default::default()
    }
}
