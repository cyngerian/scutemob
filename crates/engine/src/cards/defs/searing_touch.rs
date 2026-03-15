// Searing Touch — {R}, Instant; Buyback {4}; deals 1 damage to any target.
//
// CR 702.27a: Buyback — you may pay an additional {4} as you cast this spell.
// If you do, put this card into your hand as it resolves instead of the graveyard.
//
// Buyback {4} is encoded as AbilityDefinition::Buyback { cost } below.
// KeywordAbility::Buyback exists for presence-checking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("searing-touch"),
        name: "Searing Touch".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Buyback {4} (You may pay an additional {4} as you cast this spell. If you do, put this card into your hand as it resolves.)\nSearing Touch deals 1 damage to any target.".to_string(),
        abilities: vec![
            // CR 702.27a: Buyback cost ({4}).
            AbilityDefinition::Buyback {
                cost: ManaCost { generic: 4, ..Default::default() },
            },
            // Spell effect: deal 1 damage to any target.
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                },
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
