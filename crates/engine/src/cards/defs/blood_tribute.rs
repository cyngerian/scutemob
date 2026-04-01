// Blood Tribute — {4}{B}{B}, Sorcery
// Kicker — Tap an untapped Vampire you control.
// Target opponent loses half their life, rounded up. If this spell was kicked,
// you gain life equal to the life lost this way.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blood-tribute"),
        name: "Blood Tribute".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Kicker—Tap an untapped Vampire you control. (You may tap a Vampire you control in addition to any other costs as you cast this spell.)\nTarget opponent loses half their life, rounded up. If this spell was kicked, you gain life equal to the life lost this way.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Kicker),
            AbilityDefinition::Spell {
                // TODO: "loses half life rounded up" needs EffectAmount::HalfLife.
                // TODO: Kicker cost "tap a Vampire" is non-mana kicker.
                // TODO: "if kicked, gain life equal to life lost" needs conditional.
                effect: Effect::Nothing,
                targets: vec![TargetRequirement::TargetPlayer],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
