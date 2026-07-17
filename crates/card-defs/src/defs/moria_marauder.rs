// Moria Marauder — {R}{R}, Creature — Goblin Warrior 1/1
// Double strike; whenever a Goblin or Orc you control deals combat damage to a player,
// exile top card of library, may play it this turn.
// TODO: Trigger condition "whenever a Goblin or Orc you control deals combat damage to a player"
// requires a subtype-filtered WhenDealsCombatDamageToPlayer trigger that fires for any
// qualifying creature you control (not just this one). DSL gap: no such trigger condition.
// Deferred (multi_type_filter + non-self trigger gap).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("moria-marauder"),
        name: "Moria Marauder".to_string(),
        mana_cost: Some(ManaCost {
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Double strike\nWhenever a Goblin or Orc you control deals combat damage to \
                      a player, exile the top card of your library. You may play that card this \
                      turn."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::DoubleStrike),
            // TODO: "Whenever a Goblin or Orc you control deals combat damage to a player"
            // DSL gap: no subtype-filtered combat damage trigger for other creatures you control.
        ],
        completeness: Completeness::partial(
            "The trigger is NOT a blocker: \
             TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer takes an \
             Option<TargetFilter> on the damage-dealing creature (card_definition.rs:3232) and \
             TargetFilter::has_subtypes gives OR semantics for 'Goblin or Orc' \
             (card_definition.rs:2837). The live blocker is the effect: no Effect variant exiles \
             the top card of your library, and no 'you may play that card this turn' permission \
             grant exists. Shares this primitive with laelia_the_blade_reforged, mystic_forge, \
             mindleecher.",
        ),
        ..Default::default()
    }
}
