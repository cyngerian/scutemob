// Squee, Goblin Nabob — {2}{R}, Legendary Creature — Goblin 1/1
// At the beginning of your upkeep, you may return this card from your graveyard to your hand.
//
// CR 603.3 / PB-35 (TriggerZone::Graveyard): Upkeep trigger that fires from the graveyard.
// Squee returns itself to hand each upkeep while it is in your graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("squee-goblin-nabob"),
        name: "Squee, Goblin Nabob".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Goblin"]),
        oracle_text: "At the beginning of your upkeep, you may return this card from your graveyard to your hand.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 603.3: Upkeep trigger fires from the graveyard (TriggerZone::Graveyard).
            // Returns Squee from the graveyard to its controller's hand.
            // TODO: Oracle says "you may" — currently non-optional (bot always returns).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: Some(TriggerZone::Graveyard),
            },
        ],
        ..Default::default()
    }
}
