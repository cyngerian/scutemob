// Goblin Lackey — {R}, Creature — Goblin 1/1
// Whenever this creature deals damage to a player, you may put a Goblin
// permanent card from your hand onto the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-lackey"),
        name: "Goblin Lackey".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Whenever this creature deals damage to a player, you may put a Goblin \
                      permanent card from your hand onto the battlefield."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            // CR 603.2 (PB-DX36, `OOS-CARDS2-6`): "deals damage to a player" is ANY
            // damage, not combat-only — WhenDealsDamage { recipient: Player } closes
            // the under-fire-on-noncombat-damage blocker this def used to carry.
            trigger_condition: TriggerCondition::WhenDealsDamage {
                recipient: DamageRecipient::Player,
            },
            // TODO: "put a Goblin from hand onto battlefield" — needs MoveZone from
            // hand with subtype filter. Using Nothing stub.
            effect: Effect::Nothing,
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        completeness: Completeness::partial(
            "Blocked: (a) no effect puts a filtered (Goblin permanent) card from hand onto the \
             battlefield — Effect::PutLandFromHandOntoBattlefield is land-only; (b) 'you may' is \
             inexpressible (Effect::Choose always takes the first option, effects/mod.rs:3190). \
             Trigger currently resolves to Effect::Nothing.",
        ),
        ..Default::default()
    }
}
