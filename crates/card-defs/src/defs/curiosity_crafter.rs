// Curiosity Crafter — {3}{U}, Creature — Bird Wizard 3/3
// Flying
// You have no maximum hand size.
// Whenever a creature token you control deals combat damage to a player, draw a card.
//
// "No maximum hand size" expressed via KeywordAbility::NoMaxHandSize (PB-AC8).
// The token-only combat-damage trigger uses TargetFilter::is_token, which is checked
// on the combat_damage_filter path (abilities.rs, CR 510.3a / CR 603.2c). Controller
// scoping is applied by that same path, so the filter carries only the token predicate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("curiosity-crafter"),
        name: "Curiosity Crafter".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Bird", "Wizard"]),
        oracle_text: "Flying\nYou have no maximum hand size.\nWhenever a creature token you \
                      control deals combat damage to a player, draw a card."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::NoMaxHandSize),
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition:
                    TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer {
                        filter: Some(TargetFilter {
                            is_token: true,
                            ..Default::default()
                        }),
                    },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
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
