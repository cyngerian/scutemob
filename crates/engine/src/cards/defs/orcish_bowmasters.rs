// Orcish Bowmasters — {1}{B} Creature — Orc Archer 1/1
// Flash
// When this creature enters and whenever an opponent draws a card except the first
// one they draw in each of their draw steps, this creature deals 1 damage to any
// target. Then amass Orcs 1.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("orcish-bowmasters"),
        name: "Orcish Bowmasters".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Orc", "Archer"]),
        oracle_text: "Flash\nWhen this creature enters and whenever an opponent draws a card except the first one they draw in each of their draw steps, this creature deals 1 damage to any target. Then amass Orcs 1.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // ETB: deals 1 damage to any target, then amass Orcs 1.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::Amass {
                        subtype: "Orc".to_string(),
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::TargetAny],
            },
            // TODO: "whenever an opponent draws a card except the first one they draw in
            // each of their draw steps" — no draw-tracking trigger in DSL.
        ],
        ..Default::default()
    }
}
