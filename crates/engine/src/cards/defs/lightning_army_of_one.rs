// Lightning, Army of One — {1}{R}{W}, Legendary Creature — Human Soldier 3/2
// First strike, trample, lifelink
// Stagger — Whenever Lightning deals combat damage to a player, until your next turn,
// if a source would deal damage to that player or a permanent that player controls,
// it deals double that damage instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lightning-army-of-one"),
        name: "Lightning, Army of One".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "First strike, trample, lifelink\nStagger — Whenever Lightning deals combat damage to a player, until your next turn, if a source would deal damage to that player or a permanent that player controls, it deals double that damage instead.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            // CR 614.1 / CR 510.3a: Stagger — triggered ability fires on combat damage.
            // "Until your next turn, if a source would deal damage to that player or a
            // permanent that player controls, it deals double that damage instead."
            // PlayerId(0) in ToPlayerOrTheirPermanents is resolved to ctx.damaged_player
            // at execution time; PlayerId(0) in UntilYourNextTurn is resolved to
            // ctx.controller (Lightning's controller).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::RegisterReplacementEffect {
                    trigger: ReplacementTrigger::DamageWouldBeDealt {
                        target_filter: DamageTargetFilter::ToPlayerOrTheirPermanents(PlayerId(0)),
                    },
                    modification: ReplacementModification::DoubleDamage,
                    duration: EffectDuration::UntilYourNextTurn(PlayerId(0)),
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
