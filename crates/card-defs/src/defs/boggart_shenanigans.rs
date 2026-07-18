// Boggart Shenanigans — {2}{R} Kindred Enchantment — Goblin
// Whenever another Goblin you control is put into a graveyard from the battlefield,
// you may have this enchantment deal 1 damage to target player or planeswalker.
//
// "You may have this deal 1 damage" has no pay-cost — there is no non-gated optionality
// primitive for that shape (only Effect::MayPayThenEffect, which requires an actual cost).
// Per the W5 "you may [effect]" convention, a 1-damage ping targeted at an opponent's
// player/planeswalker is strictly beneficial, so it is modeled as mandatory (LOW: a bot/
// controller cannot decline the trigger, which could matter if they'd rather not reveal a
// target, but there is no downside to actually dealing the damage).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boggart-shenanigans"),
        name: "Boggart Shenanigans".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Kindred, CardType::Enchantment], &["Goblin"]),
        oracle_text: "Whenever another Goblin you control is put into a graveyard from the \
                      battlefield, you may have this enchantment deal 1 damage to target player \
                      or planeswalker."
            .to_string(),
        abilities: vec![
            // CR 603.10a: "another Goblin you control is put into a graveyard from the
            // battlefield" = a Goblin you control dies, excluding this enchantment (which is
            // never itself a Goblin creature that can die from the battlefield, but exclude_self
            // matches the "another" wording faithfully).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: true,
                    nontoken_only: false,
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Goblin".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
