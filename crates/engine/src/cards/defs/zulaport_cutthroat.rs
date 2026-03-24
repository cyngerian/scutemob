// Zulaport Cutthroat — {1}{B}, Creature — Human Rogue 1/1.
// "Whenever Zulaport Cutthroat or another creature you control dies, each opponent
// loses 1 life and you gain 1 life for each opponent that lost life."
// CR 603.2/603.10a: WheneverCreatureDies trigger; controller_you because oracle says
// "this creature or another creature you control." DrainLife captures the "gain life
// equal to total actually lost" semantics (CR 702.101a).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zulaport-cutthroat"),
        name: "Zulaport Cutthroat".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Rogue"]),
        oracle_text: "Whenever Zulaport Cutthroat or another creature you control dies, each opponent loses 1 life and you gain 1 life for each opponent that lost life.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 603.10a: controller_you because oracle says "this creature or another creature
            // you control dies." Self is covered since Zulaport is your creature.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(TargetController::You), exclude_self: false, nontoken_only: false },
                effect: Effect::DrainLife { amount: EffectAmount::Fixed(1) },
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}
