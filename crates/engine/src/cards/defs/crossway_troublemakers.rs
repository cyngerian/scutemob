// Crossway Troublemakers — {5}{B}, Creature — Vampire 5/5
// Attacking Vampires you control have deathtouch and lifelink.
// Whenever a Vampire you control dies, you may pay 2 life. If you do, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crossway-troublemakers"),
        name: "Crossway Troublemakers".to_string(),
        mana_cost: Some(ManaCost { generic: 5, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Attacking Vampires you control have deathtouch and lifelink.\nWhenever a Vampire you control dies, you may pay 2 life. If you do, draw a card.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // TODO: "Attacking Vampires you control have deathtouch and lifelink" —
            //   conditional static grant (only while attacking) not fully expressible.
            // Whenever a Vampire you control dies, draw (pay 2 life simplified away)
            // TODO: WheneverCreatureDies lacks subtype filter (Vampire only).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(TargetController::You), exclude_self: false, nontoken_only: false },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
