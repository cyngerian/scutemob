// Skewer the Critics — {2R} Sorcery; Spectacle {R}; deals 3 damage to any target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skewer-the-critics"),
        name: "Skewer the Critics".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Spectacle {R} (You may cast this spell for its spectacle cost rather than its mana cost if an opponent lost life this turn.)\nSkewer the Critics deals 3 damage to any target.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Spectacle),
            AbilityDefinition::Spectacle {
                cost: ManaCost { red: 1, ..Default::default() },
            },
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(3),
                },
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
