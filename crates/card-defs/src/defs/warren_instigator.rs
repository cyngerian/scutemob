// Warren Instigator — {R}{R}, Creature — Goblin Berserker 1/1
// Double strike
// Whenever this creature deals damage to an opponent, you may put a Goblin
// creature card from your hand onto the battlefield.
//
// PB-DX36 (`OOS-CARDS2-6`): the trigger CONDITION is now expressible —
// TriggerCondition::WhenDealsDamage { recipient: DamageRecipient::Opponent }
// (CR 603.2) closes the "deals damage to an opponent" gap this def used to
// carry (it previously had no trigger authored at all). Two blockers survive:
// (a) no effect puts a FILTERED (Goblin creature) card from hand onto the
// battlefield — Effect::PutLandFromHandOntoBattlefield is land-only; (b) the
// costless "you may" is inexpressible (see goblin_lackey/curiosity/ophidian_eye
// for the same gap).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("warren-instigator"),
        name: "Warren Instigator".to_string(),
        mana_cost: Some(ManaCost {
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Berserker"]),
        oracle_text: "Double strike\nWhenever this creature deals damage to an opponent, you may \
                      put a Goblin creature card from your hand onto the battlefield."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::DoubleStrike),
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDealsDamage {
                    recipient: DamageRecipient::Opponent,
                },
                // TODO: "put a Goblin creature card from hand onto battlefield" — needs
                // MoveZone from hand with subtype filter. Using Nothing stub.
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial(
            "Blocked: (a) no effect puts a filtered (Goblin creature) card from hand onto the \
             battlefield — Effect::PutLandFromHandOntoBattlefield is land-only; (b) 'you may' is \
             inexpressible (Effect::Choose always takes the first option, effects/mod.rs:3190). \
             Trigger currently resolves to Effect::Nothing.",
        ),
        ..Default::default()
    }
}
