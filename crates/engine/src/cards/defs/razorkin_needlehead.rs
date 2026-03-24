// Razorkin Needlehead — {R}{R}, Creature — Human Assassin 2/2
// This creature has first strike during your turn.
// Whenever an opponent draws a card, this creature deals 1 damage to them.
//
// CR 604.2 / CR 613.1f (Layer 6): "This creature has first strike during your turn."
// Implemented as conditional static with Condition::IsYourTurn.
//
// TODO: "Whenever an opponent draws a card, this creature deals 1 damage to them."
// DSL gap: no WheneverOpponentDrawsCard TriggerCondition variant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("razorkin-needlehead"),
        name: "Razorkin Needlehead".to_string(),
        mana_cost: Some(ManaCost { red: 2, ..Default::default() }),
        types: creature_types(&["Human", "Assassin"]),
        oracle_text: "This creature has first strike during your turn.\nWhenever an opponent draws a card, this creature deals 1 damage to them.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 604.2 / CR 613.1f (Layer 6): "This creature has first strike during your turn."
            // First strike is only active when it is the controller's turn (active player check).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::FirstStrike),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::IsYourTurn),
                },
            },
            // TODO: "Whenever an opponent draws a card, this creature deals 1 damage to them."
            // DSL gap: no WheneverOpponentDrawsCard TriggerCondition variant.
        ],
        ..Default::default()
    }
}
