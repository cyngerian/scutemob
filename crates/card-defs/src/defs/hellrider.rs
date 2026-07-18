// Hellrider — {2}{R}{R}, Creature — Devil 3/3
// Haste
// Whenever a creature you control attacks, Hellrider deals 1 damage to the player or
// planeswalker it's attacking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hellrider"),
        name: "Hellrider".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Devil"]),
        oracle_text: "Haste\nWhenever a creature you control attacks, Hellrider deals 1 damage to \
                      the player or planeswalker it's attacking."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 508.1m / CR 603.2: "Whenever a creature you control attacks, Hellrider deals 1
            // damage to the player or planeswalker it's attacking."
            // PB-EF3 (EF-W-MISS-4): EffectTarget::AttackTarget resolves to the specific
            // player or planeswalker the triggering attacker is/was attacking (CR 508.4/506.4c).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
                    filter: None,
                },
                effect: Effect::DealDamage {
                    target: EffectTarget::AttackTarget,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
