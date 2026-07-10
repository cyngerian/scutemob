// 57. Fiery Temper — {1RR}, Instant; Fiery Temper deals 3 damage to any target.
// Madness {R} (If you discard this card, discard it into exile. When you do,
// cast it for its madness cost or put it into your graveyard.)
// CR 702.35: Madness — exile replacement + triggered cast opportunity.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fiery-temper"),
        name: "Fiery Temper".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text:
            "Fiery Temper deals 3 damage to any target.\n\
             Madness {R} (If you discard this card, discard it into exile. When you do, \
             cast it for its madness cost or put it into your graveyard.)"
                .to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(3),
                },
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                cant_be_countered: false,
            },
            AbilityDefinition::Keyword(KeywordAbility::Madness),
            AbilityDefinition::Madness { cost: ManaCost { red: 1, ..Default::default() } },
        ],
        ..Default::default()
    }
}
